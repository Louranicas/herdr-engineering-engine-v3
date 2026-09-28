//! HT0 (2026-09-29): the real toolchain, run through the candidate namespace. Every other test
//! pins `runtime_files = []`, so no test had compiled and linked a candidate end to end. Here the
//! class tools are enumerated from the world — the toolchain behind `rustc`, the candidate target's
//! library directory, `rust-lld`, and the loader and libraries `ldd` names — each pinned by its
//! digest, and `workload::collect` runs the three stages with the real bwrap and shim.
//!
//! The expected results come from an independent source: a Python reading of Rust's documented
//! `u64::from_str` and of the reference patch, run over the same oracle
//! (`~/hee3-evidence/T28/HT0-musl-20260929/oracle-expectation.py`): the reference fails 0 of 335
//! cases and the base fails 139.
use habitat_engine::app::workload::{
    self, CANDIDATE_TARGET, COMPILER_DESTINATION, Outcome, Plan, Tools,
};
use habitat_engine::worker::{namespace::ReadOnlyFile, workspace::Snapshot};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

static NEXT: AtomicU64 = AtomicU64::new(0);
/// A hang guard, not a measurement: the two compiles and the driver finish in seconds.
const GUARD: Duration = Duration::from_secs(150);

const REFERENCE: &[u8] =
    include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs");
const BASE: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/base/src/lib.rs");
const ORACLE: &[u8] = include_bytes!("../evaluation/tasks/WL-U64-PARSE-001/v1/oracle/cases.json");
const WRAPPER: &[u8] = include_bytes!("../evaluation/harnesses/u64-public-wrapper.rs");

/// The toolchain directory behind `rustc` (`RUSTC` when the gate sets it, else `rustc --print
/// sysroot`), never a rustup proxy.
fn sysroot() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("RUSTC") {
        let rustc = PathBuf::from(path).canonicalize()?;
        let bin = rustc.parent().ok_or("RUSTC has no directory")?;
        return Ok(bin
            .parent()
            .ok_or("RUSTC is not under a toolchain")?
            .to_path_buf());
    }
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()?;
    if !output.status.success() {
        return Err(format!("rustc --print sysroot: {}", output.status).into());
    }
    Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()).canonicalize()?)
}

fn pinned(host: &Path, namespace: impl Into<PathBuf>) -> Result<ReadOnlyFile> {
    let host = host.canonicalize()?;
    Ok(ReadOnlyFile {
        sha256: Sha256::digest(fs::read(&host)?).into(),
        host,
        namespace: namespace.into(),
    })
}

fn files_under(directory: &Path, found: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            files_under(&path, found)?;
        } else {
            found.push(path);
        }
    }
    Ok(())
}

/// The loader and shared libraries `ldd` names for `binary`, outside the toolchain (which is
/// enumerated from its own directory).
fn host_libraries(binary: &Path, toolchain: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("ldd").arg(binary).output()?;
    if !output.status.success() {
        return Err(format!("ldd {}: {}", binary.display(), output.status).into());
    }
    let mut found = Vec::new();
    for line in String::from_utf8(output.stdout)?.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let path = match fields.as_slice() {
            [_, "=>", path, ..] | [path, ..] if path.starts_with('/') => PathBuf::from(path),
            _ => continue,
        };
        if !path.canonicalize()?.starts_with(toolchain) {
            found.push(path);
        }
    }
    Ok(found)
}

/// The class tools for the real toolchain: `rustc` at the compiler's destination; every file of
/// the toolchain's `lib/` (the driver's shared libraries) and of the candidate target's library
/// directory at its place under `/toolchain`; `rust-lld`; and the host libraries `ldd` names for
/// `rustc` and `rust-lld`, each at `/lib64/<name>`.
fn tools() -> Result<Tools> {
    let toolchain = sysroot()?;
    let rustlib = toolchain.join("lib/rustlib");
    let host = fs::read_dir(&rustlib)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.join("bin/rust-lld").is_file())
        .ok_or("no rust-lld under the toolchain")?;
    let lld = host.join("bin/rust-lld");
    let mut files = Vec::new();
    for entry in fs::read_dir(toolchain.join("lib"))? {
        let path = entry?.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files_under(&rustlib.join(CANDIDATE_TARGET).join("lib"), &mut files)?;
    files.push(lld.clone());
    let mut runtime = BTreeMap::new();
    for file in files {
        let namespace = Path::new("/toolchain").join(file.strip_prefix(&toolchain)?);
        runtime.insert(namespace.clone(), pinned(&file, namespace)?);
    }
    let compiler = toolchain.join("bin/rustc");
    for binary in [&compiler, &lld] {
        for library in host_libraries(binary, &toolchain)? {
            let name = library.file_name().ok_or("a library path with no name")?;
            let namespace = Path::new("/lib64").join(name);
            runtime.insert(namespace.clone(), pinned(&library, namespace)?);
        }
    }
    Ok(Tools {
        bwrap: "/usr/bin/bwrap".into(),
        compiler: pinned(&compiler, COMPILER_DESTINATION)?,
        shim: pinned(
            Path::new(env!("CARGO_BIN_EXE_hee-namespace-shim")),
            "/shim/namespace-shim",
        )?,
        runtime_files: runtime.into_values().collect(),
        namespace_directories: Vec::new(),
    })
}

fn private(path: &Path) -> Result<()> {
    Ok(fs::DirBuilder::new().mode(0o700).create(path)?)
}

fn frozen(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes)?;
    Ok(fs::set_permissions(
        path,
        fs::Permissions::from_mode(0o400),
    )?)
}

/// Every directory under `root` writable again, then `root` removed: the frozen outputs are
/// read-only, and a test that leaves its scratch behind is a leak (RA2 finding).
fn remove(root: &Path) -> Result<()> {
    fn open(path: &Path) -> Result<()> {
        if path.is_dir() && !path.is_symlink() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
            for entry in fs::read_dir(path)? {
                open(&entry?.path())?;
            }
        }
        Ok(())
    }
    open(root)?;
    Ok(fs::remove_dir_all(root)?)
}

/// Whether an ELF64 file has a `PT_INTERP` program header: a dynamic executable names its loader
/// there, a static one does not.
fn has_interpreter(elf: &[u8]) -> Result<bool> {
    let field = |at: usize, width: usize| -> Result<u64> {
        let bytes = elf
            .get(at..at + width)
            .ok_or("ELF shorter than its header")?;
        Ok(bytes
            .iter()
            .rev()
            .fold(0_u64, |value, byte| (value << 8) | u64::from(*byte)))
    };
    if elf.get(..5) != Some(b"\x7fELF\x02".as_slice()) {
        return Err("not an ELF64 file".into());
    }
    let offset = usize::try_from(field(0x20, 8)?)?;
    let size = usize::try_from(field(0x36, 2)?)?;
    let count = usize::try_from(field(0x38, 2)?)?;
    for index in 0..count {
        if field(offset + index * size, 4)? == 3 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn find(root: &Path, name: &str) -> Result<Option<PathBuf>> {
    let mut files = Vec::new();
    files_under(root, &mut files)?;
    Ok(files
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name)))
}

/// One real run of the three stages over `source`, with the scratch removed afterwards. Returns
/// the outcome and whether the linked driver (if any) has a loader.
fn run(tools: &Tools, source: &[u8]) -> Result<(Outcome, Option<bool>)> {
    let root = std::env::temp_dir().canonicalize()?.join(format!(
        "hee3-musl-workload-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    private(&root)?;
    let result = (|| -> Result<(Outcome, Option<bool>)> {
        for name in ["source", "source/src", "protected", "job"] {
            private(&root.join(name))?;
        }
        frozen(&root.join("source/src/lib.rs"), source)?;
        frozen(&root.join("protected/oracle.json"), ORACLE)?;
        frozen(&root.join("protected/public-wrapper.rs"), WRAPPER)?;
        let started = Instant::now();
        let source = Snapshot::capture(&root.join("source"), &[], started + GUARD)
            .map_err(|error| format!("source snapshot: {error:?}"))?;
        let protected = Snapshot::capture(&root.join("protected"), &[], started + GUARD)
            .map_err(|error| format!("protected snapshot: {error:?}"))?;
        let cancelled = AtomicBool::new(false);
        let run = workload::collect(&Plan {
            source: &source,
            protected: &protected,
            job_root: &root.join("job"),
            tools,
            deadline: started + GUARD,
            teardown_deadline: started + GUARD + Duration::from_secs(30),
            cancelled: &cancelled,
        })
        .map_err(|error| format!("collect: {error:?}"))?;
        let driver = match find(&root.join("job/frozen-driver"), "workload-driver") {
            Ok(Some(path)) => Some(has_interpreter(&fs::read(path)?)?),
            _ => None,
        };
        Ok((run.outcome, driver))
    })();
    remove(&root)?;
    result
}

fn failed_ids(outcome: &Outcome) -> Vec<&str> {
    match outcome {
        Outcome::Matched(evaluation) | Outcome::Mismatch(evaluation) => evaluation
            .vectors
            .iter()
            .filter(|vector| !vector.matched)
            .map(|vector| vector.id.as_str())
            .collect(),
        _ => Vec::new(),
    }
}

#[test]
fn reference_compiles_links_static_and_matches_every_case() -> Result<()> {
    let (outcome, interpreter) = run(&tools()?, REFERENCE)?;
    let Outcome::Matched(evaluation) = &outcome else {
        return Err(format!("reference outcome {outcome:?}").into());
    };
    assert_eq!(
        (evaluation.matched, evaluation.failed, interpreter),
        (335, 0, Some(false)),
        "the reference matches all 335 cases with a static driver (no PT_INTERP)"
    );
    Ok(())
}

#[test]
fn base_links_and_fails_the_cases_its_parser_admits() -> Result<()> {
    let (outcome, interpreter) = run(&tools()?, BASE)?;
    let Outcome::Mismatch(evaluation) = &outcome else {
        return Err(format!("base outcome {outcome:?}").into());
    };
    let failed = failed_ids(&outcome);
    assert_eq!(
        (evaluation.matched, evaluation.failed, interpreter),
        (196, 139, Some(false)),
        "the base fails 139 of 335 cases (independent Python reading), with a static driver"
    );
    for id in ["leading_zero", "plus_sign", "full_width_digit"] {
        assert!(
            failed.contains(&id),
            "base must fail {id}; failed: {failed:?}"
        );
    }
    Ok(())
}
