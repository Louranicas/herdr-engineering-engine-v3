# Closed offline Rust inputs for quality checks

The isolated quality runner consumes the reviewed lock-bound registry archives
available in the local Cargo cache. T05 had 43; APP-01 (2026-09-25) adds signal-hook 0.4.4 and signal-hook-registry 1.4.8 for 45; the original T02 tuple had eleven. Each copied `.crate` file must match the exact checksum
in the copied `Cargo.lock`. The bundle also binds the entire lock-file hash and
its closed archive inventory. Setup copies no registry index, Cargo configuration
or authentication files.

## Setup and checks

This explicit setup was run once in the isolated T02 draft:

```sh
python3 -W error tools/rust-offline.py prepare \
  --lock Cargo.lock \
  --cache /var/home/Louranicas/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f \
  --output tools/rust-offline-inputs
```

Setup refuses an existing destination. It neither downloads dependencies nor
rewrites the lock. A changed lock needs an explicit new reviewed bundle; quality
checks never refresh it automatically.

Verify the existing bundle without building:

```sh
python3 -W error tools/rust-offline.py check \
  --lock Cargo.lock --bundle tools/rust-offline-inputs
python3 -W error tests/t25_quality.py
```

The quality runner copies and hashes the lock, helper, closed archive bundle,
Rust source/tests, Pi fixtures, and the exact
`evidence/implementation/T02/sdk-smoke/metadata-benign.stdout` input used by
`tests/t02_pi.rs`. It also includes
`development/transport_process.py` when the transport test target exists.
The Julia package and tooling inputs remain byte-for-byte copies of accepted T25
inputs. `evidence/quality-offline/before/inventory.json` retains their provenance,
the original runner/tests and the original eight-control result.

Before any Cargo command, the helper rechecks the copied bundle and expands a
private vendor directory. Extraction accepts regular files and directories,
requires the exact package root, refuses links/path escapes/reserved checksum
entries, and enforces file/byte/deadline bounds. It creates Cargo's per-file
`.cargo-checksum.json` with the original archive checksum as package identity.
Every expanded file's hash, size and executable status is retained.
Each package receives the remaining global file and byte budgets. Every member
and generated checksum file is checked before writing, so a later package cannot
temporarily exceed the whole-vendor limit. Copying uses fixed chunks, checks the
deadline between reads and writes, and never writes beyond the declared size.
A separate global limit of 8,192 tar members includes empty directories and is
checked before recording a member name or creating its directory. No complete
archive member list is loaded before streaming extraction.
The vendor subtree also has one global limit of 16,384 filesystem entries.
Package roots, implicit/explicit parent directories, regular files and generated
checksum files are charged before each individual creation. The vendor root
itself is excluded. Verification checks directory inventory and counts as well
as file identity, so adding empty directories cannot bypass the bound. The
separate fixed Cargo-home directory/configuration is outside this subtree cap.

A private `CARGO_HOME/config.toml` replaces crates.io with that directory source.
The runner refuses ancestor Cargo configuration, supplies a clean explicit
environment, sets Cargo offline, and passes `--locked --offline`. It verifies
copied subject files before use, and checks source/vendor/config bytes after the
run. The private Cargo configuration hash and executable hashes are retained in
the development report.

## Fault controls and limits

The focused quality controls cover missing archives, same-size substituted
archives, changed lock bytes, unlisted bundle inputs, altered extracted source,
an archive path escape, and ancestor Cargo configuration. Restored exact bytes
and regular archive members provide benign neighbors. Separate controls ensure
the Pi fixture and captured stdout paths are included in the copied subject.

Rust result checks keep the reviewed T01 counts at 24 and 27. When either T02
target/source is present, both named T02 targets must exist at their fixed paths,
the Pi target must report exactly 60 passing cases with zero failures, ignores or
filters, and the transport target must emit its exact five-control marker.
Doctests must report exactly two passes with T02, otherwise one. These fixed
expectations are not counted from candidate source. Missing target/result and
changed/filtered-count controls exercise the refusal alongside benign summaries.
Producer summaries remain development observations rather than collector truth.

These are TH-DEV integrity checks. Cargo offline constrains package resolution;
it does not prevent arbitrary build scripts or tests from attempting network or
filesystem effects. System libraries and the Python standard library are not
an independently qualified executable closure. Process, network, evidence and
hostile workspace custody retain their existing owners. No full quality matrix,
module qualification or admission follows from a passing dependency probe.

## T04 static SQLite input

The historical T04 lock includes 34 registry packages. The previous T02/T03
11-package closure remains historical evidence. `tools/sqlite-static.py` consumes
only `tools/sqlite-inputs/sqlite-amalgamation-3530400.zip`, whose complete SHA-256
is `1e71ddf93849c6a6ecf58b827c0692073d2dd7ee40196158068f7b29f422e87d`.
It checks exact source/header member identities and bounds before extraction.
Compilation and deterministic archive creation use the existing bounded process
runner, private output paths and the original work deadline. No dependency
fetch occurs during checking.

The reviewed native feature closure includes rusqlite backup/hooks/modern_sqlite.
Upstream libsqlite3-sys transitively enables its default minimum-version feature
and pkg-config/vcpkg packages; modern_sqlite selects bundled Rust FFI declarations,
not a bundled SQLite runtime. The explicit static library/include paths and
SQLITE3_NO_PKG_CONFIG=1 prevent ambient SQLite lookup. Runtime tests independently
check the selected version, source identity and connection profile. Full host
compiler dependency custody and installable package qualification remain later
release obligations.

T03 requires its exact 34-case target; T04 requires its exact 68-case library
suite and store/SQL/input files. Missing source, absent summaries, changed counts
or skipped cases refuse the matrix. The 38 setup controls verify this behavior
with benign neighbors, without deriving case floors from candidate output.

## T05 bounded TOML input

At T05 the lock included 43 registry archives (45 since APP-01's two signal crates). Nine TOML/transitive archives are
added to the unchanged earlier package versions; the manifest binds the complete
new lock hash. TOML 1.1.5+spec-1.1.0 enables only std, serde, parse and display.
The parser's unbounded feature remains disabled. The existing rustix 1.1.4 adds
rand for bounded nonblocking UUID generation. SQLite's exact static runtime and
FFI feature tuple remain unchanged.

The quality runner compares twelve selected package/version/feature tuples with
fixed reviewed expectations, not values inferred from candidate output. The
43 setup controls cover the T05 paths, declared test targets, exact library
namespaces, feature drift and pending mutation declarations. The existing bounded
archive extraction and closed Cargo directory-source setup remain the sole path.
