"""One refusal-site sweep, shared by every module's `tools/check-*-sites` (SK-07/08, WF-08, PI).

A case per refusal CODE is not a case per SITE: a module that raises N codes from M > N places
has M - N sites with a neighbour that can answer for them. So each site is rewritten to a no-op in
turn and the module's suite is re-run, and a site counts as KILLED only when the test named for
it in the caller's `expected` table fails. A green suite is a survivor; a red suite that does not
fail the named test is KILLED-WRONG-REASON -- an import error, an unrelated case, or a census
noticing a code vanish -- and counts as a survivor too: nothing shows the site's own behaviour is
pinned. (Before this module, two of the three sweeps credited any non-zero exit.)

A site is keyed `<enclosing function>:<code>:<ordinal of that code within the function>`, not by
line number, so an edit elsewhere in the file does not re-key it. Every site found must have an
entry and every entry must name a site found: the table is reconciled against the enumeration in
both directions, so neither a new site nor a deleted one passes silently.

`--discover` prints, for every site, the tests its neuter fails; it is how a table is first
written, and it never prints a verdict.
"""

import ast
import re
import subprocess
import sys


def sites(text, callee="refuse"):
    """Every `callee(...)` call, as (line, column, end_line, end_column, code, key).

    Read from the tree rather than by regular expression: a call spanning several lines has no
    single textual form, and a pattern that missed one would shrink the denominator silently, in
    the flattering direction. A call outside any function is keyed `<module>`.
    """
    found = []
    tree = ast.parse(text)
    owners = [(tree, "<module>")] + [(node, node.name) for node in ast.walk(tree)
                                     if isinstance(node, ast.FunctionDef)]
    owner_of = {}
    for owner, name in owners:
        for node in ast.walk(owner):
            if isinstance(node, ast.Call):
                owner_of[id(node)] = name   # innermost owner wins: walked after its parent
    for node in ast.walk(tree):
        if (isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                and node.func.id == callee):
            code = (node.args[0].value if node.args and isinstance(node.args[0], ast.Constant)
                    else "?")
            found.append((node.lineno, node.col_offset, node.end_lineno, node.end_col_offset,
                          code, owner_of[id(node)]))
    ordinal = {}
    keyed = []
    for line, col, end_line, end_col, code, owner in sorted(found):
        ordinal[(owner, code)] = ordinal.get((owner, code), 0) + 1
        keyed.append((line, col, end_line, end_col, code,
                      f"{owner}:{code}:{ordinal[(owner, code)]}"))
    return keyed


def failed_tests(stderr):
    """The test names unittest reported as FAIL or ERROR in this run."""
    return set(re.findall(r"^(?:FAIL|ERROR): (\w+) ", stderr, re.MULTILINE))


def neuter(text, site):
    """Replace one call with `None`, keeping every other byte."""
    line_no, col, end_line, end_col = site[:4]
    lines = text.splitlines(keepends=True)
    head = lines[line_no - 1][:col]
    tail = lines[end_line - 1][end_col:]
    return "".join(lines[: line_no - 1]) + head + "None" + tail + "".join(lines[end_line:])


def run_tests(tests):
    # `-B`: a suite that loads its module through a source loader trusts a cached .pyc whose
    # recorded mtime (whole seconds) and size match. Two neutered variants of equal length written
    # in one second would then run the FIRST variant's bytecode and credit the wrong site;
    # measured on the pi module's plant battery, where it did exactly that.
    return subprocess.run([sys.executable, "-B", "-W", "error", str(tests)],
                          capture_output=True, text=True, check=False)


def main(source, tests, expected, callee="refuse", argv=None):
    """Sweep `source`'s `callee` sites against `tests`; return the process exit code."""
    argv = sys.argv[1:] if argv is None else argv
    discover = argv == ["--discover"]
    original = source.read_text()
    all_sites = sites(original, callee)
    if not all_sites:
        raise SystemExit("no refusal sites found; the enumeration is broken, not the module")
    if not discover:
        found_keys = {site[5] for site in all_sites}
        unlisted = sorted(found_keys - set(expected))
        stale = sorted(set(expected) - found_keys)
        if unlisted or stale:
            raise SystemExit(f"the expected table does not match the sites found: "
                             f"unlisted={unlisted} stale={stale}")
    before = run_tests(tests)
    if before.returncode != 0:
        raise SystemExit("the suite is not green before the sweep:\n" + before.stdout + before.stderr)

    survivors = []
    wrong_reason = []
    try:
        for index, site in enumerate(all_sites, 1):
            source.write_text(neuter(original, site))
            result = run_tests(tests)
            source.write_text(original)
            failed = failed_tests(result.stderr)
            if discover:
                print(f"{site[5]}\t{result.returncode}\t{' '.join(sorted(failed))}", flush=True)
                continue
            named = expected[site[5]]
            if result.returncode == 0:
                status = "SURVIVED"
                survivors.append(site)
            elif named in failed:
                status = f"killed by {named}"
            else:
                status = f"KILLED-WRONG-REASON (not {named})"
                wrong_reason.append(site)
            print(f"  {index:2}/{len(all_sites)} line {site[0]:4} {site[5]} {status}", flush=True)
    finally:
        source.write_text(original)
    if discover:
        print(f"discovered sites={len(all_sites)}")
        return 0

    killed = len(all_sites) - len(survivors) - len(wrong_reason)
    print(f"\nsites={len(all_sites)} killed={killed} wrong_reason={len(wrong_reason)} "
          f"survived={len(survivors)}")
    for label, group in (("SURVIVOR", survivors), ("WRONG-REASON", wrong_reason)):
        for site in group:
            print(f"  {label} {source.name}:{site[0]} {site[5]} expected {expected[site[5]]}")
    passed = not survivors and not wrong_reason
    print("verdict=" + ("PASS" if passed else "FAIL"))
    return 0 if passed else 1
