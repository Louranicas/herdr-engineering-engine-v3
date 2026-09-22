//! Private native-profile observation driver. Exit zero is not engine acceptance.
use habitat_engine::contracts::{Sha256Digest, UuidV4};
use habitat_engine::worker::native::{self, Daemon, FilePin, Profile, ProviderState};
use habitat_engine::worker::process::{GroupState, WaitOwnership};
use habitat_engine::worker::{self, Capabilities, Feature, Request, Selection, Terminal};
use serde::Deserialize;
use serde_json::{Value, json};
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
    model: String,
    manifest: Pin,
    blobs: PathBuf,
    client: Pin,
    daemon: Runtime,
    directory: PathBuf,
}
fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
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
                fs::write(
                    output.join(format!("{index:02}-stdout")),
                    &report.stdout.bytes,
                )
                .map_err(|e| e.to_string())?;
                fs::write(
                    output.join(format!("{index:02}-stderr")),
                    &report.stderr.bytes,
                )
                .map_err(|e| e.to_string())?;
                let initial_pending = report.pending.is_some();
                let initial_report = format!("{report:?}");
                let mut settled =
                    report.leader_reaped && report.process_group_settled && !initial_pending;
                let mut last_poll = None;
                if let Some(child) = report.pending.as_mut() {
                    let cleanup_deadline = (Instant::now() + Duration::from_secs(10)).min(deadline);
                    while Instant::now() < cleanup_deadline {
                        let poll = child.poll_cleanup(cleanup_deadline);
                        settled = poll.ownership == WaitOwnership::Reaped
                            && poll.group == GroupState::Empty;
                        last_poll = Some(format!("{poll:?}"));
                        if settled {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
                cleanup &= settled;
                exchanges.push(json!({"operation":exchange.operation,"report":initial_report,
                    "exit_code":report.exit_code,"signal":report.signal,"interruption":format!("{:?}",report.interruption),
                    "leader_pid":report.leader_pid,"leader_reaped":report.leader_reaped,
                    "process_group_settled":report.process_group_settled,"initial_pending":initial_pending,
                    "stdout":{"observed_bytes":report.stdout.observed_bytes,"eof":report.stdout.eof,"truncated":report.stdout.truncated,"failed":report.stdout.failed},
                    "stderr":{"observed_bytes":report.stderr.observed_bytes,"eof":report.stderr.eof,"truncated":report.stderr.truncated,"failed":report.stderr.failed},
                    "cleanup_poll":last_poll,"final_client_cleanup":settled}));
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
    let mut raw = Vec::new();
    fs::File::open(input_path)
        .map_err(|e| e.to_string())?
        .take(65_537)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > 65_536 {
        return Err("profile exceeds bound".into());
    }
    fs::write(output.join("profile.json"), &raw).map_err(|e| e.to_string())?;
    let input: Input = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    if input.task == input.attempt
        || input.task == input.invocation
        || input.attempt == input.invocation
    {
        return Err("task/attempt/invocation must be distinct".into());
    }
    if input.recipe != "sha256:46f9ab6eac6c2d1149a4eeb47bf146ce57fd7a9cff889686a7ceb4525f8161fa" {
        return Err("recipe does not bind the fixed prompt".into());
    }
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
        prompt: include_str!("prompt.txt").into(),
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
                &output.join("observation.json"),
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
    let identity = run.contract.identity().map(|id| json!({"origin":format!("{:?}",id.origin),"runtime_instance":id.runtime_instance,"provider_model":id.provider_model,"provider_revision":id.provider_revision,"raw":String::from_utf8_lossy(&id.raw)}));
    let candidate = run.contract.candidate().map(|candidate| json!({"text":candidate.text,"finish":format!("{:?}",candidate.finish),"usage":format!("{:?}",candidate.usage),"raw":String::from_utf8_lossy(&candidate.raw)}));
    write_json(
        &output.join("observation.json"),
        &json!({"task":input.task,"attempt":input.attempt,"invocation":input.invocation,
        "task_elapsed_ns":origin.elapsed().as_nanos().to_string(),"error":run.error.map(|e|format!("{e:?}")),"provider":format!("{:?}",run.provider),
        "phase":format!("{:?}",run.contract.phase()),"terminal":format!("{:?}",run.contract.terminal()),"identity":identity,"candidate":candidate,
        "exchanges":exchanges,"final_client_cleanup":cleanup,"typed_completion":complete,"acceptance_claim":false}),
    )?;
    Ok(complete)
}
fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: t08-native-profile-driver PROFILE_JSON FRESH_ABSOLUTE_REPORT_DIRECTORY");
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
