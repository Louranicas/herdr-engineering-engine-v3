# Fixed EOF entrypoint; no caller-selected code/path, task state or policy writer.
using HabitatAnalysis
using JSON3
using SHA

VERSION == v"1.12.7" || error("unsupported Julia version")
const MAX_INPUT = 1_048_576
const MAX_OUTPUT = 65_536
function header_binding(raw)
    try
        HabitatAnalysis.framing(raw)
        HabitatAnalysis.numeric_spelling(raw)
        q = HabitatAnalysis.decode_strings(JSON3.read(String(copy(raw)); allow_inf = false))
        HabitatAnalysis.closed(
            q,
            (
                :protocol,
                :version,
                :request_id,
                :subject,
                :cutoff_unix_ms,
                :expires_unix_ms,
                :recipe,
                :units,
                :shape,
                :observations,
            ),
        )
        q.protocol == "hee3.analysis" && q.version == 1 || return nothing
        HabitatAnalysis.uuid(q.request_id)
        HabitatAnalysis.closed(
            q.subject,
            (:task_id, :attempt_id, :generation, :artifact_sha256),
        )
        HabitatAnalysis.uuid(q.subject.task_id)
        HabitatAnalysis.uuid(q.subject.attempt_id)
        HabitatAnalysis.decimal(q.subject.generation)
        q.subject.artifact_sha256 isa AbstractString &&
        occursin(r"\Asha256:[0-9a-f]{64}\z", q.subject.artifact_sha256) || return nothing
        HabitatAnalysis.decimal(q.cutoff_unix_ms)
        HabitatAnalysis.decimal(q.expires_unix_ms)
        HabitatAnalysis.closed(q.recipe, (:id, :version))
        q.recipe.id == "descriptive" && q.recipe.version == 1 || return nothing
        HabitatAnalysis.closed(q.units, (:elapsed, :usage))
        q.units.elapsed == "ms" && q.units.usage == "token" || return nothing
        return (;
            request_id = q.request_id,
            subject = q.subject,
            recipe = q.recipe,
            cutoff_unix_ms = q.cutoff_unix_ms,
            expires_unix_ms = q.expires_unix_ms,
            units = q.units,
        )
    catch error
        error isa AnalysisError || error isa ArgumentError || rethrow()
        return nothing
    end
end

raw = read(stdin, MAX_INPUT + 1)
try
    length(raw) <= MAX_INPUT || throw(AnalysisError(:bound))
    result = analyze(raw, UInt64(floor(time() * 1000)))
    length(result) <= MAX_OUTPUT || throw(AnalysisError(:bound))
    write(stdout, result)
    flush(stdout)
catch exception
    if exception isa AnalysisError
        # The exact request hash is available only if the complete bounded body
        # reached EOF. Malformed input has no trusted decoded subject identity.
        binding = length(raw) <= MAX_INPUT ? "sha256:" * bytes2hex(sha256(raw)) : nothing
        header = length(raw) <= MAX_INPUT ? header_binding(raw) : nothing
        result = JSON3.write((
            protocol = "hee3.analysis",
            version = 1,
            kind = "error",
            request_sha256 = binding,
            binding = header,
            code = string(exception.code),
            diagnostic = "analysis refusal",
        ))
        write(stdout, result)
        flush(stdout)
        exit(2)
    end
    rethrow()
end
