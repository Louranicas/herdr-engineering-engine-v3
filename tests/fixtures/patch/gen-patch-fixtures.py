#!/usr/bin/env python3
"""Generate derive_patch known answers from GNU diff (the world, not my reasoning).

Each case: before/after text; expected = `diff -u --label a/P --label b/P` output
(empty when equal). Writes JSON to argv[1].
"""
import json, subprocess, sys, tempfile, os, random

P = "src/lib.rs"
def lines(n, tag="l"):
    return "".join(f"{tag}{i}\n" for i in range(n))

cases = [
    ("identical", "a\nb\nc\n", "a\nb\nc\n"),
    ("empty_to_text", "", "a\nb\n"),
    ("text_to_empty", "a\nb\n", ""),
    ("empty_to_no_newline", "", "a"),
    ("add_final_newline", "a\nb", "a\nb\n"),
    ("drop_final_newline", "a\nb\n", "a\nb"),
    ("both_no_newline_change_last", "a\nb", "a\nc"),
    ("both_no_newline_change_first", "x\nb\nc\nd\ne\nf", "y\nb\nc\nd\ne\nf"),
    ("first_line", "a\n" + lines(10), "A\n" + lines(10)),
    ("last_line", lines(10) + "z\n", lines(10) + "Z\n"),
    ("single_insert_middle", lines(10), lines(5) + "new\n" + lines(10)[len(lines(5)):]),
    ("single_delete_middle", lines(10), lines(4) + lines(10)[len(lines(5)):]),
    ("gap6_merged", "".join(f"{i}\n" for i in range(20)),
        "".join(("X\n" if i in (3, 10) else f"{i}\n") for i in range(20))),
    ("gap7_split", "".join(f"{i}\n" for i in range(20)),
        "".join(("X\n" if i in (3, 11) else f"{i}\n") for i in range(20))),
    ("crlf", "a\r\nb\r\nc\r\n", "a\r\nB\r\nc\r\n"),
    ("crlf_to_lf", "a\r\nb\r\n", "a\nb\n"),
    ("repeated_ties", "a\na\na\nb\na\na\n", "a\na\nb\na\na\na\n"),
    ("braces", "fn a() {\n}\n\nfn b() {\n}\n", "fn a() {\n}\n\nfn c() {\n}\n\nfn b() {\n}\n"),
    ("reverse", "1\n2\n3\n4\n", "4\n3\n2\n1\n"),
    ("insert_at_start_of_empty_lines", "\n\n\n", "x\n\n\n\n"),
    ("unicode", "α\nβ\nγ\n", "α\nδ\nγ\n"),
    ("no_newline_context", "a\nb\nc", "A\nb\nc"),
    ("no_newline_context_insert", "a\nb", "a\nx\nb"),
    ("only_newline_both_empty_lines", "\n", ""),
    ("empty_line_to_no_newline", "\n", "x"),
]
rng = random.Random(20260926)
alphabet = ["a\n", "b\n", "c\n", "}\n", "\n", "    x;\n"]
for i in range(60):
    before = "".join(rng.choice(alphabet) for _ in range(rng.randint(0, 30)))
    after = list(before.splitlines(keepends=True))
    for _ in range(rng.randint(0, 6)):
        op = rng.randint(0, 2)
        if op == 0 or not after:
            after.insert(rng.randint(0, len(after)), rng.choice(alphabet))
        elif op == 1:
            del after[rng.randrange(len(after))]
        else:
            after[rng.randrange(len(after))] = rng.choice(alphabet)
    after = "".join(after)
    if rng.random() < 0.15 and after.endswith("\n"):
        after = after[:-1]
    cases.append((f"random_{i:02d}", before, after))

def distance(before, after):
    """Minimal line insertions plus deletions, by the textbook LCS dynamic program over lines
    that keep their terminators: an implementation independent of the Rust Myers search."""
    a, b = before.splitlines(keepends=True), after.splitlines(keepends=True)
    row = [0] * (len(b) + 1)
    for x in a:
        prev, row = row, [0] * (len(b) + 1)
        for j, y in enumerate(b, 1):
            row[j] = prev[j - 1] + 1 if x == y else max(prev[j], row[j - 1])
    return len(a) + len(b) - 2 * row[len(b)]

out = []
with tempfile.TemporaryDirectory() as d:
    for name, before, after in cases:
        a, b = os.path.join(d, "a"), os.path.join(d, "b")
        open(a, "w", newline="").write(before); open(b, "w", newline="").write(after)
        r = subprocess.run(["diff", "-u", "--label", f"a/{P}", "--label", f"b/{P}", a, b],
                           capture_output=True)
        assert r.returncode in (0, 1), (name, r.returncode, r.stderr)
        out.append({"name": name, "before": before, "after": after,
                    "expected": r.stdout.decode("utf-8"),
                    "distance": distance(before, after)})
version = subprocess.run(["diff", "--version"], capture_output=True, text=True).stdout.splitlines()[0]
import hashlib
own = hashlib.sha256(open(__file__, "rb").read()).hexdigest()
json.dump({"generator": "tests/fixtures/patch/gen-patch-fixtures.py", "generator_sha256": own,
           "oracle": version,
           "command": f"diff -u --label a/{P} --label b/{P} <before> <after>",
           "distance": "textbook LCS over lines that keep their terminators (distance() above)",
           "path": P, "cases": out}, open(sys.argv[1], "w"), indent=1, ensure_ascii=False)
print(f"cases={len(out)} oracle={version!r}")
