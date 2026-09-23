//! T08 contract conformance battery: `native::execute` driven through `worker::Contract`
//! exactly as dispatch drives it, against the offline fake client
//! (`fixtures/native/contract-client.py`). Every oracle reads the typed envelope
//! (`Run`, `Contract`, `ProcessReport`), never prose. Case ids `T08C-NN` are stable and
//! one per obligation; the obligation groups are the assignment's:
//!   A contract returns (T08C-01..06) · B error families (T08C-07..11) ·
//!   C cancellation (T08C-12..14) · D pre-effect refusals (T08C-15..19) ·
//!   E model/effort recording (T08C-20..23).
//! Synthetic model/runtime observations are never backend qualification.
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::worker::native::{self, Daemon, Error, FilePin, Profile, ProviderState};
use habitat_engine::worker::process::{Interruption, exited_during_census};
use habitat_engine::worker::{
    self, CancelDispatch, CancelReason, Cancellation, Candidate, Capabilities, ContractError,
    Envelope, Event, Feature, Finish, Identity, IdentityOrigin, Phase, Request, Selection,
    Terminal, Usage, UsageForm, UsageScope, UsageStage,
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
const MODEL: &str = "hee3-t08-contract:qualification";
/// The loaded row deliberately carries a name distinct from the request so a
/// `provider_model` equal to it proves readback, not request echo.
const LOADED: &str = "hee3-t08-contract-loaded:qualification";
const PROMPT: &str = "Return exactly seven.";
const INPUT_TOKENS: u64 = 41;
const OUTPUT_TOKENS: u64 = 3;
/// Pinned llama3.2:3b manifest digest observed by both actual cohorts
/// (`tests/fixtures/native/actual/*-observation.json`).
const PINNED_MODEL_DIGEST: &str =
    "sha256:a80c4f17acd55265feec403c7aef86be0c25983ab279d83f3bcd3abbcb5b8b72";
const PINNED_RUNTIME_INSTANCE: &str = "04f2b5b0-f5ad-4765-aaf0-0584759ceac7:2776062:56361454";
/// Overbound prompt recipe: sha256 of `fixtures/native/prompt.txt` (135168 bytes).
const OVERBOUND_RECIPE: &str =
    "sha256:46f9ab6eac6c2d1149a4eeb47bf146ce57fd7a9cff889686a7ceb4525f8161fa";
const OVERBOUND_PROMPT: &str = include_str!("fixtures/native/prompt.txt");
/// Pins quoted by independent records: `root-actual-readback.json` (positive) and the
/// overbound actual review (negative).
const POSITIVE_OBSERVATION_SHA256: &str =
    "ac7442748e24ebe8a0dfa25ab7d96dccfe6ccd35163cc185a647e6db85d08281";
const OVERBOUND_OBSERVATION_SHA256: &str =
    "bfc9699fb29da4863a45f40f2276f6dc085af285604a1af6d70a6e3c95dd2eaa";
const POSITIVE_OBSERVATION: &[u8] =
    include_bytes!("fixtures/native/actual/positive-observation.json");
const OVERBOUND_OBSERVATION: &[u8] =
    include_bytes!("fixtures/native/actual/overbound-observation.json");
const OVERBOUND_STDOUT: &[u8] = include_bytes!("fixtures/native/actual/overbound-03-stdout");
const OVERBOUND_STDERR: &[u8] = include_bytes!("fixtures/native/actual/overbound-03-stderr");

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
/// The fake client prints `json.dumps(value, separators=(',', ':'))` plus LF over a
/// sorted-key object; `serde_json` compact rendering is the independent reference.
fn rendered(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}
fn request(required: &[Feature], prompt: &str) -> Request<'static> {
    Request {
        invocation: worker::Invocation {
            binding: worker::Binding {
                task: UuidV4::parse("08c00000-0000-4000-8000-000000000011").unwrap(),
                attempt: UuidV4::parse("08c00000-0000-4000-8000-000000000012").unwrap(),
                generation: "2".parse().unwrap(),
            },
            id: UuidV4::parse("08c00000-0000-4000-8000-000000000013").unwrap(),
        },
        recipe: Sha256Digest::parse(
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .unwrap(),
        workspace: Sha256Digest::parse(
            "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        )
        .unwrap(),
        adapter_profile: native::PROFILE.into(),
        selection: Selection {
            provider: native::PROVIDER.into(),
            model: MODEL.into(),
            effort: None,
        },
        required: Capabilities::new(required),
        prompt: prompt.into(),
    }
}
fn full() -> Request<'static> {
    request(
        &[Feature::FinalOutput, Feature::Identity, Feature::Usage],
        PROMPT,
    )
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
    origin: Instant,
    _daemon: DaemonStandIn,
}
impl Rig {
    fn new() -> Self {
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "t08c-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let blobs = root.join("blobs");
        fs::DirBuilder::new().mode(0o700).create(&blobs).unwrap();
        let model = b"not a real model; finite offline contract fixture";
        let model_digest = digest(model);
        fs::write(blobs.join(model_digest.replace(':', "-")), model).unwrap();
        let config=serde_json::to_vec(&json!({"model_format":"gguf","model_family":"llama","model_families":["llama"],"model_type":"3.2B","file_type":"Q4_K_M","architecture":"amd64","os":"linux","rootfs":{"type":"layers","diff_ids":[model_digest]}})).unwrap();
        let config_digest = digest(&config);
        fs::write(blobs.join(config_digest.replace(':', "-")), &config).unwrap();
        let manifest = root.join("manifest.json");
        fs::write(&manifest,serde_json::to_vec(&json!({"schemaVersion":2,"mediaType":"application/vnd.docker.distribution.manifest.v2+json","config":{"mediaType":"application/vnd.docker.container.image.v1+json","digest":config_digest,"size":config.len()},"layers":[{"mediaType":"application/vnd.ollama.image.model","digest":model_digest,"size":model.len()}]})).unwrap()).unwrap();
        let client = root.join("client.py");
        fs::write(
            &client,
            include_bytes!("fixtures/native/contract-client.py"),
        )
        .unwrap();
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
        let scenario = json!({"model":MODEL,"prompt":PROMPT,"version":{"version":"0.0.0"},
            "tags":{"models":[{"name":MODEL,"model":MODEL,"modified_at":"2026-09-21T00:00:00Z","size":2_339_219_456_u64,"digest":d,"details":details}]},
            "ps":{"models":[{"name":LOADED,"model":LOADED,"size":2_339_219_456_u64,"size_vram":2_339_219_456_u64,"expires_at":"2026-09-21T00:01:00Z","context_length":512,"digest":d,"details":details}]},
            "generated":{"model":MODEL,"created_at":"2026-09-21T00:00:01Z","response":"seven","done":true,"done_reason":"stop","total_duration":142_397_958,"load_duration":74_557_306,"prompt_eval_count":INPUT_TOKENS,"prompt_eval_duration":57_750_770,"eval_count":OUTPUT_TOKENS,"eval_duration":8_671_221}});
        Self {
            root,
            profile,
            scenario,
            origin: Instant::now(),
            _daemon: stand_in,
        }
    }
    fn manifest_digest(&self) -> String {
        self.profile.manifest.sha256.clone()
    }
    fn fault(&mut self, fault: Value) {
        self.scenario["fault"] = fault;
    }
    fn prompt(&mut self, prompt: &str) {
        self.scenario["prompt"] = json!(prompt);
    }
    /// Substitute the client bytes so `subject()` would refuse with `Error::Subject`;
    /// a refusal that still names something else proves it came first.
    fn poison_client(&self) {
        fs::write(&self.profile.client.path, b"substituted").unwrap();
    }
    fn save(&self) {
        fs::write(
            self.root.join("scenario.json"),
            serde_json::to_vec(&self.scenario).unwrap(),
        )
        .unwrap();
    }
    fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.root.join("calls.log"))
            .map(|text| text.lines().map(str::to_owned).collect())
            .unwrap_or_default()
    }
    fn elapsed(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis()).unwrap()
    }
    fn execute(
        &self,
        request: &Request<'static>,
        budget: Duration,
        cancelled: &AtomicBool,
    ) -> Result<native::Run<'static>, Error> {
        self.save();
        native::execute(
            request,
            &self.profile,
            self.origin,
            self.origin + budget,
            cancelled,
        )
    }
    fn run(&self) -> native::Run<'static> {
        self.execute(&full(), Duration::from_secs(60), &AtomicBool::new(false))
            .unwrap()
    }
    fn expected_identity(&self) -> Identity {
        Identity {
            selection: Selection {
                provider: native::PROVIDER.into(),
                model: MODEL.into(),
                effort: None,
            },
            adapter_profile: native::PROFILE.into(),
            runtime_instance: format!(
                "{}:{}:{}",
                self.profile.daemon.boot_id,
                self.profile.daemon.pid,
                self.profile.daemon.start_ticks
            ),
            origin: IdentityOrigin::RuntimeReadback,
            provider_model: Some(LOADED.into()),
            provider_revision: Some(self.manifest_digest()),
            raw: rendered(&self.scenario["ps"]),
        }
    }
    fn expected_usage(&self) -> Usage {
        Usage::Reported {
            provider: native::PROVIDER.into(),
            scope: UsageScope::Invocation,
            form: UsageForm::Cumulative,
            stage: UsageStage::Final,
            input: Some(INPUT_TOKENS),
            output: Some(OUTPUT_TOKENS),
            total: None,
            raw: rendered(&self.scenario["generated"]),
        }
    }
}
impl Drop for Rig {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn operations(run: &native::Run<'_>) -> Vec<&'static str> {
    run.exchanges.iter().map(|e| e.operation).collect()
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
/// The envelope a post-dispatch refusal leaves behind: nothing observed, phase Unknown.
fn nothing_observed(run: &native::Run<'_>) {
    assert_eq!(run.contract.phase(), Phase::Unknown);
    assert!(run.contract.identity().is_none());
    assert!(run.contract.candidate().is_none());
    assert_eq!(run.contract.terminal(), None);
    assert_eq!(*run.contract.usage(), Usage::Unknown);
}
const FULL_SEQUENCE: [&str; 7] = ["version", "tags", "ps", "generate", "version", "tags", "ps"];

// ---------------------------------------------------------------- A · contract returns

/// T08C-01 · Ack/Activity: the adapter emits no Acknowledged or Activity envelope. Its
/// three envelopes are Identity, Final, Terminal; a probe at sequence 4 passes the
/// sequence gate (else `Sequence`) and is refused by the terminal gate (`State`).
#[test]
fn t08c01_no_ack_or_activity_envelope_exactly_three_envelopes() {
    let r = Rig::new();
    let mut run = r.run();
    assert_eq!(run.error, None);
    assert_eq!(run.contract.phase(), Phase::Terminal);
    let probe = run.contract.observe(
        Envelope {
            invocation: full().invocation,
            sequence: 4,
            event: Event::Activity,
        },
        r.elapsed(),
    );
    assert_eq!(probe, Err(ContractError::State));
    assert_eq!(run.contract.phase(), Phase::Unknown);
    settled(&run);
}
/// T08C-02 · Usage: the only stage the adapter emits is `Final`, cumulative, per
/// invocation, carried inside the candidate; the raw record is the provider bytes.
#[test]
fn t08c02_usage_final_stage_only_cumulative_invocation() {
    let r = Rig::new();
    let run = r.run();
    assert_eq!(run.error, None);
    assert_eq!(*run.contract.usage(), r.expected_usage());
    assert_eq!(run.contract.candidate().unwrap().usage, r.expected_usage());
    settled(&run);
}
/// T08C-03 · Candidate: the whole candidate equals the provider record — text, no tool
/// proposals, finish, readback identity, usage and the exact raw bytes.
#[test]
fn t08c03_candidate_whole_value_from_provider_record() {
    let r = Rig::new();
    let run = r.run();
    assert_eq!(run.error, None);
    let expected = Candidate {
        text: "seven".into(),
        has_tool_proposals: false,
        finish: Finish::Stop,
        identity: Some(r.expected_identity()),
        usage: r.expected_usage(),
        raw: rendered(&r.scenario["generated"]),
    };
    assert_eq!(run.contract.candidate(), Some(&expected));
    assert_eq!(
        fs::read(r.root.join("captured-request.json")).unwrap(),
        serde_json::to_vec(&json!({"model":MODEL,"prompt":PROMPT,"stream":false,"raw":true,"truncate":false,"shift":false,"keep_alive":60,"options":{"num_ctx":512,"num_predict":64}})).unwrap()
    );
    settled(&run);
}
/// T08C-04 · `Terminal::Completed` from `done_reason: stop`, after the full seven-exchange
/// readback sequence, every exchange clean.
#[test]
fn t08c04_terminal_completed_from_done_reason_stop() {
    let r = Rig::new();
    let run = r.run();
    assert_eq!(run.error, None);
    assert_eq!(run.provider, ProviderState::ObservedComplete);
    assert_eq!(run.contract.terminal(), Some(Terminal::Completed));
    assert_eq!(run.contract.phase(), Phase::Terminal);
    assert_eq!(operations(&run), FULL_SEQUENCE);
    assert_eq!(r.calls(), FULL_SEQUENCE);
    for e in &run.exchanges {
        let p = e.result.as_ref().unwrap();
        assert_eq!(
            (p.exit_code, p.signal, p.interruption),
            (Some(0), None, None)
        );
        assert!(p.stdout.eof && p.stderr.eof && p.stderr.bytes.is_empty());
    }
    settled(&run);
}
/// T08C-05 · `Terminal::Truncated` from `done_reason: length`: a candidate with
/// `Finish::Length` is retained, the provider observed complete, and no error.
#[test]
fn t08c05_terminal_truncated_from_done_reason_length() {
    let mut r = Rig::new();
    r.scenario["generated"]["done_reason"] = json!("length");
    let run = r.run();
    assert_eq!(run.error, None);
    assert_eq!(run.provider, ProviderState::ObservedComplete);
    assert_eq!(run.contract.terminal(), Some(Terminal::Truncated));
    assert_eq!(run.contract.candidate().unwrap().finish, Finish::Length);
    assert_eq!(operations(&run), FULL_SEQUENCE);
    settled(&run);
}
/// T08C-06 · Refused/Failed/Cancelled terminals are unreachable through this adapter:
/// any other `done_reason` is `Error::Response` with nothing observed and no post readback.
#[test]
fn t08c06_refused_failed_cancelled_terminals_are_unreachable() {
    for reason in ["error", "refusal", "cancelled", "load", ""] {
        let mut r = Rig::new();
        r.scenario["generated"]["done_reason"] = json!(reason);
        let run = r.run();
        assert_eq!(run.error, Some(Error::Response), "{reason:?}");
        assert_eq!(run.provider, ProviderState::Unknown);
        nothing_observed(&run);
        assert_eq!(operations(&run), ["version", "tags", "ps", "generate"]);
        assert_eq!(run.contract.cancellation(), Cancellation::default());
        settled(&run);
    }
}

// ---------------------------------------------------------------- B · error families

/// T08C-07 · The REAL overbound refusal: HTTP 400 under `--fail-with-body` is curl exit 22
/// with the body on stdout and one diagnostic line on stderr; the adapter reports
/// `Error::Process` and retains both streams byte-exact. Oracle bytes are the actual
/// cohort's retained `03-stdout`/`03-stderr`; the request carries the 135168-byte
/// overbound prompt under its pinned recipe.
#[test]
fn t08c07_process_refusal_retains_http_400_body_and_curl_line() {
    assert_eq!(digest(OVERBOUND_PROMPT.as_bytes()), OVERBOUND_RECIPE);
    assert_eq!(OVERBOUND_PROMPT.len(), 135_168);
    let mut r = Rig::new();
    r.prompt(OVERBOUND_PROMPT);
    r.fault(
        json!({"kind":"http_error","stdout":std::str::from_utf8(OVERBOUND_STDOUT).unwrap(),
        "stderr":std::str::from_utf8(OVERBOUND_STDERR).unwrap(),"exit":22}),
    );
    let mut q = request(
        &[Feature::FinalOutput, Feature::Identity, Feature::Usage],
        OVERBOUND_PROMPT,
    );
    q.recipe = Sha256Digest::parse(OVERBOUND_RECIPE).unwrap();
    let run = r
        .execute(&q, Duration::from_secs(60), &AtomicBool::new(false))
        .unwrap();
    assert_eq!(run.error, Some(Error::Process));
    assert_eq!(run.provider, ProviderState::Unknown);
    nothing_observed(&run);
    assert_eq!(run.contract.cancellation(), Cancellation::default());
    assert_eq!(operations(&run), ["version", "tags", "ps", "generate"]);
    let codes: Vec<_> = run
        .exchanges
        .iter()
        .map(|e| e.result.as_ref().unwrap().exit_code)
        .collect();
    assert_eq!(codes, [Some(0), Some(0), Some(0), Some(22)]);
    let generate = run.exchanges[3].result.as_ref().unwrap();
    assert_eq!(generate.stdout.bytes, OVERBOUND_STDOUT);
    assert_eq!(generate.stderr.bytes, OVERBOUND_STDERR);
    assert_eq!(
        serde_json::from_slice::<Value>(&generate.stdout.bytes).unwrap(),
        json!({"error":"the input length exceeds the context length"})
    );
    assert_eq!((generate.signal, generate.interruption), (None, None));
    assert!(generate.stdout.eof && generate.stderr.eof);
    assert!(!generate.stdout.truncated && !generate.stderr.truncated);
    let captured: Value =
        serde_json::from_slice(&fs::read(r.root.join("captured-request.json")).unwrap()).unwrap();
    assert_eq!(captured["prompt"].as_str().unwrap().len(), 135_168);
    settled(&run);
}
/// T08C-08 · The synthetic shape (error body with exit 0) is `Error::Json`, not the
/// refusal: the release's `error=='Json'` predicate is false on T08C-07 and true here.
#[test]
fn t08c08_error_body_with_exit_zero_is_json_not_the_refusal() {
    let mut r = Rig::new();
    r.fault(json!({"kind":"raw","text":std::str::from_utf8(OVERBOUND_STDOUT).unwrap()}));
    let run = r.run();
    assert_eq!(run.error, Some(Error::Json));
    assert_eq!(run.provider, ProviderState::Unknown);
    nothing_observed(&run);
    let generate = run.exchanges[3].result.as_ref().unwrap();
    assert_eq!(generate.exit_code, Some(0));
    assert_eq!(generate.stdout.bytes, OVERBOUND_STDOUT);
    assert!(generate.stderr.bytes.is_empty());
    assert_eq!(operations(&run), ["version", "tags", "ps", "generate"]);
    settled(&run);
}
/// T08C-09 · Transport loss: the client dies by signal mid-generation. The report has no
/// exit code, `transport_lost` leaves the contract Unknown with no envelope observed and
/// no cancellation intent, and the dead contract refuses a later cancel.
#[test]
fn t08c09_transport_loss_signal_death_marks_phase_unknown_without_cancellation() {
    let mut r = Rig::new();
    r.fault(json!({"kind":"signal","number":9}));
    let mut run = r.run();
    assert_eq!(run.error, Some(Error::Process));
    assert_eq!(run.provider, ProviderState::Unknown);
    nothing_observed(&run);
    assert_eq!(run.contract.cancellation(), Cancellation::default());
    let generate = run.exchanges[3].result.as_ref().unwrap();
    assert_eq!(
        (generate.exit_code, generate.signal, generate.interruption),
        (None, Some(9), None)
    );
    assert!(generate.stdout.bytes.is_empty() && generate.stdout.eof);
    assert_eq!(
        run.contract
            .cancel(CancelReason::Operator, r.elapsed())
            .err(),
        Some(ContractError::State)
    );
    assert_eq!(operations(&run), ["version", "tags", "ps", "generate"]);
    settled(&run);
}
/// T08C-10 · Deadline before effect: an expired, over-long or future-origin clock is
/// `Error::Deadline` before any contract, subject check or client process exists.
#[test]
fn t08c10_deadline_before_effect_refuses_without_contract_or_client() {
    let r = Rig::new();
    r.poison_client();
    r.save();
    let now = Instant::now();
    for (origin, deadline) in [
        (now, now),
        (now, now + Duration::from_mins(15) + Duration::from_secs(1)),
        (
            now + Duration::from_secs(60),
            now + Duration::from_secs(120),
        ),
    ] {
        let result = native::execute(
            &full(),
            &r.profile,
            origin,
            deadline,
            &AtomicBool::new(false),
        );
        assert!(matches!(result, Err(Error::Deadline)), "{result:?}");
    }
    assert!(r.calls().is_empty());
}
/// T08C-11 · Deadline during generation: the client is stopped at the work deadline
/// (`Interruption::Timeout`, no exit code), the error is `Process`, and the contract
/// records intent `Deadline` with dispatch `Unsupported`. The `Error::Deadline` variant
/// is not the post-dispatch envelope; the deadline reaches dispatch as this shape.
#[test]
fn t08c11_deadline_during_generation_records_deadline_intent_unsupported() {
    let mut r = Rig::new();
    r.fault(json!({"kind":"pause","seconds":40}));
    let run = r
        .execute(&full(), Duration::from_secs(12), &AtomicBool::new(false))
        .unwrap();
    assert!(
        r.root.join("generation-started").exists(),
        "generation not reached inside the 12 s budget; {:?}",
        run.error
    );
    assert_eq!(run.error, Some(Error::Process));
    assert_eq!(run.provider, ProviderState::Unknown);
    nothing_observed(&run);
    assert_eq!(
        run.contract.cancellation(),
        Cancellation {
            intent: Some(CancelReason::Deadline),
            dispatch: CancelDispatch::Unsupported,
            acknowledged: false,
            adapter_idle: false,
        }
    );
    let generate = run.exchanges[3].result.as_ref().unwrap();
    assert_eq!(
        (generate.exit_code, generate.interruption),
        (None, Some(Interruption::Timeout))
    );
    assert_eq!(operations(&run), ["version", "tags", "ps", "generate"]);
    settled(&run);
}

// ---------------------------------------------------------------- C · cancellation

/// T08C-12 · Cancel before dispatch: a raised flag is `Error::Cancelled` before any
/// subject check or client process; no `CancelCommand` exists because no contract does.
#[test]
fn t08c12_cancel_before_dispatch_returns_cancelled_without_client() {
    let r = Rig::new();
    r.poison_client();
    let result = r.execute(&full(), Duration::from_secs(10), &AtomicBool::new(true));
    assert!(matches!(result, Err(Error::Cancelled)), "{result:?}");
    assert!(r.calls().is_empty());
}
/// T08C-13 · Cancel after dispatch: the operator flag stops the client
/// (`Interruption::Cancelled`), the contract records intent `Operator` with dispatch
/// `Unsupported` — never an issued `CancelCommand` — and a second cancel on the Unknown
/// contract is refused.
#[test]
fn t08c13_cancel_during_generation_is_unsupported_never_a_cancel_command() {
    let mut r = Rig::new();
    r.fault(json!({"kind":"pause","seconds":40}));
    r.save();
    let flag = Arc::new(AtomicBool::new(false));
    let raiser = flag.clone();
    let marker = r.root.join("generation-started");
    let thread = std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(15);
        while !marker.exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(marker.exists(), "generation not reached inside 15 s");
        raiser.store(true, Ordering::Release);
    });
    let mut run = r.execute(&full(), Duration::from_secs(60), &flag).unwrap();
    thread.join().unwrap();
    assert_eq!(run.error, Some(Error::Process));
    assert_eq!(run.provider, ProviderState::Unknown);
    nothing_observed(&run);
    assert_eq!(
        run.contract.cancellation(),
        Cancellation {
            intent: Some(CancelReason::Operator),
            dispatch: CancelDispatch::Unsupported,
            acknowledged: false,
            adapter_idle: false,
        }
    );
    let generate = run.exchanges[3].result.as_ref().unwrap();
    assert_eq!(generate.interruption, Some(Interruption::Cancelled));
    assert_eq!(
        run.contract
            .cancel(CancelReason::Operator, r.elapsed())
            .err(),
        Some(ContractError::State)
    );
    settled(&run);
}
/// T08C-14 · Cancel after terminal: a raced cancel on a completed contract is `Ok(None)`;
/// the terminal stays historical and no intent is recorded.
#[test]
fn t08c14_cancel_after_terminal_is_historical_ok_none() {
    let r = Rig::new();
    let mut run = r.run();
    assert_eq!(run.contract.terminal(), Some(Terminal::Completed));
    let result = run.contract.cancel(CancelReason::Superseded, r.elapsed());
    assert!(matches!(result, Ok(None)), "{result:?}");
    assert_eq!(run.contract.cancellation(), Cancellation::default());
    assert_eq!(run.contract.terminal(), Some(Terminal::Completed));
    assert_eq!(run.contract.phase(), Phase::Terminal);
}

// ---------------------------------------------------------------- D · pre-effect refusals

fn refused_before_subject(feature: Feature) {
    let r = Rig::new();
    r.poison_client();
    let result = r.execute(
        &request(&[Feature::FinalOutput, feature], PROMPT),
        Duration::from_secs(10),
        &AtomicBool::new(false),
    );
    assert!(
        matches!(result, Err(Error::Contract(ContractError::Unsupported(f))) if f == feature),
        "{result:?}"
    );
    assert!(r.calls().is_empty());
}
/// T08C-15 · A request requiring Cancel is refused at launch preflight, before the subject
/// check (a poisoned client would otherwise be `Subject`) and before any client process.
#[test]
fn t08c15_required_cancel_refused_before_subject_or_client() {
    refused_before_subject(Feature::Cancel);
}
/// T08C-16 · A request requiring Deltas is refused the same way.
#[test]
fn t08c16_required_deltas_refused_before_subject_or_client() {
    refused_before_subject(Feature::Deltas);
}
/// T08C-17 · A request requiring `ToolProposals` is refused the same way.
#[test]
fn t08c17_required_tool_proposals_refused_before_subject_or_client() {
    refused_before_subject(Feature::ToolProposals);
}
/// T08C-18 · An effort selection is `Error::Profile` before the contract is built; the
/// adapter never rewrites it to `None`.
#[test]
fn t08c18_effort_selection_refused_as_profile_before_contract() {
    let r = Rig::new();
    r.poison_client();
    let mut q = full();
    q.selection.effort = Some("high".into());
    let result = r.execute(&q, Duration::from_secs(10), &AtomicBool::new(false));
    assert!(matches!(result, Err(Error::Profile)), "{result:?}");
    assert!(r.calls().is_empty());
}
/// T08C-19 · Prompt bound at the point of acquisition: 262144 bytes is dispatched and
/// completes; 262145 bytes is `Contract(InvalidRequest)` with no client process.
#[test]
fn t08c19_prompt_bound_262144_accepted_262145_refused_invalid_request() {
    let long = "x".repeat(262_144);
    let mut r = Rig::new();
    r.prompt(&long);
    let run = r
        .execute(
            &request(
                &[Feature::FinalOutput, Feature::Identity, Feature::Usage],
                &long,
            ),
            Duration::from_secs(60),
            &AtomicBool::new(false),
        )
        .unwrap();
    assert_eq!(run.error, None);
    assert_eq!(run.contract.terminal(), Some(Terminal::Completed));
    settled(&run);
    let over = format!("{long}x");
    let r = Rig::new();
    let result = r.execute(
        &request(
            &[Feature::FinalOutput, Feature::Identity, Feature::Usage],
            &over,
        ),
        Duration::from_secs(10),
        &AtomicBool::new(false),
    );
    assert!(
        matches!(result, Err(Error::Contract(ContractError::InvalidRequest))),
        "{result:?}"
    );
    assert!(r.calls().is_empty());
}

// ---------------------------------------------------------------- E · model/effort recording

/// T08C-20 · The Identity in the envelope is a runtime readback: `provider_revision` is
/// the manifest pin, which is the digest the catalogue and loaded readbacks reported;
/// `provider_model` is the loaded row's name (not the request); effort is `None`.
#[test]
fn t08c20_identity_readback_binds_manifest_digest_runtime_and_effort_none() {
    let r = Rig::new();
    let run = r.run();
    assert_eq!(run.error, None);
    let identity = run.contract.identity().unwrap();
    assert_eq!(identity, &r.expected_identity());
    let reported = format!(
        "sha256:{}",
        r.scenario["ps"]["models"][0]["digest"].as_str().unwrap()
    );
    assert_eq!(
        identity.provider_revision.as_deref(),
        Some(reported.as_str())
    );
    assert_eq!(
        r.scenario["tags"]["models"][0]["digest"],
        r.scenario["ps"]["models"][0]["digest"]
    );
    assert_ne!(identity.provider_model.as_deref(), Some(MODEL));
    assert_eq!(identity.selection.effort, None);
    assert_eq!(identity.origin, IdentityOrigin::RuntimeReadback);
}
/// T08C-21 · Record binding: the two actual cohorts' retained observations (pinned by
/// independent records) carry the pinned model digest through this same mechanism,
/// `RuntimeReadback` origin, the pinned runtime instance, the exact `seven`, and the
/// negative envelope T08C-07 reproduces (exit 22, 55/49-byte streams, nothing observed).
#[test]
fn t08c21_actual_cohort_records_name_pinned_digest_and_expected_envelopes() {
    assert_eq!(
        digest(POSITIVE_OBSERVATION),
        format!("sha256:{POSITIVE_OBSERVATION_SHA256}")
    );
    assert_eq!(
        digest(OVERBOUND_OBSERVATION),
        format!("sha256:{OVERBOUND_OBSERVATION_SHA256}")
    );
    let positive: Value = serde_json::from_slice(POSITIVE_OBSERVATION).unwrap();
    assert_eq!(
        positive["identity"]["provider_revision"],
        PINNED_MODEL_DIGEST
    );
    assert_eq!(positive["identity"]["origin"], "RuntimeReadback");
    assert_eq!(
        positive["identity"]["runtime_instance"],
        PINNED_RUNTIME_INSTANCE
    );
    assert_eq!(positive["candidate"]["text"], "seven");
    assert_eq!(positive["candidate"]["finish"], "Stop");
    assert_eq!(positive["terminal"], "Some(Completed)");
    assert_eq!(positive["provider"], "ObservedComplete");
    assert_eq!(positive["error"], Value::Null);
    assert_eq!(positive["acceptance_claim"], false);
    let usage = positive["candidate"]["usage"].as_str().unwrap();
    assert!(usage.starts_with("Reported { provider: \"ollama-local\", scope: Invocation, form: Cumulative, stage: Final, input: Some(40), output: Some(2), total: None,"), "{usage}");
    let positive_ops: Vec<_> = positive["exchanges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["operation"].as_str().unwrap())
        .collect();
    assert_eq!(positive_ops, FULL_SEQUENCE);
    let negative: Value = serde_json::from_slice(OVERBOUND_OBSERVATION).unwrap();
    assert_eq!(negative["error"], "Process");
    assert_eq!(negative["provider"], "Unknown");
    assert_eq!(negative["phase"], "Unknown");
    assert_eq!(negative["terminal"], "None");
    assert_eq!(negative["identity"], Value::Null);
    assert_eq!(negative["candidate"], Value::Null);
    assert_eq!(negative["typed_completion"], false);
    let rows: Vec<_> = negative["exchanges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["operation"].as_str().unwrap(),
                e["exit_code"].as_u64(),
                e["stdout"]["observed_bytes"].as_u64(),
                e["stderr"]["observed_bytes"].as_u64(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("version", Some(0), Some(19), Some(0)),
            ("tags", Some(0), Some(1092), Some(0)),
            ("ps", Some(0), Some(480), Some(0)),
            ("generate", Some(22), Some(55), Some(49)),
        ]
    );
    assert_eq!(OVERBOUND_STDOUT.len(), 55);
    assert_eq!(OVERBOUND_STDERR.len(), 49);
    assert!(
        negative["exchanges"][3]["report"]
            .as_str()
            .unwrap()
            .contains("exit_code: Some(22)")
    );
}
fn digest_control(key: &str, expected_operations: &[&str]) {
    let mut r = Rig::new();
    r.scenario[key]["models"][0]["digest"] =
        json!(PINNED_MODEL_DIGEST.strip_prefix("sha256:").unwrap());
    let run = r.run();
    assert_eq!(run.error, Some(Error::Identity));
    assert_eq!(run.provider, ProviderState::NotDispatched);
    nothing_observed(&run);
    assert_eq!(operations(&run), expected_operations);
    assert_eq!(r.calls(), expected_operations);
    settled(&run);
}
/// T08C-22 · Control: a catalogue row naming a different digest (here the real pinned
/// one, against a manifest that does not hash to it) is refused at the tags readback,
/// before the loaded readback and before generation.
#[test]
fn t08c22_catalogue_digest_mismatch_refused_at_tags_readback() {
    digest_control("tags", &["version", "tags"]);
}
/// T08C-23 · Control: a loaded row naming a different digest is refused at the ps
/// readback, before generation.
#[test]
fn t08c23_loaded_digest_mismatch_refused_at_ps_readback() {
    digest_control("ps", &["version", "tags", "ps"]);
}

/// T08C-24 · A process that exits between the group census listing it and reading its stat
/// is gone, not an unobserved census. `ESRCH` on the read was reported as I/O failure, so
/// any process on the machine exiting in that window made a clean exchange end in
/// `Interruption::WaitError` -- this target failed about one run in four under load, on
/// whichever case's exchange lost the race (T08C-06, -08 and -19 all observed). The
/// decision is pure, so it is pinned here by argument.
#[test]
fn t08c24_a_process_gone_mid_census_is_not_a_census_failure() {
    use rustix::io::Errno;
    assert!(exited_during_census(Some(Errno::SRCH.raw_os_error())));
    assert!(exited_during_census(Some(Errno::NOENT.raw_os_error())));
    for errno in [
        Errno::ACCESS,
        Errno::PERM,
        Errno::IO,
        Errno::INVAL,
        Errno::NOTDIR,
    ] {
        assert!(
            !exited_during_census(Some(errno.raw_os_error())),
            "{errno:?} is an unobserved census, not an exited process"
        );
    }
    assert!(
        !exited_during_census(None),
        "an error with no errno is not an exit"
    );
}
