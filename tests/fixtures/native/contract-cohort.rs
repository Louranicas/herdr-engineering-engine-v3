//! Private contract-cohort helper. Runs ONE request through the SAME `worker::Contract`
//! path dispatch uses (`native::execute`) against the endpoint a profile file names, and
//! writes the typed envelope. It carries no oracle text: exit 0 is typed completion
//! (`error == None`, provider `ObservedComplete`, terminal `Completed`, client cleanup
//! settled), exit 1 is any other retained outcome, exit 2 is a usage/binding error.
//! Exit zero is not engine acceptance; the caller owns the oracle.
//!
//! Profile JSON: the driver's `Input` fields plus `prompt` (absolute path, at most
//! 262144 UTF-8 bytes) whose sha256 must equal `recipe`.
//!
//! Expected positive envelope (explicit-word cohort, `fixtures/native/actual/
//! positive-observation.json`): `error: null`, `provider: ObservedComplete`, `phase:
//! Terminal`, `terminal: Completed`, `cancellation` all default, `identity.origin:
//! RuntimeReadback`, `identity.provider_revision: sha256:a80c4f17…`, `identity.selection
//! .effort: null`, `candidate.text: "seven"`, `candidate.finish: Stop`, usage
//! `Final/Invocation/Cumulative` with `input`/`output` present and `total: null`, seven
//! exchanges `version,tags,ps,generate,version,tags,ps` all exit 0, exit code 0.
//!
//! Expected negative envelope (overbound cohort, `fixtures/native/actual/
//! overbound-observation.json`): `error: "Process"`, `provider: Unknown`, `phase: Unknown`,
//! `terminal: null`, `cancellation` all default, `identity: null`, `candidate: null`, four
//! exchanges `version,tags,ps,generate` with exit `0,0,0,22`, generate stdout exactly
//! `{"error":"the input length exceeds the context length"}` (55 bytes) and stderr exactly
//! `curl: (22) The requested URL returned error: 400` + LF (49 bytes), exit code 1.
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::worker::native::{self, Daemon, FilePin, Profile, ProviderState};
use habitat_engine::worker::process::{GroupState, WaitOwnership};
use habitat_engine::worker::{self, Capabilities, Feature, Request, Selection, Terminal, Usage};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::os::unix::fs::DirBuilderExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    path: PathBuf,
    sha256: String,
    bytes: u64,
}
impl From<Pin> for FilePin {
    fn from(pin: Pin) -> Self {
        Self {
            path: pin.path,
            sha256: pin.sha256,
            bytes: pin.bytes,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Runtime {
    pid: u32,
    start_ticks: u64,
    boot_id: String,
    executable_sha256: String,
    executable_bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    task: String,
    attempt: String,
    invocation: String,
    generation: String,
    recipe: String,
    workspace: String,
    prompt: PathBuf,
    model: String,
    manifest: Pin,
    blobs: PathBuf,
    client: Pin,
    daemon: Runtime,
    directory: PathBuf,
}
fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        let _ = write!(out, "{byte:02x}");
    }
    out
}
fn bounded_read(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let mut raw = Vec::new();
    fs::File::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .take(u64::try_from(limit + 1).map_err(|e| e.to_string())?)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > limit {
        return Err(format!("{} exceeds {limit} bytes", path.display()));
    }
    Ok(raw)
}
fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}
fn usage_json(usage: &Usage) -> Value {
    match usage {
        Usage::Unknown => json!("Unknown"),
        Usage::Reported {
            provider,
            scope,
            form,
            stage,
            input,
            output,
            total,
            raw,
        } => json!({"provider":provider,"scope":format!("{scope:?}"),"form":format!("{form:?}"),
            "stage":format!("{stage:?}"),"input":input,"output":output,"total":total,
            "raw_sha256":digest(raw),"raw_bytes":raw.len()}),
    }
}
fn retain(
    run: &mut native::Run<'_>,
    output: &Path,
    deadline: Instant,
) -> Result<(Vec<Value>, bool), String> {
    let mut exchanges = Vec::new();
    let mut cleanup = true;
    for (index, exchange) in run.exchanges.iter_mut().enumerate() {
        match &mut exchange.result {
            Err(error) => exchanges
                .push(json!({"operation":exchange.operation,"refusal":format!("{error:?}")})),
            Ok(report) => {
                for (name, stream) in [("stdout", &report.stdout), ("stderr", &report.stderr)] {
                    fs::write(output.join(format!("{index:02}-{name}")), &stream.bytes)
                        .map_err(|e| e.to_string())?;
                }
                let initial_pending = report.pending.is_some();
                let mut settled =
                    report.leader_reaped && report.process_group_settled && !initial_pending;
                if let Some(child) = report.pending.as_mut() {
                    let cleanup_deadline = (Instant::now() + Duration::from_secs(10)).min(deadline);
                    while Instant::now() < cleanup_deadline {
                        let poll = child.poll_cleanup(cleanup_deadline);
                        settled = poll.ownership == WaitOwnership::Reaped
                            && poll.group == GroupState::Empty;
                        if settled {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
                cleanup &= settled;
                exchanges.push(json!({"operation":exchange.operation,"exit_code":report.exit_code,
                    "signal":report.signal,"interruption":report.interruption.map(|i|format!("{i:?}")),
                    "leader_reaped":report.leader_reaped,"process_group_settled":report.process_group_settled,
                    "initial_pending":initial_pending,"final_client_cleanup":settled,
                    "stdout":{"observed_bytes":report.stdout.observed_bytes,"retained_bytes":report.stdout.bytes.len(),"sha256":digest(&report.stdout.bytes),"eof":report.stdout.eof,"truncated":report.stdout.truncated,"failed":report.stdout.failed},
                    "stderr":{"observed_bytes":report.stderr.observed_bytes,"retained_bytes":report.stderr.bytes.len(),"sha256":digest(&report.stderr.bytes),"eof":report.stderr.eof,"truncated":report.stderr.truncated,"failed":report.stderr.failed}}));
            }
        }
    }
    Ok((exchanges, cleanup))
}
fn run(input_path: &Path, output: &Path) -> Result<bool, String> {
    if !input_path.is_absolute() || !output.is_absolute() {
        return Err("absolute input/output paths required".into());
    }
    fs::DirBuilder::new()
        .mode(0o700)
        .create(output)
        .map_err(|e| e.to_string())?;
    let raw = bounded_read(input_path, 65_536)?;
    fs::write(output.join("profile.json"), &raw).map_err(|e| e.to_string())?;
    let input: Input = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    if input.task == input.attempt
        || input.task == input.invocation
        || input.attempt == input.invocation
    {
        return Err("task/attempt/invocation must be distinct".into());
    }
    if !input.prompt.is_absolute() {
        return Err("absolute prompt path required".into());
    }
    let prompt_bytes = bounded_read(&input.prompt, 262_144)?;
    if digest(&prompt_bytes) != input.recipe {
        return Err("recipe does not bind the prompt bytes".into());
    }
    let prompt = String::from_utf8(prompt_bytes).map_err(|e| e.to_string())?;
    let parse = |e| format!("invalid binding: {e:?}");
    let request = Request {
        invocation: worker::Invocation {
            binding: worker::Binding {
                task: UuidV4::parse(&input.task).map_err(parse)?,
                attempt: UuidV4::parse(&input.attempt).map_err(parse)?,
                generation: input.generation.parse().map_err(parse)?,
            },
            id: UuidV4::parse(&input.invocation).map_err(parse)?,
        },
        recipe: Sha256Digest::parse(&input.recipe).map_err(parse)?,
        workspace: Sha256Digest::parse(&input.workspace).map_err(parse)?,
        adapter_profile: native::PROFILE.into(),
        selection: Selection {
            provider: native::PROVIDER.into(),
            model: input.model.clone(),
            effort: None,
        },
        required: Capabilities::new(&[Feature::FinalOutput, Feature::Identity, Feature::Usage]),
        prompt,
    };
    let profile = Profile {
        model: input.model,
        manifest: input.manifest.into(),
        blobs: input.blobs,
        client: input.client.into(),
        directory: input.directory,
        daemon: Daemon {
            pid: input.daemon.pid,
            start_ticks: input.daemon.start_ticks,
            boot_id: input.daemon.boot_id,
            executable_sha256: input.daemon.executable_sha256,
            executable_bytes: input.daemon.executable_bytes,
        },
    };
    let origin = Instant::now();
    let deadline = origin + Duration::from_secs(240);
    let cancelled = AtomicBool::new(false);
    let mut run = match native::execute(&request, &profile, origin, deadline, &cancelled) {
        Ok(run) => run,
        Err(error) => {
            write_json(
                &output.join("envelope.json"),
                &json!({"pre_effect_refusal":format!("{error:?}"),"task_elapsed_ns":origin.elapsed().as_nanos().to_string(),"acceptance_claim":false}),
            )?;
            return Ok(false);
        }
    };
    let (exchanges, cleanup) = retain(&mut run, output, deadline)?;
    let complete = run.error.is_none()
        && run.provider == ProviderState::ObservedComplete
        && run.contract.terminal() == Some(Terminal::Completed)
        && cleanup;
    let cancellation = run.contract.cancellation();
    let identity = run.contract.identity().map(|id| json!({
        "selection":{"provider":id.selection.provider,"model":id.selection.model,"effort":id.selection.effort},
        "adapter_profile":id.adapter_profile,"runtime_instance":id.runtime_instance,"origin":format!("{:?}",id.origin),
        "provider_model":id.provider_model,"provider_revision":id.provider_revision,
        "raw_sha256":digest(&id.raw),"raw_bytes":id.raw.len()}));
    let candidate = run.contract.candidate().map(|c| {
        json!({
        "text":c.text,"has_tool_proposals":c.has_tool_proposals,"finish":format!("{:?}",c.finish),
        "usage":usage_json(&c.usage),"raw_sha256":digest(&c.raw),"raw_bytes":c.raw.len()})
    });
    write_json(
        &output.join("envelope.json"),
        &json!({"task":input.task,"attempt":input.attempt,"invocation":input.invocation,
        "recipe":input.recipe,"task_elapsed_ns":origin.elapsed().as_nanos().to_string(),
        "error":run.error.map(|e|format!("{e:?}")),"provider":format!("{:?}",run.provider),
        "phase":format!("{:?}",run.contract.phase()),"terminal":run.contract.terminal().map(|t|format!("{t:?}")),
        "cancellation":{"intent":cancellation.intent.map(|i|format!("{i:?}")),"dispatch":format!("{:?}",cancellation.dispatch),
            "acknowledged":cancellation.acknowledged,"adapter_idle":cancellation.adapter_idle},
        "contract_usage":usage_json(run.contract.usage()),"identity":identity,"candidate":candidate,
        "exchanges":exchanges,"final_client_cleanup":cleanup,"typed_completion":complete,"acceptance_claim":false}),
    )?;
    Ok(complete)
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: t08-contract-cohort PROFILE_JSON FRESH_ABSOLUTE_REPORT_DIRECTORY");
        std::process::exit(2);
    }
    match run(Path::new(&args[0]), Path::new(&args[1])) {
        Ok(true) => (),
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
