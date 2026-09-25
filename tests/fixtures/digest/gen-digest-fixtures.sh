#!/bin/bash
# Known answers for Snapshot::content_digest and criteria_digest (B14-P2a), produced by the world:
# coreutils (find, sha256sum, sort) for the manifest digest, Python's json.dumps for the criteria.
# Writes tests/fixtures/digest/cases.json. Run from the repository root.
set -euo pipefail
out=tests/fixtures/digest/cases.json
root=$(mktemp -d)
trap 'rm -rf "$root"' EXIT
tree=$root/tree
mkdir -m 700 "$tree"
# Entries chosen for the order and field rules: an empty directory, an empty file, an executable, `a-b`
# beside `a/b` (0x2d sorts before 0x2f), a file beside a directory that prefixes it, nested depth.
mkdir -m 700 "$tree/a" "$tree/empty" "$tree/dir" "$tree/dir/sub"
printf 'x\n' > "$tree/a-b"; printf 'y\n' > "$tree/a/b"; : > "$tree/empty.txt"
printf '#!/bin/sh\necho\n' > "$tree/run.sh"; printf 'pub fn f() {}\n' > "$tree/dir/sub/deep.rs"
printf 'z' > "$tree/dir.rs"
# The prefix cases of `a` (review P2-R1.1): 0x20, 0x21, 0x2d, 0x2f and 0x5c after it, and a non-ASCII name.
printf 's\n' > "$tree/a b"; printf 'b\n' > "$tree/a!"; printf 'k\n' > "$tree/a\\b"; printf 'e\n' > "$tree/é"
chmod 600 "$tree/a-b" "$tree/a/b" "$tree/empty.txt" "$tree/dir/sub/deep.rs" "$tree/dir.rs" \
  "$tree/a b" "$tree/a!" "$tree/a\\b" "$tree/é"
chmod 700 "$tree/run.sh"
manifest() {
  ( cd "$1"
    find . -mindepth 1 -type d -printf '%P\td\n'
    # Names from find's %P (sha256sum would escape a backslash); the exec bit is -perm /111, capture's
    # `mode & 0o111`, never access(2); hashes from stdin, lowercase hex, the first 64 columns.
    for x in x -; do
      if [ "$x" = x ]; then sel=(-perm /111); else sel=(! -perm /111); fi
      find . -mindepth 1 -type f "${sel[@]}" -printf '%P\n' | while IFS= read -r p; do
        printf '%s\tf\t%s\t%s\n' "$p" "$x" "$(sha256sum < "$p" | cut -c1-64)"
      done
    done ) | LC_ALL=C sort
}
manifest "$tree" > "$root/manifest"
mkdir -m 700 "$root/none"; manifest "$root/none" > "$root/empty-manifest"
python3 - "$tree" "$root/manifest" "$root/empty-manifest" "$out" <<'PY'
import hashlib, json, os, stat, sys, subprocess
tree, manifest, empty, out = sys.argv[1:5]
entries = []
for base, dirs, files in os.walk(tree):
    for name in sorted(dirs + files):
        path = os.path.join(base, name); rel = os.path.relpath(path, tree)
        mode = stat.S_IMODE(os.lstat(path).st_mode)
        if os.path.isdir(path): entries.append({"path": rel, "kind": "directory", "mode": mode})
        else: entries.append({"path": rel, "kind": "file", "mode": mode, "content": open(path, "rb").read().decode()})
entries.sort(key=lambda e: e["path"].encode())
text = open(manifest, "rb").read(); none = open(empty, "rb").read()
criteria = [["u64-frozen-exact-output"], ["a \"quote\"", "back\\slash", "tab\there", "ctl\u0001x", "slash/ok", "\U0001F600 non-BMP", "é"],
            ["dup", "dup"], [], ["unit\u001fseparator", "del\u007f"]]
json.dump({
  "generator": "tests/fixtures/digest/gen-digest-fixtures.sh",
  "generator_sha256": hashlib.sha256(open(__import__('os').environ.get('GEN', 'tests/fixtures/digest/gen-digest-fixtures.sh'), 'rb').read()).hexdigest(),
  "tools": [subprocess.run([t, "--version"], capture_output=True, text=True).stdout.splitlines()[0] for t in ("find", "sha256sum", "sort")],
  "tree": {"entries": entries, "manifest": text.decode(), "digest": hashlib.sha256(text).hexdigest()},
  "empty": {"manifest": none.decode(), "digest": hashlib.sha256(none).hexdigest()},
  "criteria": [{"criteria": c, "json": json.dumps(c, separators=(",", ":"), ensure_ascii=False),
                "digest": "sha256:" + hashlib.sha256(json.dumps(c, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()} for c in criteria],
}, open(out, "w"), indent=1, ensure_ascii=False)
print("entries", len(entries), "digest", hashlib.sha256(text).hexdigest()[:16])
PY
