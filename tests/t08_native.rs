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
        let scenario = json!({"model":MODEL,"version":{"version":"0.0.0"},"tags":{"models":[{"name":MODEL,"model":MODEL,"modified_at":"2026-09-21T00:00:00Z","size":100,"digest":d,"details":details}]},"ps":{"models":[{"name":"llama3.2:3b","model":"llama3.2:3b","size":100,"size_vram":0,"expires_at":"2026-09-21T00:01:00Z","context_length":512,"digest":d,"details":details}]},"generated":{"model":MODEL,"created_at":"2026-09-21T00:00:01Z","response":"seven","done":true,"done_reason":"stop","total_duration":20,"load_duration":3,"prompt_eval_count":7,"prompt_eval_duration":4,"eval_count":1,"eval_duration":5}});
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
