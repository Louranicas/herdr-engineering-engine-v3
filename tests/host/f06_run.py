#!/usr/bin/env python3
"""HT2: the committed F06 host run. It acquires only, and every value and verdict is f06_decide.py's. It runs ON
THE HOST (`flatpak-spawn --host python3 tests/host/f06_run.py …`): serve, systemctl --user, /proc and the ledger
are the host's. Every row prints `row=<id> rc=<code> …`. The daemon is the live ollama.service, resolved by the product
at its endpoint (N6b; Luke, "resolve by endpoint"): nothing here stops, swaps or restarts it. The
disposable HOME is removed at T3. Contracts: hee3-evidence/T28/HO01-T3-20260928/FLOW-CONTRACT.md §3 and
hee3-evidence/T28/HT2-host-f06-20260929/FLOW-CONTRACT.md (§8 and after; §6's swap is superseded by N6b).

    f06_run.py --release DIR --sysroot DIR --run-id ID --evidence DIR --record FILE

No time limit of its own: each observation poll has a budget whose two numbers it prints when it expires (F102).
"""
import argparse
import hashlib
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import time
import uuid
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
sys.path.insert(0, str(HERE))
import f06_decide as decide  # noqa: E402  (the decide module sits beside this runner)

# The daemon executable's pin: sha256sum /usr/bin/ollama in fedora-toolbox-44, measured 2026-09-29, whose prefix is
# HO-04's adapter id (ollama-fc44-12ff8654). P8 requires the one LISTEN holder at :11434 to run exactly these bytes.
OLLAMA_SHA256 = "sha256:12ff8654a500a29048e2a40ff297e778f98c31742ecbd354dc948dbab3cea1aa"
OLLAMA_BYTES = 32276424
ENDPOINT_PORT = 11434
MODEL = "llama3.2:3b"
MODELS = Path.home() / ".ollama/models"
MANIFEST = MODELS / "manifests/registry.ollama.ai/library/llama3.2/3b"
UID = os.geteuid()
RUNTIME = Path(f"/run/user/{UID}")
LOG = []


def row(name, rc, detail=""):
    line = f"row={name} rc={rc} {detail}".rstrip()
    LOG.append(line)
    print(line, flush=True)


def run(argv, env=None, check=True, timeout=None):
    result = subprocess.run(argv, env=env, capture_output=True, text=True, check=False, timeout=timeout)
    if check and result.returncode != 0:
        raise RuntimeError(f"{argv[0]} exited {result.returncode}: {result.stderr.strip()[-300:]}")
    return result


def sha_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return "sha256:" + digest.hexdigest()


def poll(label, predicate, budget):
    """Wait for `predicate()` to hold, at most `budget` seconds; refuse with both numbers when it does not."""
    start = time.monotonic()
    while True:
        value = predicate()
        if value:
            return value
        if time.monotonic() - start >= budget:
            raise RuntimeError(f"{label}: not reached after {time.monotonic() - start:.1f}s of a {budget}s budget")
        time.sleep(0.2)


def private(path):
    path.mkdir(mode=0o700, parents=True, exist_ok=True)
    os.chmod(path, 0o700)


def write(path, data):
    path.write_bytes(data)
    os.chmod(path, 0o600)


def units():
    out = run(["systemctl", "--user", "list-units", "--all", "--no-legend", "--plain"]).stdout
    return sorted(line.split()[0] for line in out.splitlines() if line.strip())


def unit_prop(unit, prop):
    return run(["systemctl", "--user", "show", unit, "-p", prop, "--value"], check=False).stdout.strip()


def manifest_digest(root):
    """`Snapshot::content_digest` by its published recipe (`tests/fixtures/digest/gen-digest-fixtures.sh`):
    directories `<path>\\td`, files `<path>\\tf\\t<x|->\\t<sha256>`, sorted bytewise; the digest is the product's
    spelling of sha256 over those bytes. `prepare` enforces it (baseline_mismatch / protected_mismatch)."""
    script = r"""cd "$1"
find . -mindepth 1 -type d -printf '%P\td\n'
for x in x -; do
  if [ "$x" = x ]; then sel=(-perm /111); else sel=(! -perm /111); fi
  find . -mindepth 1 -type f "${sel[@]}" -printf '%P\n' | while IFS= read -r p; do
    printf '%s\tf\t%s\t%s\n' "$p" "$x" "$(sha256sum < "$p" | cut -c1-64)"
  done
done"""
    listing = run(["bash", "-c", script, "manifest", str(root)]).stdout.encode()
    ordered = b"".join(sorted(line + b"\n" for line in listing.split(b"\n") if line))
    return decide.sha(ordered)


def closure_files(sysroot, target):
    """The candidate namespace's toolchain closure (HT0, the rule `t28_musl_workload.rs` `tools()` states): every
    file of the toolchain's `lib/`, of `lib/rustlib/<target>/lib`, and `rust-lld`, at their place under
    `/toolchain`; and the libraries `ldd` names for rustc and rust-lld, at `/lib64/<name>`. Host digests."""
    rustlib = sysroot / "lib/rustlib"
    lld = next(p / "bin/rust-lld" for p in rustlib.iterdir() if (p / "bin/rust-lld").is_file())
    files = [p for p in (sysroot / "lib").iterdir() if p.is_file()]
    files += [p for p in (rustlib / target / "lib").rglob("*") if p.is_file()]
    files.append(lld)
    rows = {}
    for path in files:
        namespace = str(Path("/toolchain") / path.relative_to(sysroot))
        rows[namespace] = (str(path.resolve()), namespace, sha_file(path.resolve()))
    for binary in (sysroot / "bin/rustc", lld):
        for line in run(["ldd", str(binary)]).stdout.splitlines():
            fields = line.split()
            if "not" in fields and "found" in fields:
                raise RuntimeError(f"ldd {binary}: {line.strip()}")
            path = fields[2] if len(fields) > 2 and fields[1] == "=>" else (fields[0] if fields and fields[0].startswith("/") else None)
            if path and path.startswith("/") and not Path(path).resolve().is_relative_to(sysroot):
                namespace = "/lib64/" + Path(path).name
                rows[namespace] = (str(Path(path).resolve()), namespace, sha_file(Path(path).resolve()))
    return [rows[key] for key in sorted(rows)]


def listen_holder(port):
    """(pid, why) of the one process holding the one LISTEN socket at 127.0.0.1:`port` (v4 loopback or any, v6 any or
    v4-mapped loopback): a WITNESS for P8, measured here independently of the product. The enforcing rule is
    `native::endpoint_holder` at dispatch (N6b), which also requires the listener to be this uid's and treats a missing
    tcp6 as empty; this witness does neither (review 2c P1). (0, reason) when there is not exactly one."""
    spellings = ("0100007F", "00000000", "0" * 32, "0000000000000000FFFF00000100007F")
    inodes = []
    for table in ("/proc/net/tcp", "/proc/net/tcp6"):
        for line in Path(table).read_text().splitlines()[1:]:
            f = line.split()
            host, _, listened = f[1].partition(":")
            if f[3] == "0A" and listened == f"{port:04X}" and host in spellings:
                inodes.append(f[9])
    if len(inodes) != 1:
        return 0, f"listeners={len(inodes)}"
    holders = []
    for pid in filter(str.isdigit, os.listdir("/proc")):
        try:
            if any(os.readlink(f"/proc/{pid}/fd/{fd}") == f"socket:[{inodes[0]}]" for fd in os.listdir(f"/proc/{pid}/fd")):
                holders.append(int(pid))
        except OSError:
            pass
    return (holders[0], "holders=1") if len(holders) == 1 else (0, f"holders={len(holders)}")


def hee3(env, *args):
    result = run(["bash", str(REPO / "integrations/bash/hee3"), *args], env=env, check=False, timeout=120)
    return result.returncode, result.stdout, result.stderr


def main():
    parser = argparse.ArgumentParser()
    for name in ("--release", "--sysroot", "--run-id", "--evidence", "--record"):
        parser.add_argument(name, required=True)
    a = parser.parse_args()
    release, sysroot = Path(a.release).resolve(), Path(a.sysroot).resolve()
    evidence = Path(a.evidence)
    evidence.mkdir(parents=True, exist_ok=True)
    root = Path.home() / ".cache/hee3-t3" / a.run_id
    home = root / "home"
    state = home / ".local/state/herdr-engineering-engine-v3"
    config = home / ".config/herdr-engineering-engine-v3"
    backups = Path("/var/mnt/STORAGE-10TB/herdr-engineering-engine-v3-t3-backups") / a.run_id
    serve_unit = f"herdr-engineering-engine-v3-t3-{a.run_id}.service"
    manifest = json.loads((release / "manifest.json").read_text())
    created_root = False
    before = []
    measured, fields = {}, {"compensating": {"netns_differ": False, "slices": []}}
    lines, problems = [], ["the run did not reach the closure"]
    try:
        # ---- preconditions ----------------------------------------------------------------------------------
        if root.exists():
            raise RuntimeError(f"run root exists: {root}")
        private(home)
        created_root = True
        before = units()
        row("L0", 0, f"units={len(before)}")
        pins = {name: sha_file(path) for name, path in
                (("busctl", "/usr/bin/busctl"), ("systemd_run", "/usr/bin/systemd-run"), ("curl", "/usr/bin/curl"))}
        measured["host_pins"] = (",".join(f"{k}={v[7:19]}" for k, v in sorted(pins.items())),
                                 all(pins[k][7:] == manifest["host_pins"][k] for k in pins))
        engine, shim = release / "habitat-engine", release / "hee-namespace-shim"
        seams = sum(p.read_bytes().count(b"HEE3_TEST_") for p in (engine, shim))
        head = run(["git", "-C", str(REPO), "rev-parse", "HEAD"]).stdout.strip()
        measured["release"] = decide.release_row(
            manifest, release.name, {"habitat-engine": sha_file(engine)[7:], "hee-namespace-shim": sha_file(shim)[7:]},
            seams, head)
        measured["state_root"] = ("absent" if not state.exists() else "present", not state.exists())
        stray = [u for u in before if u.startswith(("hee3aggregate", "hee3-resource"))]
        measured["units"] = (f"hee3={len(stray)}", not stray)
        census = []
        for pid in os.listdir("/proc"):
            if pid.isdigit():
                try:
                    census.append(os.readlink(f"/proc/{pid}/exe"))
                except OSError:
                    pass
        foreign = [exe for exe in census if Path(exe.removesuffix(" (deleted)")).name == "habitat-engine"]
        measured["foreign_serve"] = (f"serve={len(foreign)}", not foreign)
        probe = f"hee3-t3-probe-{a.run_id}"
        run(["systemd-run", "--user", "--quiet", f"--unit={probe}", "/usr/bin/sleep", "infinity"])
        probe_pid = poll("probe MainPID", lambda: int(unit_prop(probe + ".service", "MainPID") or 0), 10)
        cgroup = Path(f"/proc/{probe_pid}/cgroup").read_text().strip()
        run(["systemctl", "--user", "stop", probe + ".service"])
        measured["service_probe"] = (f"cgroup_tail={cgroup.rsplit('/', 1)[-1]}",
                                     cgroup.endswith(f"/{probe}.service") and "libpod" not in cgroup)
        socket = RUNTIME / "habitat-engine/control.sock"
        measured["control_socket"] = ("absent" if not socket.exists() else "present", not socket.exists())
        free_state = shutil.disk_usage(home).free
        backups.parent.mkdir(exist_ok=True)
        free_backup = shutil.disk_usage(backups.parent).free
        measured["headroom"] = (f"state={free_state >> 30}GiB backup={free_backup >> 30}GiB",
                                free_state >= 96 << 30 and free_backup >= 256 << 30
                                and os.stat(home).st_dev != os.stat(backups.parent).st_dev)
        # ---- P8: the live daemon at the endpoint, its bytes and its GPU (N6b; review 2c N1) -----------------------
        holder, why = listen_holder(ENDPOINT_PORT)
        holder_exe = sha_file(f"/proc/{holder}/exe") if holder else "-"
        probe_started = time.monotonic()
        run(["curl", "-s", "--noproxy", "*", "-m", "120", "http://127.0.0.1:11434/api/generate", "-d",
             json.dumps({"model": MODEL, "prompt": "ok", "stream": False, "keep_alive": 60,
                         "options": {"num_predict": 1}})], check=False)
        resident = json.loads(run(["curl", "-s", "--noproxy", "*", "-m", "5", "http://127.0.0.1:11434/api/ps"],
                                  check=False).stdout or "{}").get("models", [])
        vram = max((m.get("size_vram", 0) for m in resident if m.get("name") == MODEL), default=0)
        # The probe's wall time tells a cold model load from a CPU-backed daemon when P8 fails (review 2c P2).
        probe_s = time.monotonic() - probe_started
        measured["native_daemon"] = (f"holder={holder} {why} exe={holder_exe[7:19]} vram={vram >> 20}MiB "
                                     f"probe={probe_s:.1f}s",
                                     holder_exe == OLLAMA_SHA256 and os.path.getsize(f"/proc/{holder}/exe") == OLLAMA_BYTES
                                     and vram > 0)
        # ---- provisioning (the class, its closure, the native file) -------------------------------------------
        target = "x86_64-unknown-linux-musl"
        runtime_files = closure_files(sysroot, target)
        class_dir = config / "classes/rust-library-change-1"
        for d in (config, class_dir, class_dir / "base", class_dir / "base/src", class_dir / "protected",
                  class_dir / "reviewed", config / "grants", config / "native", config / "backup"):
            private(d)
        v1 = REPO / "evaluation/tasks/WL-U64-PARSE-001/v1"
        write(class_dir / "base/src/lib.rs", (v1 / "base/src/lib.rs").read_bytes())
        write(class_dir / "base/Cargo.toml", (v1 / "base/Cargo.toml").read_bytes())
        write(class_dir / "protected/oracle.txt", b"frozen oracle\n")
        write(class_dir / "protected/oracle.json", (v1 / "oracle/cases.json").read_bytes())
        write(class_dir / "protected/public-wrapper.rs", (REPO / "evaluation/harnesses/u64-public-wrapper.rs").read_bytes())
        for entry in (REPO / "tests/fixtures/reviewed-003").iterdir():
            write(class_dir / "reviewed" / entry.name, entry.read_bytes())
        write(class_dir / "authority.json", decide.AUTHORITY)
        write(class_dir / "isolation.json", decide.SPECIFICATION)
        pins_ok = all(Path(host).is_file() for host, _, _ in runtime_files) and len(runtime_files) <= 510
        measured["class_pins"] = (f"runtime_files={len(runtime_files)}", pins_ok)
        workspace = str(uuid.uuid4())
        write(class_dir / "profile.toml", decide.profile({
            "workspace_id": workspace, "base_digest": manifest_digest(class_dir / "base"),
            "protected_digest": manifest_digest(class_dir / "protected"),
            "compiler": (str(sysroot / "bin/rustc"), sha_file(sysroot / "bin/rustc")),
            "shim": (str(shim), sha_file(shim)), "runtime_files": runtime_files, "busctl": pins["busctl"],
            "systemd_run": pins["systemd_run"], "class_grant_id": str(uuid.uuid4()),
            "manifest_sha256": sha_file(MANIFEST), "model": MODEL}).encode())
        native_dir = home / ".local/state/hee3-native"
        private(native_dir)
        write(config / "native/native.toml", decide.native({
            "directory": str(native_dir), "curl_sha256": pins["curl"], "curl_bytes": os.path.getsize("/usr/bin/curl"),
            "manifest_path": str(MANIFEST), "manifest_sha256": sha_file(MANIFEST),
            "manifest_bytes": os.path.getsize(MANIFEST), "blobs": str(MODELS / "blobs"), "unit": "ollama.service",
            "daemon_sha256": OLLAMA_SHA256, "daemon_bytes": OLLAMA_BYTES, "roster_key": str(uuid.uuid4()),
            "endpoint_ref": str(uuid.uuid4())}).encode())
        private(backups)
        write(config / "backup/backup.json", decide.backup(backups, 3600))
        grant_id = str(uuid.uuid4())
        granted = decide.grant(grant_id, UID, int(time.time() * 1000) + 86_400_000)
        write(config / "grants" / f"{grant_id}.json", granted)
        prows, clear = decide.rows(measured)
        for line in prows:
            print(line, flush=True)
            LOG.append(line)
        if not clear:
            raise RuntimeError("a precondition row failed; stopped before S1")
        # ---- S1 commission, S2 serve ---------------------------------------------------------------------------
        env = {"HOME": str(home), "XDG_RUNTIME_DIR": str(RUNTIME), "PATH": "/usr/bin:/bin", "LC_ALL": "C"}
        commissioned = run([str(engine), "commission", "600"], env=env, check=False, timeout=900)
        row("S1", commissioned.returncode, commissioned.stdout.strip()[-200:] or commissioned.stderr.strip()[-200:])
        if commissioned.returncode != 0:
            raise RuntimeError("commission refused")
        run(["systemd-run", "--user", "--quiet", f"--unit={serve_unit}", "--property=TimeoutStopSec=infinity",
             f"--setenv=HOME={home}", f"--setenv=XDG_RUNTIME_DIR={RUNTIME}", str(engine), "serve"])
        poll("control socket", lambda: socket.exists(), 60)
        serve_pid = int(unit_prop(serve_unit, "MainPID") or 0)
        exe = sha_file(f"/proc/{serve_pid}/exe")
        serve_cgroup = Path(f"/proc/{serve_pid}/cgroup").read_text().strip()
        fields["exe_sha256"], fields["engine_sha256"] = exe[7:], manifest["binaries"]["habitat-engine"]["sha256"]
        row("S2", 0, f"{serve_unit} pid={serve_pid}")
        row("S3", 0 if exe[7:] == fields["engine_sha256"] and serve_cgroup.endswith(f"/{serve_unit}") else 1,
            f"exe={exe[7:19]} cgroup_tail={serve_cgroup.rsplit('/', 1)[-1]}")
        # ---- S4 submit through the engine's own door -----------------------------------------------------------
        wrapper = {**env, "HEE3_PRODUCER": str(engine), "HEE3_GRANT_ID": grant_id,
                   "HEE3_SCOPE_SHA256": decide.sha(granted)}
        rc, out, err = hee3(wrapper, "task.submit", f"@idempotency_key={uuid.uuid4()}",
                            "spec:=" + json.dumps(decide.spec(workspace)))
        (evidence / "S4-submit.json").write_text(out + err)
        row("S4", rc, out.strip()[-240:] or err.strip()[-240:])
        task_id = json.loads(out)["body"]["task"]["task_id"] if rc == 0 else None
        if task_id is None:
            raise RuntimeError("task.submit refused")
        # ---- R-exec: observe until the task's attempt settles; R0 while a candidate lives ----------------------
        serve_net = os.readlink(f"/proc/{serve_pid}/ns/net")
        deadline = time.monotonic() + 1500
        final, last_reply = None, None
        while time.monotonic() < deadline:
            for pid in os.listdir("/proc"):
                if not pid.isdigit():
                    continue
                try:
                    # A candidate runs in a transient scope under the check's aggregate slice (bounded path).
                    if Path(os.readlink(f"/proc/{pid}/exe")).name in ("bwrap", "workload-driver", "rustc"):
                        net = os.readlink(f"/proc/{pid}/ns/net")
                        if net != serve_net and "hee3aggregate" in Path(f"/proc/{pid}/cgroup").read_text():
                            fields["compensating"]["netns_differ"] = True
                            fields["compensating"].setdefault("observed", f"pid={pid} net={net} serve={serve_net}")
                except OSError:
                    pass
            for unit in units():
                if unit.startswith("hee3aggregate") and unit not in [s["unit"] for s in fields["compensating"]["slices"]]:
                    fields["compensating"]["slices"].append({"unit": unit, **{p: unit_prop(unit, p) for p in
                                                             ("CPUQuotaPerSecUSec", "MemoryMax", "TasksMax")}})
            rc, out, _ = hee3(wrapper, "task.get", "selector:=" + json.dumps({"task_id": task_id}), "evidence=none")
            if rc == 0:
                reply = last_reply = json.loads(out)["body"]
                attempts = reply.get("attempts", [])
                if decide.observation_finished(reply.get("task", {}).get("state"),
                                               [x.get("state") for x in attempts]):
                    final = reply
                    break
            time.sleep(0.2)
        (evidence / "R-exec-task-get.json").write_text(json.dumps(final, indent=1))
        # On budget, the last state seen names why (review 2c M1): a state the decide does not know is visible.
        last = json.dumps((final or last_reply or {}).get("task", {}))[:240]
        row("R-exec", 0 if final else 1, last if final else f"budget 1500s spent; last task {last}")
        # ---- C1: the ledger, read-only ------------------------------------------------------------------------
        active = json.loads((state / "active.json").read_text())
        generation = active["generation"]
        ledger_path = state / "generations" / str(generation) / "ledger.sqlite3"
        db = sqlite3.connect(f"file:{ledger_path}?mode=ro", uri=True)
        version = db.execute("PRAGMA user_version").fetchone()[0]
        attempts = [{"id": r[0], "state": r[1], "generation": r[2]} for r in
                    db.execute("SELECT id, state, generation FROM attempts WHERE task_id=? ORDER BY CAST(generation AS INT)", (task_id,))]
        task_generation = db.execute("SELECT generation FROM tasks WHERE id=?", (task_id,)).fetchone()[0]
        stale = decide.stale_count([r[0] for r in db.execute(
            "SELECT generation FROM events WHERE task_id=? ORDER BY sequence", (task_id,))])
        last = attempts[-1]["id"] if attempts else None
        settled_event = db.execute("SELECT settled_event FROM attempts WHERE id=?", (last,)).fetchone()[0] if last else None
        records = [{"kind": r[0], "event_id": r[1], "digest": r[2]} for r in
                   db.execute("SELECT kind, event_id, digest FROM attempt_records WHERE attempt_id=?", (last,))]
        settle = {}
        for r in records:
            if r["kind"] == "worker_settle":
                hexd = r["digest"][7:]
                # The object store is objects/sha256/<first two hex>/<hex> (measured on the t3a ledger, 2026-09-29).
                settle = json.loads((state / "generations" / str(generation) / "objects" / "sha256" / hexd[:2] / hexd)
                                    .read_bytes())
        db.close()
        row("C0", 0 if version == decide.LEDGER_VERSION else 1, f"user_version={version}")
        lines, problems = decide.closure({"task": task_id, "task_generation": task_generation, "attempts": attempts,
                                          "stale": stale, "settled_event": settled_event, "records": records,
                                          "settle": settle})
        for line in lines + [f"problem: {p}" for p in problems]:
            print(line, flush=True)
            LOG.append(line)
    except Exception as error:  # every stop is a finding: named, recorded, and the teardown still runs
        row("STOP", 1, f"{type(error).__name__}: {error}")
    finally:
        # ---- T1 stop serve, T2 units, T3 remove the disposable HOME ----------------------------------------------
        started = time.monotonic()
        stopped, why = True, ""
        if unit_prop(serve_unit, "LoadState") == "loaded":
            run(["systemctl", "--user", "stop", serve_unit], check=False)
            try:
                poll("serve gone", lambda: unit_prop(serve_unit, "MainPID") in ("0", ""), 1800)
            except RuntimeError as error:
                stopped, why = False, str(error)
        row("T1", 0 if stopped else 1, why or f"seal={time.monotonic() - started:.2f}s")
        after = units()
        fields["units_before"], fields["units_after"] = before, after
        extra = sorted(set(after) - set(fields["units_before"]))
        row("T2", 0 if not extra else 1, f"extra={extra}")
        if created_root:
            shutil.rmtree(root)
            row("T3", 0 if not root.exists() else 1, f"removed {root}; backups kept at {backups}")
        else:
            row("T3", 0, f"nothing created; {root} left as found")
    fields.update({"dirty": int(run(["git", "-C", str(REPO), "status", "--porcelain"]).stdout != ""),
                   "source_sha": run(["git", "-C", str(REPO), "rev-parse", "HEAD"]).stdout.strip(),
                   "release_manifest_sha256": release.name,
                   "shim_sha256": manifest["binaries"]["hee-namespace-shim"]["sha256"],
                   "test_sha256": hashlib.sha256((HERE / "f06_run.py").read_bytes()
                                                 + (HERE / "f06_decide.py").read_bytes()).hexdigest(),
                   "run_id": a.run_id, "ran_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())})
    fields.setdefault("exe_sha256", "-")
    fields.setdefault("engine_sha256", manifest["binaries"]["habitat-engine"]["sha256"])
    made, refused = decide.record(fields, lines, problems)
    (evidence / "rows.log").write_text("\n".join(LOG) + "\n")
    if made is None:
        print("record refused: " + "; ".join(refused))
        return 1
    Path(a.record).write_text(json.dumps(made, indent=1, sort_keys=True) + "\n")
    print(f"record written: {a.record}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
