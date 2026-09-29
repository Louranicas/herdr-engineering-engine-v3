//! Offline client controls. Synthetic model/runtime observations are never backend qualification.
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::worker::native::{self, Daemon, Error, FilePin, Profile, ProviderState};
use habitat_engine::worker::{
    self, Capabilities, ContractError, Feature, Finish, Phase, Request, Selection, Terminal, Usage,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
static NEXT: AtomicU64 = AtomicU64::new(0);
const MODEL: &str = "hee3-t08-fixture:qualification";
fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}
fn pin(path: &Path) -> FilePin {
    let raw = fs::read(path).unwrap();
    FilePin {
        path: path.to_owned(),
        sha256: digest(&raw),
        bytes: raw.len() as u64,
    }
}
fn request(required: &[Feature]) -> Request<'static> {
    Request {
        invocation: worker::Invocation {
            binding: worker::Binding {
                task: UuidV4::parse("08000000-0000-4000-8000-000000000001").unwrap(),
                attempt: UuidV4::parse("08000000-0000-4000-8000-000000000002").unwrap(),
                generation: "1".parse().unwrap(),
            },
            id: UuidV4::parse("08000000-0000-4000-8000-000000000003").unwrap(),
        },
        recipe: Sha256Digest::parse(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        workspace: Sha256Digest::parse(
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap(),
        adapter_profile: native::PROFILE.into(),
        selection: Selection {
            provider: native::PROVIDER.into(),
            model: MODEL.into(),
            effort: None,
        },
        required: Capabilities::new(required),
        prompt: "Return exactly seven.".into(),
    }
}
fn full() -> Request<'static> {
    request(&[Feature::FinalOutput, Feature::Identity, Feature::Usage])
}
/// A live process pinned as the fixture's daemon. Before QC-F3b the fixture pinned the test
/// executable itself (47 MiB in debug), which the adapter re-hashes through `/proc/<pid>/exe`
/// up to three times per run (`native::daemon` inside `subject` before and after generation
/// and once before the request): 87 debug SHA-256 passes over 47 MiB were the entire 10x
/// debug/release cost. `/usr/bin/sleep` is tens of KiB; the bound below is the control.
struct DaemonStandIn {
    child: Child,
}
impl DaemonStandIn {
    const EXECUTABLE: &'static str = "/usr/bin/sleep";
    /// QC-F3b control: a stand-in above this bound reintroduces the hashing cost.
    const EXECUTABLE_BOUND: u64 = 1024 * 1024;
    fn spawn() -> Self {
        let child = Command::new(Self::EXECUTABLE)
            .arg("600")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self { child }
    }
    fn daemon(&self) -> Daemon {
        let pid = self.child.id();
        let exe = pin(Path::new(&format!("/proc/{pid}/exe")));
        assert!(
            exe.bytes <= Self::EXECUTABLE_BOUND,
            "QC-F3b: pinned daemon executable {} (resolves to {}) is {} bytes, above the {} byte bound the adapter hashes up to three times per run",
            exe.path.display(),
            fs::canonicalize(&exe.path).unwrap_or_default().display(),
            exe.bytes,
            Self::EXECUTABLE_BOUND
        );
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        Daemon {
            pid,
            start_ticks: stat
                .rsplit_once(')')
                .unwrap()
                .1
                .split_whitespace()
                .nth(19)
                .unwrap()
                .parse()
                .unwrap(),
            boot_id: fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .unwrap()
                .trim()
                .into(),
            executable_sha256: exe.sha256,
            executable_bytes: exe.bytes,
        }
    }
}
impl Drop for DaemonStandIn {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
struct Rig {
    root: PathBuf,
    profile: Profile,
    scenario: Value,
    _daemon: DaemonStandIn,
}
impl Rig {
    fn new() -> Self {
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "t08-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let blobs = root.join("blobs");
        fs::DirBuilder::new().mode(0o700).create(&blobs).unwrap();
        let model = b"not a real model; finite offline fixture";
        let model_digest = digest(model);
        fs::write(blobs.join(model_digest.replace(':', "-")), model).unwrap();
        let config=serde_json::to_vec(&json!({"model_format":"gguf","model_family":"llama","model_families":["llama"],"model_type":"3.2B","file_type":"Q4_K_M","architecture":"amd64","os":"linux","rootfs":{"type":"layers","diff_ids":[model_digest]}})).unwrap();
        let config_digest = digest(&config);
        fs::write(blobs.join(config_digest.replace(':', "-")), &config).unwrap();
        let manifest = root.join("manifest.json");
        fs::write(&manifest,serde_json::to_vec(&json!({"schemaVersion":2,"mediaType":"application/vnd.docker.distribution.manifest.v2+json","config":{"mediaType":"application/vnd.docker.container.image.v1+json","digest":config_digest,"size":config.len()},"layers":[{"mediaType":"application/vnd.ollama.image.model","digest":model_digest,"size":model.len()}]})).unwrap()).unwrap();
        let client = root.join("client.py");
        fs::write(&client, include_bytes!("fixtures/native/client.py")).unwrap();
        fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).unwrap();
        let stand_in = DaemonStandIn::spawn();
        let daemon = stand_in.daemon();
        let profile = Profile {
            model: MODEL.into(),
            manifest: pin(&manifest),
            blobs,
            client: pin(&client),
            daemon,
            directory: root.clone(),
        };
        let details = json!({"parent_model":"","format":"gguf","family":"llama","families":["llama"],"parameter_size":"3.2B","quantization_level":"Q4_K_M"});
        let d = profile.manifest.sha256.strip_prefix("sha256:").unwrap();
        let scenario = json!({"model":MODEL,"expect":{"raw":true,"options":{"num_ctx":512,"num_predict":64}},"version":{"version":"0.0.0"},"tags":{"models":[{"name":MODEL,"model":MODEL,"modified_at":"2026-09-21T00:00:00Z","size":100,"digest":d,"details":details}]},"ps":{"models":[{"name":"llama3.2:3b","model":"llama3.2:3b","size":100,"size_vram":0,"expires_at":"2026-09-21T00:01:00Z","context_length":512,"digest":d,"details":details}]},"generated":{"model":MODEL,"created_at":"2026-09-21T00:00:01Z","response":"seven","done":true,"done_reason":"stop","total_duration":20,"load_duration":3,"prompt_eval_count":7,"prompt_eval_duration":4,"eval_count":1,"eval_duration":5}});
        Self {
            root,
            profile,
            scenario,
            _daemon: stand_in,
        }
    }
    fn save(&self) {
        fs::write(
            self.root.join("scenario.json"),
            serde_json::to_vec(&self.scenario).unwrap(),
        )
        .unwrap();
    }
    fn run(&self) -> native::Run<'static> {
        self.save();
        let origin = Instant::now();
        native::execute(
            &full(),
            &self.profile,
            origin,
            origin + Duration::from_secs(20),
            &AtomicBool::new(false),
        )
        .unwrap()
    }
}
impl Drop for Rig {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn settled(run: &native::Run<'_>) {
    for e in &run.exchanges {
        if let Ok(p) = &e.result {
            assert!(
                p.leader_reaped && p.process_group_settled && p.pending.is_none(),
                "{p:?}"
            );
        }
    }
}
#[test]
fn complete_actual_process_contract_keeps_identity_usage_and_raw() {
    let r = Rig::new();
    let run = r.run();
    assert_eq!(run.error, None);
    assert_eq!(run.provider, ProviderState::ObservedComplete);
    assert_eq!(run.contract.terminal(), Some(Terminal::Completed));
    assert_eq!(run.exchanges.len(), 7);
    settled(&run);
    let c = run.contract.candidate().unwrap();
    assert_eq!(c.text, "seven");
    assert_eq!(c.finish, Finish::Stop);
    assert_eq!(
        run.contract.identity().unwrap().provider_model.as_deref(),
        Some("llama3.2:3b")
    );
    let Usage::Reported {
        input,
        output,
        total,
        raw,
        ..
    } = &c.usage
    else {
        panic!("actual usage absent")
    };
    assert_eq!((*input, *output, *total), (Some(7), Some(1), None));
    assert_eq!(raw, &c.raw);
    assert!(r.root.join("captured-request.json").exists());
}
#[test]
fn length_is_truncated_not_success() {
    let mut r = Rig::new();
    r.scenario["generated"]["done_reason"] = json!("length");
    let run = r.run();
    assert_eq!(run.error, None);
    assert_eq!(run.contract.terminal(), Some(Terminal::Truncated));
    settled(&run);
}
#[test]
fn unsupported_requirements_refuse_before_client() {
    for feature in [Feature::Cancel, Feature::Deltas, Feature::ToolProposals] {
        let r = Rig::new();
        r.save();
        let origin = Instant::now();
        let result = native::execute(
            &request(&[Feature::FinalOutput, feature]),
            &r.profile,
            origin,
            origin + Duration::from_secs(10),
            &AtomicBool::new(false),
        );
        assert!(matches!(result,Err(Error::Contract(ContractError::Unsupported(f)))if f==feature));
        assert!(!r.root.join("captured-request.json").exists());
    }
}
#[test]
fn effort_and_foreign_selection_are_not_silently_rewritten() {
    for n in 0..3 {
        let r = Rig::new();
        r.save();
        let mut q = full();
        match n {
            0 => q.selection.effort = Some("high".into()),
            1 => q.selection.provider = "remote".into(),
            _ => q.selection.model = "other".into(),
        }
        let origin = Instant::now();
        assert!(matches!(
            native::execute(
                &q,
                &r.profile,
                origin,
                origin + Duration::from_secs(10),
                &AtomicBool::new(false)
            ),
            Err(Error::Profile)
        ));
        assert!(!r.root.join("captured-request.json").exists());
    }
}
#[test]
fn absent_loaded_identity_cannot_borrow_request_echo() {
    let mut r = Rig::new();
    r.scenario["ps"]["models"] = json!([]);
    let run = r.run();
    assert_eq!(run.error, Some(Error::Identity));
    assert_eq!(run.provider, ProviderState::NotDispatched);
    assert!(run.contract.identity().is_none());
    assert!(!r.root.join("captured-request.json").exists());
    settled(&run);
}
#[test]
fn foreign_loaded_digest_and_wrong_context_refuse() {
    for n in 0..2 {
        let mut r = Rig::new();
        r.scenario["ps"]["models"][0][if n == 0 { "digest" } else { "context_length" }] = if n == 0
        {
            json!("a".repeat(64))
        } else {
            json!(1024)
        };
        let run = r.run();
        assert_eq!(run.error, Some(Error::Identity));
        assert_eq!(run.provider, ProviderState::NotDispatched);
        settled(&run);
    }
}
#[test]
fn duplicate_catalogue_or_loaded_matches_are_ambiguous() {
    for key in ["tags", "ps"] {
        let mut r = Rig::new();
        let row = r.scenario[key]["models"][0].clone();
        r.scenario[key]["models"].as_array_mut().unwrap().push(row);
        let run = r.run();
        assert_eq!(run.error, Some(Error::Identity));
        assert_eq!(run.provider, ProviderState::NotDispatched);
        settled(&run);
    }
}
#[test]
fn packaging_version_change_requires_new_profile() {
    let mut r = Rig::new();
    r.scenario["version"]["version"] = json!("0.12.11");
    let run = r.run();
    assert_eq!(run.error, Some(Error::Identity));
    assert_eq!(run.exchanges.len(), 1);
    settled(&run);
}
#[test]
fn remote_catalogue_metadata_refuses_before_generation() {
    let mut r = Rig::new();
    r.scenario["tags"]["models"][0]["remote_host"] = json!("https://example.invalid");
    let run = r.run();
    assert_eq!(run.error, Some(Error::Json));
    assert_eq!(run.provider, ProviderState::NotDispatched);
    settled(&run);
}
#[test]
fn daemon_incarnation_mismatch_has_no_http_effect() {
    let mut r = Rig::new();
    r.profile.daemon.start_ticks += 1;
    r.save();
    let origin = Instant::now();
    assert!(matches!(
        native::execute(
            &full(),
            &r.profile,
            origin,
            origin + Duration::from_secs(10),
            &AtomicBool::new(false)
        ),
        Err(Error::Identity)
    ));
    assert!(!r.root.join("captured-request.json").exists());
}
#[test]
fn substituted_model_or_client_bytes_refuse() {
    for model in [true, false] {
        let r = Rig::new();
        let path = if model {
            r.profile.manifest.path.clone()
        } else {
            r.profile.client.path.clone()
        };
        fs::write(path, b"substituted").unwrap();
        r.save();
        let origin = Instant::now();
        assert!(matches!(
            native::execute(
                &full(),
                &r.profile,
                origin,
                origin + Duration::from_secs(10),
                &AtomicBool::new(false)
            ),
            Err(Error::Subject)
        ));
    }
}
#[test]
fn loaded_identity_drift_after_response_retains_unknown() {
    let mut r = Rig::new();
    r.scenario["post_ps"] = json!({"models":[]});
    let run = r.run();
    assert_eq!(run.error, Some(Error::Identity));
    assert_eq!(run.provider, ProviderState::Unknown);
    assert!(run.contract.candidate().is_none());
    assert_eq!(run.exchanges.len(), 7);
    assert!(
        !run.exchanges[3]
            .result
            .as_ref()
            .unwrap()
            .stdout
            .bytes
            .is_empty()
    );
    settled(&run);
}
#[test]
fn closed_json_duplicate_unknown_and_trailing_records_refuse() {
    for raw in ["{\"model\":\"x\",\"model\":\"y\"}", "{} {}", "not-json"] {
        let mut r = Rig::new();
        r.scenario["raw"] = json!(raw);
        let run = r.run();
        assert_eq!(run.error, Some(Error::Json));
        assert_eq!(run.provider, ProviderState::Unknown);
        settled(&run);
    }
    let mut r = Rig::new();
    r.scenario["generated"]["unexpected"] = json!(true);
    assert_eq!(r.run().error, Some(Error::Json));
}
#[test]
fn missing_negative_and_overflow_usage_never_becomes_zero() {
    for raw in ["missing", "-1", "18446744073709551616"] {
        let mut r = Rig::new();
        if raw == "missing" {
            r.scenario["generated"]
                .as_object_mut()
                .unwrap()
                .remove("eval_count");
        } else {
            let text = serde_json::to_string(&r.scenario["generated"])
                .unwrap()
                .replace("\"eval_count\":1", &format!("\"eval_count\":{raw}"));
            r.scenario["raw"] = json!(text);
        }
        let run = r.run();
        assert_eq!(run.error, Some(Error::Json));
        assert_eq!(run.provider, ProviderState::Unknown);
        settled(&run);
    }
}
#[test]
fn zero_usage_remains_explicit_zero() {
    let mut r = Rig::new();
    r.scenario["generated"]["eval_count"] = json!(0);
    let run = r.run();
    assert_eq!(run.error, None);
    let Usage::Reported { output, .. } = run.contract.usage() else {
        panic!()
    };
    assert_eq!(*output, Some(0));
}
#[test]
fn input_and_output_cap_violations_refuse_even_done() {
    for key in ["prompt_eval_count", "eval_count"] {
        let mut r = Rig::new();
        r.scenario["generated"][key] = json!(if key == "eval_count" { 65 } else { 513 });
        let run = r.run();
        assert_eq!(run.error, Some(Error::Usage));
        assert_eq!(run.provider, ProviderState::Unknown);
        settled(&run);
    }
}
#[test]
fn prose_success_cannot_replace_done_or_reason() {
    for n in 0..3 {
        let mut r = Rig::new();
        r.scenario["generated"]["response"] = json!("SUCCESS accepted task");
        match n {
            0 => r.scenario["generated"]["done"] = json!(false),
            1 => r.scenario["generated"]["done_reason"] = json!("error"),
            _ => r.scenario["generated"]["model"] = json!("other"),
        }
        let run = r.run();
        assert_eq!(run.error, Some(Error::Response));
        assert!(run.contract.terminal().is_none());
        settled(&run);
    }
}
#[test]
fn failed_client_and_stderr_are_not_successful_json() {
    for key in ["exit", "stderr"] {
        let mut r = Rig::new();
        r.scenario[key] = if key == "exit" { json!(7) } else { json!(true) };
        let run = r.run();
        assert_eq!(run.error, Some(Error::Process));
        assert_eq!(run.provider, ProviderState::Unknown);
        settled(&run);
    }
}
#[test]
fn oversized_process_output_preserves_raw_failure() {
    let mut r = Rig::new();
    r.scenario["large"] = json!(true);
    let run = r.run();
    assert_eq!(run.error, Some(Error::Process));
    assert_eq!(run.provider, ProviderState::Unknown);
    assert!(run.exchanges[3].result.as_ref().unwrap().stdout.truncated);
    settled(&run);
}
#[test]
fn cancelled_before_dispatch_has_no_client_or_candidate() {
    let r = Rig::new();
    r.save();
    let origin = Instant::now();
    assert!(matches!(
        native::execute(
            &full(),
            &r.profile,
            origin,
            origin + Duration::from_secs(10),
            &AtomicBool::new(true)
        ),
        Err(Error::Cancelled)
    ));
    assert!(!r.root.join("captured-request.json").exists());
}
#[test]
fn cancellation_during_generation_is_unsupported_and_unknown_not_settled() {
    let mut r = Rig::new();
    r.scenario["pause"] = json!(true);
    r.save();
    let flag = Arc::new(AtomicBool::new(false));
    let worker = flag.clone();
    let path = r.root.join("generation-started");
    let thread = std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(5);
        while !path.exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(path.exists());
        worker.store(true, Ordering::Release);
    });
    let origin = Instant::now();
    let run = native::execute(
        &full(),
        &r.profile,
        origin,
        origin + Duration::from_secs(15),
        &flag,
    )
    .unwrap();
    thread.join().unwrap();
    assert_eq!(run.provider, ProviderState::Unknown);
    assert_eq!(run.error, Some(Error::Process));
    assert_eq!(run.contract.phase(), Phase::Unknown);
    assert!(run.contract.cancellation().unsupported());
    assert!(!run.contract.cancellation().acknowledged);
    assert!(run.contract.terminal().is_none());
    settled(&run);
}
/// WK-04 custody: while a control's client forks a descendant, this test process is the
/// descendant's subreaper, so an orphan is reaped here by pid rather than adopted by the
/// quality supervisor (whose `descendants_detected` would otherwise refuse the battery).
struct Subreaper;
impl Subreaper {
    fn claim() -> Result<Self, Box<dyn std::error::Error>> {
        use rustix::process::{child_subreaper, getpid, set_child_subreaper};
        set_child_subreaper(Some(getpid()))?;
        // A successful prctl is not a changed state: read it back.
        let guard = Self;
        if child_subreaper()?.is_none() {
            return Err("PR_SET_CHILD_SUBREAPER did not take effect".into());
        }
        Ok(guard)
    }
}
impl Drop for Subreaper {
    fn drop(&mut self) {
        let _ = rustix::process::set_child_subreaper(None);
    }
}
fn descendant(root: &Path) -> Result<rustix::process::Pid, Box<dyn std::error::Error>> {
    let raw: i32 = fs::read_to_string(root.join("descendant.pid"))?
        .trim()
        .parse()?;
    rustix::process::Pid::from_raw(raw).ok_or_else(|| "descendant pid is not positive".into())
}
/// Reap an adopted descendant by pid within a hang-guard budget; returns its wait status.
fn reap(
    pid: rustix::process::Pid,
) -> Result<rustix::process::WaitStatus, Box<dyn std::error::Error>> {
    use rustix::process::{WaitOptions, waitpid};
    let budget = Duration::from_secs(15);
    let start = Instant::now();
    loop {
        if let Some((_, status)) = waitpid(Some(pid), WaitOptions::NOHANG)? {
            return Ok(status);
        }
        if start.elapsed() >= budget {
            return Err(format!(
                "descendant {} not reaped within {:?} (elapsed {:?})",
                pid.as_raw_nonzero(),
                budget,
                start.elapsed()
            )
            .into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn surviving_writable_descendant_refuses_a_valid_frame_and_stays_unknown()
-> Result<(), Box<dyn std::error::Error>> {
    use habitat_engine::worker::process::Interruption;
    let _custody = Subreaper::claim()?;
    let mut r = Rig::new();
    r.scenario["descendant"] = json!("survive");
    let run = r.run();
    // Reap before any assertion so a red control cannot leak an orphan to the supervisor.
    let status = reap(descendant(&r.root)?)?;
    assert!(
        status.signaled(),
        "the descendant must be ended by the owner's group cleanup, not exit on its own: {status:?}"
    );
    assert_eq!(run.error, Some(Error::Process));
    assert_eq!(run.provider, ProviderState::Unknown);
    assert_eq!(run.contract.phase(), Phase::Unknown);
    assert!(run.contract.terminal().is_none());
    assert!(run.contract.candidate().is_none());
    assert_eq!(
        run.exchanges.len(),
        4,
        "no identity readback after the refusal"
    );
    let generate = &run.exchanges[3];
    assert_eq!(generate.operation, "generate");
    let report = generate.result.as_ref().map_err(|e| format!("{e:?}"))?;
    // The intended diagnostic: the residual group, not the leader's own exit or its frame.
    assert_eq!(report.interruption, Some(Interruption::ResidualGroup));
    assert_eq!(report.exit_code, Some(0));
    assert!(report.stderr.bytes.is_empty() && !report.stdout.truncated);
    let frame: Value = serde_json::from_slice(&report.stdout.bytes)?;
    assert_eq!(
        frame, r.scenario["generated"],
        "the leader's complete, valid frame is retained raw but never accepted"
    );
    settled(&run);
    Ok(())
}
#[test]
fn descendant_exiting_before_leader_keeps_completion_and_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    let _custody = Subreaper::claim()?;
    let mut r = Rig::new();
    r.scenario["descendant"] = json!("exit");
    let run = r.run();
    // The benign control must actually have forked, or it proves nothing about the fault one.
    let pid = descendant(&r.root)?;
    assert!(
        matches!(
            rustix::process::waitpid(Some(pid), rustix::process::WaitOptions::NOHANG),
            Err(rustix::io::Errno::CHILD)
        ),
        "the leader reaps its own descendant; nothing is orphaned to this process"
    );
    assert_eq!(run.error, None);
    assert_eq!(run.provider, ProviderState::ObservedComplete);
    assert_eq!(run.contract.terminal(), Some(Terminal::Completed));
    assert_eq!(run.exchanges.len(), 7);
    let report = run.exchanges[3]
        .result
        .as_ref()
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!(report.interruption, None);
    settled(&run);
    let candidate = run.contract.candidate().ok_or("candidate absent")?;
    assert_eq!(candidate.text, "seven");
    assert_eq!(
        candidate
            .identity
            .as_ref()
            .and_then(|i| i.provider_model.as_deref()),
        Some("llama3.2:3b")
    );
    assert_eq!(
        candidate
            .identity
            .as_ref()
            .and_then(|i| i.provider_revision.as_deref()),
        Some(r.profile.manifest.sha256.as_str())
    );
    Ok(())
}

/// R21 N7 · the resolver's candidate set is `MainPID` and its descendants: `descendants` follows
/// parent links through a `/proc` census, breadth first, and refuses a set larger than its bound
/// naming both numbers. Over DS18's measured chain (`1901` toolbox → `2668` podman exec → `2693`
/// ollama, `DS18-daemon-20260927.json`) with two unrelated processes it returns the two below
/// 1901; a cycle a racing census could show (pid reuse) ends the walk rather than looping. The
/// census itself is read live and checked against the kernel's own `getppid` for this process.
#[test]
fn descendants_follow_ppid_links_and_refuse_past_their_bound_with_both_numbers()
-> Result<(), Box<dyn std::error::Error>> {
    use habitat_engine::worker::process::{
        CensusError, DescendantBound, Stat, census, descendants,
    };
    let stat = |ppid: u32| Stat {
        state: 'S',
        ppid,
        pgrp: 1,
        start_ticks: 3_119,
    };
    let measured = vec![
        (1, stat(0)),
        (1901, stat(1)),
        (2668, stat(1901)),
        (2693, stat(2668)),
        (2700, stat(1)),
        (2701, stat(2700)),
    ];
    assert_eq!(descendants(&measured, 1901, 64), Ok(vec![2668, 2693]));
    assert_eq!(
        descendants(&measured, 1901, 2),
        Ok(vec![2668, 2693]),
        "a set exactly at its bound is admitted"
    );
    assert_eq!(
        descendants(&measured, 1901, 1),
        Err(DescendantBound { found: 2, limit: 1 })
    );
    assert_eq!(
        descendants(&measured, 1, 3),
        Err(DescendantBound { found: 5, limit: 3 })
    );
    assert_eq!(descendants(&measured, 2693, 64), Ok(vec![]));
    let cycle = vec![(5, stat(6)), (6, stat(5))];
    assert_eq!(descendants(&cycle, 5, 64), Ok(vec![6]));
    let running = AtomicBool::new(false);
    let live = census(Instant::now() + Duration::from_secs(10), &running)
        .map_err(|error| format!("the live census: {error:?}"))?;
    let me = std::process::id();
    assert_eq!(
        live.iter()
            .find(|(pid, _)| *pid == me)
            .map(|(_, stat)| stat.ppid),
        Some(std::os::unix::process::parent_id())
    );
    assert_eq!(
        census(
            Instant::now() + Duration::from_secs(10),
            &AtomicBool::new(true)
        ),
        Err(CensusError::Cancelled)
    );
    assert_eq!(census(Instant::now(), &running), Err(CensusError::Deadline));
    Ok(())
}

/// R21 N6 · the resolver reads the unit's `MainPID` over the one pinned busctl door; its reply is
/// one `u` and zero means no process. Independent source: the host's own replies, recorded
/// read-only in `~/hee3-evidence/T00-plan-20260926/DS18-busctl-20260927.json` (2026-09-27):
/// `busctl --user --json=short get-property … ollama_2eservice … MainPID` printed the first fixture,
/// and the same property on an unloaded unit's path printed the second with rc=0 — "0 = no
/// process" is systemd's own answer, not an error it raises.
#[test]
fn a_main_pid_reply_is_one_u32_and_zero_is_no_process() {
    use habitat_engine::worker::aggregate::{Error as ManagerError, main_pid_reply};
    assert_eq!(main_pid_reply(br#"{"type":"u","data":1901}"#), Ok(1901));
    assert_eq!(
        main_pid_reply(br#"{"type":"u","data":0}"#),
        Err(ManagerError::Manager),
        "an unloaded unit's MainPID is 0: no process to resolve"
    );
    assert_eq!(
        main_pid_reply(br#"{"type":"s","data":"1901"}"#),
        Err(ManagerError::Manager)
    );
    assert_eq!(
        main_pid_reply(br#"{"type":"u","data":1901,"extra":1}"#),
        Err(ManagerError::Manager)
    );
    assert_eq!(
        main_pid_reply(br#"{"type":"u","data":4294967296}"#),
        Err(ManagerError::Manager)
    );
}

/// R21 closure C6 (M3, F2, FT2-09) · the user manager's refusal of `MainPID` keeps its own name:
/// the window's deadline and cancellation are the adapter's, every other kind is carried whole, so a
/// busctl pin refusal, an absent unit and a process-less one are never all `Identity`. The table
/// is every kind the manager has, and it is asserted whole.
#[test]
fn every_manager_refusal_of_main_pid_keeps_its_name() {
    use habitat_engine::worker::aggregate::Error as ManagerError;
    let kinds = [
        ManagerError::Invalid,
        ManagerError::Bound,
        ManagerError::Deadline,
        ManagerError::Cancelled,
        ManagerError::Identity,
        ManagerError::Io,
        ManagerError::State,
        ManagerError::Manager,
        ManagerError::Process,
        ManagerError::Limits,
        ManagerError::Busy,
    ];
    let said: Vec<_> = kinds
        .iter()
        .map(|kind| (*kind, native::daemon_refusal(*kind)))
        .collect();
    let expected = vec![
        (ManagerError::Invalid, Error::Manager(ManagerError::Invalid)),
        (ManagerError::Bound, Error::Manager(ManagerError::Bound)),
        (ManagerError::Deadline, Error::Deadline),
        (ManagerError::Cancelled, Error::Cancelled),
        (
            ManagerError::Identity,
            Error::Manager(ManagerError::Identity),
        ),
        (ManagerError::Io, Error::Manager(ManagerError::Io)),
        (ManagerError::State, Error::Manager(ManagerError::State)),
        (ManagerError::Manager, Error::Manager(ManagerError::Manager)),
        (ManagerError::Process, Error::Manager(ManagerError::Process)),
        (ManagerError::Limits, Error::Manager(ManagerError::Limits)),
        (ManagerError::Busy, Error::Manager(ManagerError::Busy)),
    ];
    assert_eq!(said, expected);
    assert_eq!(Error::Manager(ManagerError::State).name(), "manager");
}

/// The `MainPid` seam's double (F101): it records every unit, deadline and cancellation reading it
/// was handed, and answers one scripted pid or refusal.
struct MainPidDouble {
    answer: Result<u32, Error>,
    asked: Vec<(String, Instant, bool)>,
}
impl native::MainPid for MainPidDouble {
    fn main_pid(
        &mut self,
        unit: &str,
        deadline: Instant,
        cancelled: &AtomicBool,
    ) -> Result<u32, Error> {
        self.asked
            .push((unit.to_owned(), deadline, cancelled.load(Ordering::Acquire)));
        self.answer
    }

    fn walks_descendants(&self) -> bool {
        true
    }
}

/// A `/usr/bin/sh` whose one child is a `/usr/bin/sleep`: `MainPID` is the shell, the pinned
/// executable is its descendant's — the host's own shape (DS18: `ollama.service`'s `MainPID` 1901
/// is `/usr/bin/toolbox`; the daemon, 2693, is two links below it). The sleep is killed by its pid
/// and reaped by the shell's `wait`, so nothing is orphaned; the group is killed only as a fallback.
struct ShellWithChild {
    shell: Child,
    child: Option<u32>,
}
impl ShellWithChild {
    fn spawn() -> Result<Self, Box<dyn std::error::Error>> {
        use std::os::unix::process::CommandExt;
        let shell = Command::new("/usr/bin/sh")
            .args(["-c", "/usr/bin/sleep 600 & wait"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn()?;
        let mut guard = Self { shell, child: None };
        let budget = Duration::from_secs(5);
        let start = Instant::now();
        let running = AtomicBool::new(false);
        while guard.child.is_none() {
            let census = habitat_engine::worker::process::census(start + budget, &running)
                .map_err(|e| format!("census: {e:?}"))?;
            guard.child = census
                .iter()
                .find(|(_, stat)| stat.ppid == guard.shell.id())
                .map(|(pid, _)| *pid);
            if guard.child.is_none() && start.elapsed() >= budget {
                return Err(format!(
                    "the shell {} forked no child within {budget:?} (elapsed {:?})",
                    guard.shell.id(),
                    start.elapsed()
                )
                .into());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(guard)
    }
}
impl Drop for ShellWithChild {
    fn drop(&mut self) {
        use rustix::process::{Pid, Signal, kill_process, kill_process_group};
        if let Some(pid) = self.child.and_then(|pid| Pid::from_raw(pid.cast_signed())) {
            let _ = kill_process(pid, Signal::KILL);
        } else if let Some(group) = Pid::from_raw(self.shell.id().cast_signed()) {
            let _ = kill_process_group(group, Signal::KILL);
        }
        let _ = self.shell.wait();
    }
}

/// The resolved daemon, compared field by field (`Daemon` has no `PartialEq`).
fn same_daemon(got: &Daemon, want: &Daemon) -> bool {
    (
        got.pid,
        got.start_ticks,
        got.boot_id.as_str(),
        got.executable_sha256.as_str(),
        got.executable_bytes,
    ) == (
        want.pid,
        want.start_ticks,
        want.boot_id.as_str(),
        want.executable_sha256.as_str(),
        want.executable_bytes,
    )
}

/// `MainPID` is a shell; the pinned executable is its child, resolved through the descendant walk.
fn resolves_below_a_shell(
    pin: &native::DaemonPin,
    expected: &Daemon,
) -> Result<(), Box<dyn std::error::Error>> {
    let shell = ShellWithChild::spawn()?;
    let child = shell.child.ok_or("no child")?;
    let later = Instant::now() + Duration::from_secs(40);
    let mut seam = MainPidDouble {
        answer: Ok(shell.shell.id()),
        asked: vec![],
    };
    let other = native::DaemonPin {
        unit: "hee3-t08-shell.service".into(),
        ..pin.clone()
    };
    let below = native::resolve(&other, &mut seam, later, &AtomicBool::new(false))
        .map_err(|e| format!("{e:?}"))?;
    let stat = fs::read_to_string(format!("/proc/{child}/stat"))?;
    let ticks: u64 = stat
        .rsplit_once(')')
        .ok_or("stat")?
        .1
        .split_whitespace()
        .nth(19)
        .ok_or("field 22")?
        .parse()?;
    assert!(
        same_daemon(
            &below,
            &Daemon {
                pid: child,
                start_ticks: ticks,
                ..expected.clone()
            }
        ),
        "{below:?}"
    );
    assert_ne!(child, expected.pid);
    assert_eq!(
        seam.asked,
        vec![("hee3-t08-shell.service".to_owned(), later, false)]
    );
    Ok(())
}

/// R21 N6, N7 · the resolver: candidates are `MainPID` and its descendants, each `/proc/<pid>/exe`
/// hashed against the pin, and exactly one must match. Over the stand-in (`MainPID` is the pinned
/// process itself) and over a shell whose descendant is the pinned executable (the host's shape),
/// the resolved daemon equals what the stand-in reader (`DaemonStandIn::daemon`, a separate reader of
/// `/proc`) reports, field by field. The seam's double saw the unit, the deadline and the
/// cancellation the caller passed. A pin nothing matches is `Matches` with none of the one
/// candidate matched (R21 round-1 LOW L7: the stand-in `sleep` has no descendants); a pin that is
/// not a pin is `Profile` before the seam is asked; the seam's refusal passes through. The pure
/// selection is `the_daemon_selection_refuses_with_how_many_matched_of_how_many`'s.
#[test]
fn the_resolver_selects_the_one_candidate_whose_executable_is_the_pin()
-> Result<(), Box<dyn std::error::Error>> {
    use native::{DaemonPin, resolve};
    let stand_in = DaemonStandIn::spawn();
    let expected = stand_in.daemon();
    let pin = DaemonPin {
        unit: "hee3-t08-stand-in.service".into(),
        executable_sha256: expected.executable_sha256.clone(),
        executable_bytes: expected.executable_bytes,
    };
    let running = AtomicBool::new(false);
    let deadline = Instant::now() + Duration::from_secs(30);
    let asked_once = vec![("hee3-t08-stand-in.service".to_owned(), deadline, false)];
    // MainPID is the pinned process.
    let mut seam = MainPidDouble {
        answer: Ok(expected.pid),
        asked: vec![],
    };
    let resolved = resolve(&pin, &mut seam, deadline, &running).map_err(|e| format!("{e:?}"))?;
    assert!(
        same_daemon(&resolved, &expected),
        "{resolved:?} != {expected:?}"
    );
    assert_eq!(seam.asked, asked_once);
    resolves_below_a_shell(&pin, &expected)?;
    // A pin no candidate's executable matches: zero matches, of the one candidate.
    let mut seam = MainPidDouble {
        answer: Ok(expected.pid),
        asked: vec![],
    };
    let foreign = DaemonPin {
        executable_sha256: format!("sha256:{}", "0".repeat(64)),
        ..pin.clone()
    };
    assert!(matches!(
        resolve(&foreign, &mut seam, deadline, &running),
        Err(Error::Matches(native::DaemonMatches {
            matched: 0,
            candidates: 1
        }))
    ));
    // A pin that is not a pin is refused before the seam is asked.
    for broken in [
        DaemonPin {
            executable_bytes: 0,
            ..pin.clone()
        },
        DaemonPin {
            executable_sha256: "12ff8654".into(),
            ..pin.clone()
        },
    ] {
        assert!(matches!(
            resolve(&broken, &mut seam, deadline, &running),
            Err(Error::Profile)
        ));
    }
    assert_eq!(seam.asked, asked_once, "only the foreign pin asked");
    // A raised flag stops before the seam; the seam's own refusal passes through.
    let mut seam = MainPidDouble {
        answer: Err(Error::Identity),
        asked: vec![],
    };
    assert!(matches!(
        resolve(&pin, &mut seam, deadline, &AtomicBool::new(true)),
        Err(Error::Cancelled)
    ));
    assert_eq!(seam.asked, vec![]);
    assert!(matches!(
        resolve(&pin, &mut seam, deadline, &running),
        Err(Error::Identity)
    ));
    assert_eq!(seam.asked, asked_once);
    Ok(())
}

/// R21 round-1 LOW L7 · the selection's refusal carries its count: exactly one candidate in the
/// matched set is selected; none and several are refused `Matches`, with how many matched of how
/// many candidates — fixtures that differ in both numbers — and a matched pid outside the
/// candidates is neither selected nor counted.
#[test]
fn the_daemon_selection_refuses_with_how_many_matched_of_how_many() {
    use native::{DaemonMatches, select_daemon};
    use std::collections::BTreeSet;
    let refused = |matched, candidates| {
        Err(Error::Matches(DaemonMatches {
            matched,
            candidates,
        }))
    };
    assert_eq!(select_daemon(&[1, 2], &BTreeSet::new()), refused(0, 2));
    assert_eq!(
        select_daemon(&[4, 5, 6], &BTreeSet::from([4, 6])),
        refused(2, 3)
    );
    assert_eq!(select_daemon(&[1, 2], &BTreeSet::from([2])), Ok(2));
    assert_eq!(select_daemon(&[7, 8, 9], &BTreeSet::from([3, 9])), Ok(9));
    assert_eq!(
        select_daemon(&[1, 2, 5, 6], &BTreeSet::from([3])),
        refused(0, 4)
    );
}

// ---- N6b (2026-09-30; Luke: "resolve by endpoint"): the daemon is the one process holding the one
// listener at the endpoint. The table fixture is the host's own /proc/net/tcp, recorded 2026-09-30
// (the world produced it): one LISTEN at 127.0.0.1:11434, uid 1000, inode 34847.

const HOST_TCP: &str = include_str!("fixtures/native/proc-net-tcp-host-20260930.txt");
const HEADER: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n";

fn row(local: &str, state: &str, uid: u32, inode: u64) -> String {
    format!(
        "   9: {local} 00000000:0000 {state} 00000000:00000000 00:00000000 00000000  {uid}        0 {inode} 1 0000000000000000 100 0 0 10 0\n"
    )
}

#[test]
fn the_endpoint_listener_is_read_from_the_hosts_own_table() {
    assert_eq!(native::listener(&[HOST_TCP, ""], 11434, 1000), Ok(34847));
    assert_eq!(native::ENDPOINT_PORT, 11434);
    assert!(
        native::endpoint().contains(":11434/"),
        "the port constant and the endpoint URL name one port"
    );
}

#[test]
fn every_listener_refusal_is_named_by_its_own_reason() {
    use native::EndpointWhy as Why;
    let one = format!("{HEADER}{}", row("0100007F:2CAA", "0A", 1000, 7));
    let not_listening = format!(
        "{HEADER}{}{}",
        row("0100007F:2CAA", "01", 1000, 8),
        row("0100007F:2CAB", "0A", 1000, 9)
    );
    let two = format!("{one}{}", row("00000000:2CAA", "0A", 1000, 10));
    let any6 = format!(
        "{HEADER}{}",
        row("00000000000000000000000000000000:2CAA", "0A", 1000, 11)
    );
    let mapped6 = format!(
        "{HEADER}{}",
        row("0000000000000000FFFF00000100007F:2CAA", "0A", 1000, 12)
    );
    let loopback6 = format!(
        "{HEADER}{}",
        row("00000000000000000000000001000000:2CAA", "0A", 1000, 13)
    );
    let other = format!("{HEADER}{}", row("0100007F:2CAA", "0A", 1001, 14));
    let broken = format!("{HEADER}   9: 0100007F:2CAA 00000000:0000 0A\n");
    assert_eq!(
        [
            native::listener(&[&one, ""], 11434, 1000),
            native::listener(&[&not_listening, ""], 11434, 1000),
            native::listener(&[&two, ""], 11434, 1000),
            native::listener(&[&one, &any6], 11434, 1000),
            native::listener(&["", &mapped6], 11434, 1000),
            native::listener(&["", &loopback6], 11434, 1000),
            native::listener(&[&other, ""], 11434, 1000),
            native::listener(&[&broken, ""], 11434, 1000),
            native::listener(&[HOST_TCP, ""], 11435, 1000),
        ],
        [
            Ok(7),
            Err(Why::NoListener),
            Err(Why::Listeners(2)),
            Err(Why::Listeners(2)),
            Ok(12),
            Err(Why::NoListener),
            Err(Why::OtherUid(1001)),
            Err(Why::Table),
            Err(Why::NoListener),
        ]
    );
}

#[test]
fn the_holder_is_one_readable_process_or_refused_by_name() {
    use native::EndpointWhy as Why;
    let held = |pid: u32, links: &[&str]| {
        (
            pid,
            links
                .iter()
                .map(|link| (*link).to_owned())
                .collect::<Vec<_>>(),
        )
    };
    let one = [
        held(40, &["/dev/null", "socket:[77]"]),
        held(41, &["socket:[78]", "socket:[770]"]),
    ];
    let two = [
        held(40, &["socket:[77]"]),
        held(52, &["pipe:[3]", "socket:[77]"]),
    ];
    assert_eq!(
        [
            native::holder(&one, 77, 3),
            native::holder(&one, 7, 5),
            native::holder(&two, 77, 0),
        ],
        [
            Ok(40),
            Err(Why::Unobserved { unreadable: 5 }),
            Err(Why::Owners(2))
        ]
    );
}

#[test]
fn an_endpoint_source_names_the_daemon_and_walks_nothing_below_it() {
    assert!(!native::MainPid::walks_descendants(&native::Endpoint {
        port: 1
    }));
    assert_eq!(
        (
            native::daemon_candidates(7, Vec::new()),
            native::daemon_candidates(9, vec![11, 12])
        ),
        (vec![7], vec![9, 11, 12])
    );
}

#[test]
fn a_real_listener_is_resolved_to_this_process_and_a_closed_port_to_none()
-> Result<(), Box<dyn std::error::Error>> {
    let open = std::net::TcpListener::bind("127.0.0.1:0")?;
    let port = open.local_addr()?.port();
    let found = native::endpoint_holder(
        port,
        Instant::now() + Duration::from_secs(60),
        &AtomicBool::new(false),
    );
    // Port 0 is never a listening port: the "nothing listens" case without a bind-and-release race.
    let none = native::endpoint_holder(
        0,
        Instant::now() + Duration::from_secs(60),
        &AtomicBool::new(false),
    );
    assert_eq!(
        (found, none),
        (
            Ok(std::process::id()),
            Err(Error::Endpoint(native::EndpointWhy::NoListener))
        )
    );
    drop(open);
    Ok(())
}

#[test]
fn an_ipv6_any_listener_is_the_endpoints_and_an_ipv6_loopback_is_not()
-> Result<(), Box<dyn std::error::Error>> {
    // The kernel's own spellings (review 2c M2): `[::]` accepts 127.0.0.1 on a dual-stack socket and is
    // this process; `[::1]` does not accept an IPv4 connection, so nothing listens for the endpoint.
    let any = std::net::TcpListener::bind("[::]:0")?;
    let loopback = std::net::TcpListener::bind("[::1]:0")?;
    let (any_port, loopback_port) = (any.local_addr()?.port(), loopback.local_addr()?.port());
    let at = |port| {
        native::endpoint_holder(
            port,
            Instant::now() + Duration::from_secs(60),
            &AtomicBool::new(false),
        )
    };
    assert_eq!(
        (at(any_port), at(loopback_port)),
        (
            Ok(std::process::id()),
            Err(Error::Endpoint(native::EndpointWhy::NoListener))
        )
    );
    Ok(())
}

#[test]
fn the_descriptor_bound_and_the_namespace_are_decided_by_argument() {
    use native::EndpointWhy as Why;
    use std::path::Path;
    assert_eq!(
        (
            native::within_descriptor_bound(7, 7),
            native::within_descriptor_bound(8, 7),
            native::same_namespace(Path::new("net:[4026531840]"), Path::new("net:[4026531840]")),
            native::same_namespace(Path::new("net:[4026531840]"), Path::new("net:[4026532211]")),
        ),
        (Ok(()), Err(Why::Bound), Ok(()), Err(Why::Namespace))
    );
}
