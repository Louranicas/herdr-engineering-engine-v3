//! The offline native fixture shared by the t08 contract battery and the t28 runtime proofs (B14a-4,
//! R18 A12): one daemon stand-in, one pin/digest/rendering helper set and one fixture builder — the
//! pieces both crates need, in one file, so neither holds a copy. Never opens a network connection.

use habitat_engine::worker::native::{Daemon, FilePin, Profile};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

pub fn pin(path: &Path) -> FilePin {
    let raw = fs::read(path).unwrap();
    FilePin {
        path: path.to_owned(),
        sha256: digest(&raw),
        bytes: raw.len() as u64,
    }
}

/// A live process pinned as the fixture's daemon. Before QC-F3b the fixture pinned the test
/// executable itself (47 MiB in debug), which the adapter re-hashes through `/proc/<pid>/exe`
/// up to three times per run (`native::daemon` inside `subject` before and after generation
/// and once before the request): 87 debug SHA-256 passes over 47 MiB were the entire 10x
/// debug/release cost. `/usr/bin/sleep` is tens of KiB; the bound below is the control.
pub struct DaemonStandIn {
    child: Child,
}

impl DaemonStandIn {
    const EXECUTABLE: &'static str = "/usr/bin/sleep";
    /// QC-F3b control: a stand-in above this bound reintroduces the hashing cost.
    const EXECUTABLE_BOUND: u64 = 1024 * 1024;

    pub fn spawn() -> Self {
        let child = Command::new(Self::EXECUTABLE)
            .arg("600")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self { child }
    }

    pub fn daemon(&self) -> Daemon {
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

/// A pinned install under `root` (created here, 0700) with the finite offline model blobs, the
/// manifest, the contract fake client, and the base scenario the fake answers from: `model` is the
/// catalogue's name, `loaded` the resident row's (distinct on purpose, so `provider_model` proves
/// readback), `prompt` what the fake expects to be asked, and the generate answer's token counts.
/// The caller writes `scenario.json` (and `expect`, its own literals) before running.
pub fn fixture(
    root: &Path,
    daemon: &DaemonStandIn,
    model: &str,
    loaded: &str,
    prompt: &str,
    tokens: (u64, u64),
) -> (Profile, Value) {
    fs::DirBuilder::new().mode(0o700).create(root).unwrap();
    let blobs = root.join("blobs");
    fs::DirBuilder::new().mode(0o700).create(&blobs).unwrap();
    let bytes = b"not a real model; finite offline contract fixture";
    let model_digest = digest(bytes);
    fs::write(blobs.join(model_digest.replace(':', "-")), bytes).unwrap();
    let config = serde_json::to_vec(&json!({"model_format":"gguf","model_family":"llama","model_families":["llama"],"model_type":"3.2B","file_type":"Q4_K_M","architecture":"amd64","os":"linux","rootfs":{"type":"layers","diff_ids":[model_digest]}})).unwrap();
    let config_digest = digest(&config);
    fs::write(blobs.join(config_digest.replace(':', "-")), &config).unwrap();
    let manifest = root.join("manifest.json");
    fs::write(&manifest,serde_json::to_vec(&json!({"schemaVersion":2,"mediaType":"application/vnd.docker.distribution.manifest.v2+json","config":{"mediaType":"application/vnd.docker.container.image.v1+json","digest":config_digest,"size":config.len()},"layers":[{"mediaType":"application/vnd.ollama.image.model","digest":model_digest,"size":bytes.len()}]})).unwrap()).unwrap();
    let client = root.join("client.py");
    fs::write(
        &client,
        include_bytes!("fixtures/native/contract-client.py"),
    )
    .unwrap();
    fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).unwrap();
    let profile = Profile {
        model: model.into(),
        manifest: pin(&manifest),
        blobs,
        client: pin(&client),
        daemon: daemon.daemon(),
        directory: root.to_owned(),
    };
    let details = json!({"parent_model":"","format":"gguf","family":"llama","families":["llama"],"parameter_size":"3.2B","quantization_level":"Q4_K_M"});
    let d = profile.manifest.sha256.strip_prefix("sha256:").unwrap();
    let (input_tokens, output_tokens) = tokens;
    let scenario = json!({"model":model,"prompt":prompt,
        "expect":{"raw":true,"options":{"num_ctx":512,"num_predict":64}},
        "version":{"version":"0.0.0"},
        "tags":{"models":[{"name":model,"model":model,"modified_at":"2026-09-21T00:00:00Z","size":2_339_219_456_u64,"digest":d,"details":details}]},
        "ps":{"models":[{"name":loaded,"model":loaded,"size":2_339_219_456_u64,"size_vram":2_339_219_456_u64,"expires_at":"2026-09-21T00:01:00Z","context_length":512,"digest":d,"details":details}]},
        "generated":{"model":model,"created_at":"2026-09-21T00:00:01Z","response":"seven","done":true,"done_reason":"stop","total_duration":142_397_958,"load_duration":74_557_306,"prompt_eval_count":input_tokens,"prompt_eval_duration":57_750_770,"eval_count":output_tokens,"eval_duration":8_671_221}});
    (profile, scenario)
}
