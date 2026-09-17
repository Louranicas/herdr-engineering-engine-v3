using JuliaFormatter
using SHA
using TOML

VERSION == v"1.12.7" || error("Julia version differs")
pkgversion(JuliaFormatter) == v"2.12.5" || error("JuliaFormatter version differs")

# Format only authored Julia bodies. Corpus comments have a separate generator.
function authored_parts(text)
    marker = findfirst(r"(?m)^# HEE3-ANCHORS-END$", text)
    marker === nothing && return "", text
    boundary = findnext('\n', text, last(marker))
    boundary === nothing && error("Unterminated managed anchor block")
    return text[begin:boundary], text[nextind(text, boundary):end]
end

function main(args)
    isempty(args) && error("No Julia subjects selected")
    fix = first(args) == "--fix"
    files = fix ? args[2:end] : args
    isempty(files) && error("No Julia subjects selected")
    # Independent small formatting expectations, including a nearby valid input.
    fault = "function f()\n1\nend\n"
    benign = "function f()\n    1\nend\n"
    JuliaFormatter.format_text(fault) == benign || error("Formatter fault control differs")
    JuliaFormatter.format_text(benign) == benign ||
        error("Formatter benign control differs")
    literal = "marker = \"HEE3-ANCHORS-END\"\n"
    authored_parts(literal) == ("", literal) || error("Marker literal split source")
    anchors = "# HEE3-ANCHORS-BEGIN\n# fixture\n# HEE3-ANCHORS-END\n"
    authored_parts(anchors * fault) == (anchors, fault) || error("Managed comments differ")
    subjects = Dict{String,String}()
    for path in files
        before = read(path, String)
        anchors, body = authored_parts(before)
        formatted = JuliaFormatter.format_text(body)
        JuliaFormatter.format_text(formatted) == formatted ||
            error("Non-idempotent formatter: $path")
        after = anchors * formatted
        if before != after
            fix || error("Julia formatting differs: $path")
            write(path, after)
        end
        subjects[path] = bytes2hex(sha256(after))
    end
    TOML.print(
        stdout,
        Dict(
            "scope" => "Pinned Julia authored-source formatting; no module admission",
            "julia" => string(VERSION),
            "formatter" => string(pkgversion(JuliaFormatter)),
            "fault_and_benign_controls" => "pass",
            "subjects_sha256" => subjects,
        ),
    )
end

main(ARGS)
