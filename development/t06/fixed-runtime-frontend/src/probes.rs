//! Actual bounded local observations, invoked only by the explicit run entrypoint.
use crate::{
    Result, checked,
    manifest::{self, Manifest, Pin},
};
use habitat_engine::worker::process::{self, ProcessReport, ProcessSpec};
use hee3_fixed_task_runtime::clock::TaskClock;
use serde_json::{Value, json};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Actual probe custody survives later preparation or evidence-persistence errors.
#[derive(Default)]
pub(crate) struct Cleanup {
    active: Option<String>,
    observations: Vec<Value>,
    pending: Option<ProcessReport>,
}
impl Cleanup {
    pub(crate) fn begin(&mut self, label: &str) -> Result<()> {
        if !self.settled() {
            return Err("previous probe custody remains unresolved".into());
        }
        self.active = Some(label.into());
        Ok(())
    }
    pub(crate) fn settled(&self) -> bool {
        self.active.is_none() && self.pending.is_none()
    }
    pub(crate) fn observation(&self) -> Value {
        json!({"settled":self.settled(),"active_probe":self.active,
            "pending_child_retained":self.pending.is_some(),"reports":self.observations,
            "initial_state":"no process dispatched; later settlement requires actual process report"})
    }
    fn observed(&mut self, label: &str, r: &ProcessReport) -> bool {
        self.observations
            .push(json!({"probe":label,"report":report(r)}));
        let settled = r.leader_reaped
            && r.process_group_settled
            && r.pending.is_none()
            && r.stdout.eof
            && r.stderr.eof
            && !r.stdout.failed
            && !r.stderr.failed;
        if settled {
            self.active = None;
        }
        settled
    }
}

pub struct Observations {
    pub versions: [ProcessReport; 3],
    pub host: Host,
    pub provenance: Vec<u8>,
}
pub struct Host {
    pub os_release: String,
    pub kernel_release: String,
    pub boot_id: String,
    pub cpuinfo: String,
    pub meminfo: String,
    pub architecture: String,
    pub observed_unix_ms: u64,
}
fn stream(s: &process::Stream) -> Value {
    json!({"hex":manifest::hex(&s.bytes),"observed_bytes":s.observed_bytes,
    "eof":s.eof,"truncated":s.truncated,"failed":s.failed})
}
fn report(r: &ProcessReport) -> Value {
    json!({"leader_pid":r.leader_pid,"exit_code":r.exit_code,"signal":r.signal,
    "interruption":format!("{:?}",r.interruption),"interruption_observed_after_start_ms":r.interruption_observed_at.and_then(|t|t.checked_duration_since(r.started_at)).map(|d|d.as_millis().to_string()),
    "stdout":stream(&r.stdout),"stderr":stream(&r.stderr),"elapsed_ns":r.elapsed.as_nanos().to_string(),
    "signals":format!("{:?}",r.signals),"leader_reaped":r.leader_reaped,"group_settled":r.process_group_settled,"observer_ready":r.observer_ready,"pending":r.pending.is_some()})
}
pub(crate) fn command(
    pin: &Pin,
    args: &[&str],
    directory: &Path,
    output: &Path,
    label: &str,
    clock: TaskClock,
    control: (&mut Cleanup, &AtomicBool),
) -> Result<ProcessReport> {
    let (cleanup, cancelled) = control;
    if cancelled.load(Ordering::Acquire) {
        return Err("preparation stopped before probe dispatch".into());
    }
    manifest::read(pin, 16 * 1024 * 1024, clock.deadline())?;
    let environment = vec![
        ("PATH".into(), "/usr/bin:/bin".into()),
        ("LANG".into(), "C".into()),
        ("LC_ALL".into(), "C".into()),
    ];
    let spec = ProcessSpec {
        executable: pin.path.clone(),
        arguments: args.iter().map(Into::into).collect(),
        directory: directory.into(),
        environment,
        input: vec![],
        stream_limit: 1024 * 1024,
    };
    let start = clock.elapsed();
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(10))
        .ok_or("probe deadline")?
        .min(clock.deadline());
    manifest::json(
        &output.join(format!("{label}.command.json")),
        &json!({"executable":pin,"argv":args,"cwd":directory,"environment":{"PATH":"/usr/bin:/bin","LANG":"C","LC_ALL":"C"},"task_elapsed_before_ns":start.as_nanos().to_string(),"relative_probe_cap_ms":10000,"same_task_deadline":true}),
    )?;
    cleanup.begin(label)?;
    let actual = process::run(&spec, deadline, cancelled);
    match actual {
        Ok(r) => {
            if !cleanup.observed(label, &r) {
                cleanup.pending = Some(r);
                let retained = manifest::json(
                    &output.join(format!("{label}.result.json")),
                    &cleanup.observation(),
                );
                return Err(format!(
                    "probe cleanup unresolved; persistence: {retained:?}"
                ));
            }
            manifest::write(&output.join(format!("{label}.stdout")), &r.stdout.bytes)?;
            manifest::write(&output.join(format!("{label}.stderr")), &r.stderr.bytes)?;
            manifest::json(&output.join(format!("{label}.result.json")), &report(&r))?;
            manifest::read(pin, 16 * 1024 * 1024, clock.deadline())?;
            Ok(r)
        }
        Err(error) => {
            let text = format!("{error:?}");
            let retained = manifest::json(
                &output.join(format!("{label}.result.json")),
                &json!({"refusal":text,"cleanup":cleanup.observation()}),
            );
            Err(format!("{text}; probe persistence: {retained:?}"))
        }
    }
}
fn complete(r: &ProcessReport) -> Result<()> {
    if r.exit_code != Some(0)
        || r.signal.is_some()
        || r.interruption.is_some()
        || !r.leader_reaped
        || !r.process_group_settled
        || r.pending.is_some()
        || [&r.stdout, &r.stderr]
            .iter()
            .any(|s| !s.eof || s.failed || s.truncated || s.observed_bytes != s.bytes.len() as u64)
        || !r.stderr.bytes.is_empty()
        || r.stdout.bytes.is_empty()
    {
        return Err(
            "version/host probe incomplete or unsuccessful; raw observations retained".into(),
        );
    }
    Ok(())
}
fn host_file(path: &Path, output: &Path, label: &str, deadline: Instant) -> Result<String> {
    manifest::tick(deadline)?;
    let mut file = checked(File::open(path))?;
    let mut bytes = Vec::new();
    checked(file.by_ref().take(1024 * 1024 + 1).read_to_end(&mut bytes))?;
    if bytes.is_empty() || bytes.len() > 1024 * 1024 {
        return Err("host observation bound".into());
    }
    manifest::write(&output.join(label), &bytes)?;
    manifest::tick(deadline)?;
    checked(String::from_utf8(bytes))
}
fn capture_host(output: &Path, clock: TaskClock, arch: &ProcessReport) -> Result<Host> {
    Ok(Host {
        os_release: host_file(
            Path::new("/etc/os-release"),
            output,
            "os-release",
            clock.deadline(),
        )?,
        kernel_release: host_file(
            Path::new("/proc/sys/kernel/osrelease"),
            output,
            "kernel-release",
            clock.deadline(),
        )?,
        boot_id: host_file(
            Path::new("/proc/sys/kernel/random/boot_id"),
            output,
            "boot-id",
            clock.deadline(),
        )?,
        cpuinfo: host_file(
            Path::new("/proc/cpuinfo"),
            output,
            "cpuinfo",
            clock.deadline(),
        )?,
        meminfo: host_file(
            Path::new("/proc/meminfo"),
            output,
            "meminfo",
            clock.deadline(),
        )?,
        architecture: checked(std::str::from_utf8(&arch.stdout.bytes))?
            .trim()
            .into(),
        observed_unix_ms: checked(hee3_fixed_task_runtime::clock::unix_ms())?,
    })
}

pub(crate) fn capture(
    m: &Manifest,
    output: &Path,
    clock: TaskClock,
    cleanup: &mut Cleanup,
    cancelled: &AtomicBool,
) -> Result<Observations> {
    manifest::private_dir(output)?;
    let compiler = command(
        &m.tools.compiler.input,
        &["-Vv"],
        output,
        output,
        "rustc",
        clock,
        (cleanup, cancelled),
    )?;
    let bwrap = command(
        &m.tools.bwrap,
        &["--version"],
        output,
        output,
        "bubblewrap",
        clock,
        (cleanup, cancelled),
    )?;
    let systemd = command(
        &m.tools.systemd_run,
        &["--version"],
        output,
        output,
        "systemd-run",
        clock,
        (cleanup, cancelled),
    )?;
    for r in [&compiler, &bwrap, &systemd] {
        complete(r)?;
    }
    let arch = command(
        &m.tools.uname,
        &["-m"],
        output,
        output,
        "uname",
        clock,
        (cleanup, cancelled),
    )?;
    complete(&arch)?;
    let host = capture_host(output, clock, &arch)?;
    manifest::json(
        &output.join("host-observations.json"),
        &json!({"observed_unix_ms":host.observed_unix_ms,"architecture":host.architecture,"files":["os-release","kernel-release","boot-id","cpuinfo","meminfo"],"origin":"actual local /etc and /proc reads plus bounded uname; raw bytes retained"}),
    )?;
    let provenance = checked(serde_json::to_vec(&json!({
        "kind":"fixed-frontend-local-probes/1",
        "commands":[
            {"input":m.tools.compiler.input,"argv":["-Vv"],"result":report(&compiler)},
            {"input":m.tools.bwrap,"argv":["--version"],"result":report(&bwrap)},
            {"input":m.tools.systemd_run,"argv":["--version"],"result":report(&systemd)},
            {"input":m.tools.uname,"argv":["-m"],"result":report(&arch)}],
        "cwd":output,"environment":{"PATH":"/usr/bin:/bin","LANG":"C","LC_ALL":"C"},
        "host_sources":["/etc/os-release","/proc/sys/kernel/osrelease","/proc/sys/kernel/random/boot_id","/proc/cpuinfo","/proc/meminfo"],
        "host_timestamp_unix_ms":host.observed_unix_ms,
        "origin":"Actual bounded local process reports and local file observations; no candidate/provider execution in these probes"
    })))?;
    manifest::write(&output.join("provenance.json"), &provenance)?;
    Ok(Observations {
        versions: [compiler, bwrap, systemd],
        host,
        provenance,
    })
}
