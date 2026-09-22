use super::{
    Error, ErrorKind, Evidence, Frozen, NamespaceReport, Snapshot, budget, fail, file, name, r,
    raw, text, u64_oracle, workload,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

pub(super) fn tool_pins(tools: &workload::Tools) -> Result<BTreeMap<PathBuf, [u8; 32]>, Error> {
    if tools.runtime_files.len() > 4096
        || tools.compiler.namespace != std::path::Path::new("/toolchain/bin/rustc")
    {
        return Err(fail(ErrorKind::Binding));
    }
    let mut pins = BTreeMap::new();
    for file in tools.runtime_files.iter().chain([&tools.shim]) {
        if pins
            .insert(file.namespace.clone(), file.sha256)
            .is_some_and(|old| old != file.sha256)
        {
            return Err(fail(ErrorKind::Binding));
        }
    }
    Ok(pins)
}
fn input(pins: &mut BTreeMap<PathBuf, [u8; 32]>, path: &str, bytes: &[u8]) {
    pins.insert(path.into(), Sha256::digest(bytes).into());
}
fn matches(report: &NamespaceReport, pins: &BTreeMap<PathBuf, [u8; 32]>) -> bool {
    let facts = &report.observer.facts;
    facts.source_sha256_before == *pins
        && facts.source_sha256_after == *pins
        && facts.source_sha256_postrun_host == *pins
        && report.postrun_sources_match
}
fn output(
    snapshot: &Snapshot,
    path: &str,
    evidence: &mut Evidence<'_>,
) -> Result<(r::ArtifactV1, [u8; 32]), Error> {
    let bytes = file(snapshot, path)?;
    let payload = raw(evidence, bytes)?;
    Ok((
        r::ArtifactV1 {
            object: payload.as_ref().clone(),
            role: name(path)?,
            required: true,
            truncated: false,
            availability: r::ArtifactV1Availability::Available,
            reason: text(
                "Exact captured compiler output; separate from the frozen result source.",
            )?,
        },
        Sha256::digest(bytes).into(),
    ))
}
/// Bind actual namespace byte maps and retain captured outputs without inventing a source subject.
pub(super) fn capture_outputs(
    evidence: &mut Evidence<'_>,
    frozen: &Frozen,
    run: &workload::Run,
    reports: &[&NamespaceReport],
    deadline: Instant,
) -> Result<(bool, Vec<r::ArtifactV1>), Error> {
    budget(deadline)?;
    if run.outputs.len() > 2 {
        return Err(fail(ErrorKind::Binding));
    }
    let mut intact = run.outputs.len() == reports.len().saturating_sub(1);
    let mut artifacts = Vec::new();
    let mut digests = Vec::new();
    for (snapshot, path) in run
        .outputs
        .iter()
        .zip(["libstrict_u64_workload.rlib", "workload-driver"])
    {
        intact &= snapshot.readback_source(deadline).is_ok();
        let (artifact, digest) = output(snapshot, path, evidence)?;
        artifacts.push(artifact);
        digests.push(digest);
    }
    let mut pins = frozen.tool_pins.clone();
    pins.insert(frozen.compiler_pin.0.clone(), frozen.compiler_pin.1);
    input(
        &mut pins,
        "/frozen/source/lib.rs",
        file(&frozen.sources[1].snapshot, "src/lib.rs")?,
    );
    intact &= matches(reports[0], &pins);
    if let Some(report) = reports.get(1) {
        let mut pins = frozen.tool_pins.clone();
        pins.insert(frozen.compiler_pin.0.clone(), frozen.compiler_pin.1);
        input(
            &mut pins,
            "/frozen/public-wrapper.rs",
            file(&frozen.protected, "public-wrapper.rs")?,
        );
        if let Some(digest) = digests.first() {
            pins.insert("/frozen/libstrict_u64_workload.rlib".into(), *digest);
        } else {
            intact = false;
        }
        intact &= matches(report, &pins);
    }
    if let Some(report) = reports.get(2) {
        let mut pins = frozen.tool_pins.clone();
        // Runtime files may legitimately include rustc; match the fixed workload's
        // actual map from retained runtime pins plus shim and linked candidate.
        if let Some(digest) = digests.get(1) {
            pins.insert("/frozen/bin/workload-driver".into(), *digest);
        } else {
            intact = false;
        }
        let oracle = u64_oracle::FrozenOracle::from_bytes(&frozen.oracle_bytes)
            .map_err(|_| fail(ErrorKind::Binding))?;
        input(&mut pins, "/frozen/inputs.hex", &oracle.public_inputs());
        intact &= matches(report, &pins);
    }
    Ok((intact, artifacts))
}
