# Fixed T06 application composition

These two existing packages are owned by app. They are an internal offline development profile, not the T28 public task.submit surface or an installed service. Both depend on the single repository engine; the frontend also depends on the sibling task-runtime package. No nested candidate/runtime source copies exist.

From the repository root, select packages explicitly:

```sh
cargo test --offline --locked --manifest-path development/t06/task-runtime/Cargo.toml --all-targets
cargo test --offline --locked --manifest-path development/t06/fixed-runtime-frontend/Cargo.toml --all-targets
cargo build --offline --locked --release --manifest-path development/t06/fixed-runtime-frontend/Cargo.toml
```

Use the declared Rust 1.98.0, retained offline vendor and exact SQLite 3.53.4 static inputs; ambient SQLite is not accepted. The isolated integration run-check.py records those environments, unique targets, short temporary directories and the bounded existing process owner. Root cargo test alone does not cover independent workspaces.

The built fixed binary accepts exactly `run <frozen-manifest> <fresh-absolute-output>`. A manifest must be created from a successful exact-source build record by prepare-inputs.py. Preparation copies/hashes only; no execution is performed by that helper. Reviewed shim, review bundle and finite host file inventory remain explicit pinned historical inputs referenced by that helper; this is not yet a standalone deployment package. Do not run the binary without coordinator allocation of the bounded namespace profile.

Schema001 remains an unreleased draft. Old draft ledgers refuse under a changed whole-file checksum; never rewrite history. No migration release/upgrade, module admission or deployment is claimed.
