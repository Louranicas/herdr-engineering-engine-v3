#!/usr/bin/env python3
"""HT2's pure decisions for the committed F06 host run: every value the run writes and every verdict it prints,
reached by argument (F95). `f06_run.py` acquires; this file decides. `--control` proves each rule by its own
diagnostic. Contracts: hee3-evidence/T28/HO01-T3-20260928/FLOW-CONTRACT.md §3 and
hee3-evidence/T28/HT2-host-f06-20260929/FLOW-CONTRACT.md.

    f06_decide.py --control
"""
import hashlib
import json
import sys

RECORD_SCHEMA = "hee3.host-record/1"
GRANT_SCHEMA = "hee3.grant/1"
BACKUP_SCHEMA = "hee3.backup-target/1"
NATIVE_SCHEMA = "hee3.native/1"
OPERATOR_ROLE = "operator"
CLASS = "rust-library-change/1"
CRITERIA = ["u64-frozen-exact-output"]
ADAPTER = "ollama-fc44-12ff8654/2"
LEDGER_VERSION = 8
# The reviewed closure the class pins: the 003 lane's receipts, installed by digest (tests/fixtures/reviewed-003).
REVIEWED = (
    'expectation = { artifact_id = "c220e7ce-0753-47ef-bdac-15710bc4981c", sha256 = '
    '"sha256:3a7faa5510790c20322ad5829091211eb8c04ab6016e16ec8391433dae3392b9", byte_length = 794, '
    'media_type = "application/json", schema_id = "hee3.receipt/1:ExpectationV1" }\n'
    'review = { artifact_id = "a47470c5-f11c-4f64-9ac8-6dcf80750ed6", sha256 = '
    '"sha256:f288225476120254f5c3a93266fc8f2a8763810462fbddd7c62161107cb39adb", byte_length = 1145, '
    'media_type = "application/json", schema_id = "hee3.receipt/1:ReviewV1" }\n')
AUTHORITY = b'{"authority":"WL-U64 fixed workload","issuer":"operator"}\n'
SPECIFICATION = b'{"isolation":"bwrap --unshare-all; no network"}\n'


def sha(data):
    """The product's spelling of a digest (`contracts::control::request_sha256`)."""
    return "sha256:" + hashlib.sha256(data).hexdigest()


def quote(text):
    return json.dumps(str(text))


# ---- what the run writes ------------------------------------------------------------------------------------

def grant(grant_id, uid, expires_ms):
    """The operator's grant file: owners task, effects read and durable admission. Its scope is its own digest."""
    return (json.dumps({"schema": GRANT_SCHEMA, "grant_id": grant_id, "uid": uid, "role": OPERATOR_ROLE,
                        "owners": ["task"], "effects": ["read", "durable admission"],
                        "expires_unix_ms": str(expires_ms)}, indent=2) + "\n").encode()


def backup(destination, deadline_seconds):
    return (json.dumps({"schema": BACKUP_SCHEMA, "destination": str(destination),
                        "deadline_seconds": deadline_seconds}) + "\n").encode()


def profile(v):
    """The class profile (`hee3.class-profile/2`), from measured values only: `v` holds workspace_id, base_digest,
    protected_digest, compiler and shim ((host, sha)), runtime_files ([(host, namespace, sha)]), busctl, systemd_run,
    manifest_sha256 and model."""
    runtime = "".join(f"  {{ host = {quote(host)}, namespace = {quote(namespace)}, sha256 = {quote(digest)} }},\n"
                      for host, namespace, digest in v["runtime_files"])
    return (f'schema = "hee3.class-profile/2"\nclass = "{CLASS}"\n\n'
            f'[[workspace]]\nid = {quote(v["workspace_id"])}\nbaseline = "base"\n'
            f'baseline_digest = {quote(v["base_digest"])}\nprotected = "protected"\n'
            f'protected_digest = {quote(v["protected_digest"])}\n\n'
            f'[pins]\ncompiler = {{ host = {quote(v["compiler"][0])}, sha256 = {quote(v["compiler"][1])} }}\n'
            f'shim = {{ host = {quote(v["shim"][0])}, sha256 = {quote(v["shim"][1])} }}\n'
            f'runtime_files = [\n{runtime}]\nnamespace_directories = []\n'
            f'busctl_sha256 = {quote(v["busctl"])}\nsystemd_run_sha256 = {quote(v["systemd_run"])}\n'
            f'[reviewed]\n{REVIEWED}'
            f'[grant]\ngrant_id = {quote(v["class_grant_id"])}\nissuer_id = "operator"\n'
            f'authority = {{ file = "authority.json", sha256 = {quote(sha(AUTHORITY))} }}\n'
            f'[effect]\neffect_id = "fixed-u64-workload-output"\nscope = "the T3 host run"\n'
            f'specification = {{ file = "isolation.json", sha256 = {quote(sha(SPECIFICATION))} }}\n\n'
            f'[native]\nmodel = {quote(v["model"])}\nmanifest_sha256 = {quote(v["manifest_sha256"])}\n'
            f'adapter = "{ADAPTER}"\n')


def native(v):
    """The operator's native file, every pin measured on the host: curl, the model manifest, the daemon unit and
    its executable."""
    return (f'schema = "{NATIVE_SCHEMA}"\ndirectory = {quote(v["directory"])}\n\n'
            f'[client]\npath = "/usr/bin/curl"\nsha256 = {quote(v["curl_sha256"])}\nbytes = {v["curl_bytes"]}\n\n'
            f'[install]\nmanifest = {{ path = {quote(v["manifest_path"])}, sha256 = {quote(v["manifest_sha256"])}, '
            f'bytes = {v["manifest_bytes"]} }}\nblobs = {quote(v["blobs"])}\n\n'
            f'[daemon]\nunit = {quote(v["unit"])}\nscope = "user"\n'
            f'executable = {{ sha256 = {quote(v["daemon_sha256"])}, bytes = {v["daemon_bytes"]} }}\n\n'
            f'[roster]\nidempotency_key = {quote(v["roster_key"])}\nendpoint_ref = {quote(v["endpoint_ref"])}\n')


def spec(workspace_id):
    """The one WL-U64 task the run submits."""
    return {"task_class": CLASS, "intent": "Make parse_u64 accept only canonical ASCII decimal text.",
            "criteria": CRITERIA, "privacy": "local_only", "workspace_id": workspace_id,
            "budget": {"mode": "hard", "wall_ms": "600000", "tokens": "0", "currency_microunits": "0"},
            "parent": None}


# ---- what the run decides -----------------------------------------------------------------------------------

def rows(measured):
    """The precondition rows, each `P<n> <name> <value> PASS|FAIL`, and whether S1 may start (every row PASS).
    `measured` maps each row name to (value, ok); a row the run did not measure is a FAIL by name."""
    names = ("P0 class_pins", "P1 host_pins", "P2 release", "P3 state_root", "P4 units", "P5 foreign_serve",
             "P6 service_probe", "P7 control_socket", "P8 native_daemon", "P9 headroom")
    lines, clear = [], True
    for name in names:
        value, ok = measured.get(name.split(" ", 1)[1], ("UNMEASURED", False))
        clear &= bool(ok)
        lines.append(f"{name} {value} {'PASS' if ok else 'FAIL'}")
    lines.append(f"prows={sum(line.endswith('PASS') for line in lines)}/{len(names)} "
                 f"{'S1 may start' if clear else 'stop before S1'}")
    return lines, clear


def closure(ledger):
    """The four F06 lines from the ledger's rows (read-only), and the reasons the closure does not hold.
    `ledger`: task, task_generation, attempts [{id, state, generation}], stale (a count), settled_event,
    records [{kind, event_id}], settle {adapter_profile, input_tokens, output_tokens, identity_sha256}."""
    problems = []
    task, attempts = ledger["task"], ledger["attempts"]
    terminal_states = [a["state"] for a in attempts if a["state"] in ("settled", "unknown")]
    if not attempts or len(terminal_states) != len(attempts):
        problems.append(f"forward: an attempt with neither a reconciled terminal nor an explicit unknown "
                        f"({[a['state'] for a in attempts]})")
    terminal = "reconciled" if attempts and all(a["state"] == "settled" for a in attempts) else "unknown"
    last = max(attempts, key=lambda a: int(a["generation"])) if attempts else {"id": "-", "generation": "-"}
    if ledger["stale"] != 0:
        problems.append(f"generation: stale_accepted={ledger['stale']}")
    kinds = sorted(record["kind"] for record in ledger["records"] if record["event_id"] == ledger["settled_event"])
    settle = ledger.get("settle") or {}
    provider = "native" if str(settle.get("adapter_profile", "")).startswith("ollama-") else "none"
    if provider != "native":
        problems.append(f"execute: the settle names no native adapter ({settle.get('adapter_profile')!r})")
    worker = [r for r in ledger["records"] if r["kind"] == "worker_settle"]
    usage_event = worker[0]["event_id"] if len(worker) == 1 else "-"
    identity_event = usage_event if settle.get("identity_sha256") else "-"
    if settle.get("input_tokens") is None or settle.get("output_tokens") is None:
        usage_event = "-"
    same = usage_event != "-" and usage_event == identity_event == ledger["settled_event"]
    if not same:
        problems.append(f"one_settle: usage {usage_event} and identity {identity_event} are not the settle "
                        f"{ledger['settled_event']}")
    lines = [
        f"F06 forward task={task} attempts={len(attempts)} terminal={terminal} each={len(terminal_states)}/{len(attempts)}",
        f"F06 generation task={task} attempt_generation={last['generation']} "
        f"task_generation={ledger['task_generation']} stale_accepted={ledger['stale']}",
        f"F06 execute attempt={last['id']} provider={provider} settled_event={ledger['settled_event']} "
        f"run_records={','.join(kinds) or '-'}",
        f"F06 one_settle attempt={last['id']} usage_event={usage_event} identity_event={identity_event} "
        f"same={'true' if same else 'false'}",
    ]
    return lines, problems


def record(fields, lines, problems):
    """The committed host record, or the reasons it is refused. A record exists only when all four lines held on
    an install whose digest the manifest names, from a clean tree, with the host's units as they were."""
    refused = list(problems)
    if fields["dirty"] != 0:
        refused.append(f"dirty={fields['dirty']}")
    if fields["exe_sha256"] != fields["engine_sha256"]:
        refused.append(f"serve exe {fields['exe_sha256']} != manifest engine {fields['engine_sha256']}")
    extra = sorted(set(fields["units_after"]) - set(fields["units_before"]))
    if extra:
        refused.append(f"units after != before: {extra}")
    if not fields.get("compensating", {}).get("netns_differ"):
        refused.append("R0: the candidate's network namespace was not observed apart from serve's")
    if refused:
        return None, refused
    return ({"schema": RECORD_SCHEMA, "flow": "F06", "source_sha": fields["source_sha"], "dirty": 0,
             "release_manifest_sha256": fields["release_manifest_sha256"], "engine_sha256": fields["engine_sha256"],
             "shim_sha256": fields["shim_sha256"], "test_sha256": fields["test_sha256"], "run_id": fields["run_id"],
             "asserted": lines, "ran_utc": fields["ran_utc"], "units_before_equals_after": True,
             "compensating": fields["compensating"]}, [])


# ---- the control --------------------------------------------------------------------------------------------

def ledger_fixture(tag):
    """Two ledgers that hold, differing in every field."""
    return {"task": f"t{tag}", "task_generation": str(4 + tag),
            "attempts": [{"id": f"a{tag}", "state": "settled", "generation": str(1 + tag)}], "stale": 0,
            "settled_event": f"e{tag}", "records": [{"kind": "worker_settle", "event_id": f"e{tag}"}],
            "settle": {"adapter_profile": ADAPTER, "input_tokens": 10 + tag, "output_tokens": 20 + tag,
                       "identity_sha256": "i" * 64}}


def fields_fixture():
    return {"dirty": 0, "exe_sha256": "e" * 64, "engine_sha256": "e" * 64, "units_before": ["a.service"],
            "units_after": ["a.service"], "compensating": {"netns_differ": True}, "source_sha": "s" * 40,
            "release_manifest_sha256": "m" * 64, "shim_sha256": "h" * 64, "test_sha256": "t" * 64, "run_id": "r1",
            "ran_utc": "2026-09-29T00:00:00Z"}


def control():
    cases = []

    def case(name, got, want):
        if got != want:
            raise AssertionError(f"control case {name}: got {got!r}, want {want!r}")
        cases.append(name)

    # 1 · the four lines, whole, from two ledgers differing in every field.
    case("four lines", closure(ledger_fixture(0)), ([
        "F06 forward task=t0 attempts=1 terminal=reconciled each=1/1",
        "F06 generation task=t0 attempt_generation=1 task_generation=4 stale_accepted=0",
        "F06 execute attempt=a0 provider=native settled_event=e0 run_records=worker_settle",
        "F06 one_settle attempt=a0 usage_event=e0 identity_event=e0 same=true"], []))
    case("four lines, second fixture", closure(ledger_fixture(1))[0], [
        "F06 forward task=t1 attempts=1 terminal=reconciled each=1/1",
        "F06 generation task=t1 attempt_generation=2 task_generation=5 stale_accepted=0",
        "F06 execute attempt=a1 provider=native settled_event=e1 run_records=worker_settle",
        "F06 one_settle attempt=a1 usage_event=e1 identity_event=e1 same=true"])
    # 2 · an attempt neither reconciled nor an explicit unknown.
    running = ledger_fixture(0)
    running["attempts"][0]["state"] = "running"
    case("terminal", closure(running)[1],
         ["forward: an attempt with neither a reconciled terminal nor an explicit unknown (['running'])"])
    unknown = ledger_fixture(0)
    unknown["attempts"][0]["state"] = "unknown"
    case("explicit unknown is terminal", closure(unknown)[0][0],
         "F06 forward task=t0 attempts=1 terminal=unknown each=1/1")
    # 3 · a stale acceptance.
    stale = ledger_fixture(0)
    stale["stale"] = 1
    case("stale_accepted", closure(stale)[1], ["generation: stale_accepted=1"])
    # 4 · usage and identity keyed by another event than the settle.
    moved = ledger_fixture(0)
    moved["records"] = [{"kind": "worker_settle", "event_id": "e9"}]
    case("same=false", (closure(moved)[0][3], closure(moved)[1]),
         ("F06 one_settle attempt=a0 usage_event=e9 identity_event=e9 same=false",
          ["one_settle: usage e9 and identity e9 are not the settle e0"]))
    absent = ledger_fixture(0)
    absent["settle"] = {**absent["settle"], "identity_sha256": None}
    case("identity absent", closure(absent)[1], ["one_settle: usage e0 and identity - are not the settle e0"])
    untokened = ledger_fixture(0)
    untokened["settle"] = {**untokened["settle"], "output_tokens": None}
    case("usage absent", closure(untokened)[1], ["one_settle: usage - and identity e0 are not the settle e0"])
    other = ledger_fixture(0)
    other["settle"] = {**other["settle"], "adapter_profile": "openai-x"}
    case("provider not native", closure(other)[1], ["execute: the settle names no native adapter ('openai-x')"])
    lines = closure(ledger_fixture(0))[0]
    # 5 · a dirty tree; 6 · serve's exe not the manifest's; 7 · units left behind; R0 unobserved.
    for label, change, want in (
            ("dirty", {"dirty": 1}, ["dirty=1"]),
            ("exe digest", {"exe_sha256": "f" * 64}, [f"serve exe {'f' * 64} != manifest engine {'e' * 64}"]),
            ("units", {"units_after": ["a.service", "hee3-t3-x.service"]},
             ["units after != before: ['hee3-t3-x.service']"]),
            ("R0 netns", {"compensating": {"netns_differ": False}},
             ["R0: the candidate's network namespace was not observed apart from serve's"])):
        case(label, record({**fields_fixture(), **change}, lines, []), (None, want))
    made, refused = record(fields_fixture(), lines, [])
    case("record", (refused, made["asserted"], made["schema"], made["flow"]), ([], lines, RECORD_SCHEMA, "F06"))
    case("a closure problem refuses the record", record(fields_fixture(), lines, ["x"]), (None, ["x"]))
    # 8 · each P-row's FAIL stops before S1; the census is the row table's.
    every = {name: ("ok", True) for name in ("class_pins", "host_pins", "release", "state_root", "units",
                                             "foreign_serve", "service_probe", "control_socket", "native_daemon",
                                             "headroom")}
    printed, clear = rows(every)
    case("P-rows all pass", (printed[-1], clear), ("prows=10/10 S1 may start", True))
    for name in every:
        printed, clear = rows({**every, name: ("bad", False)})
        case(f"P-row {name} stops", (printed[-1], clear), ("prows=9/10 stop before S1", False))
    case("an unmeasured row fails", rows({})[0][0], "P0 class_pins UNMEASURED FAIL")
    # The written values: the grant's scope is its own digest; the profile carries measured pins only.
    written = grant("g", 1000, 5)
    case("grant", (json.loads(written)["owners"], json.loads(written)["effects"], sha(written)[:7]),
         (["task"], ["read", "durable admission"], "sha256:"))
    text = profile({"workspace_id": "w", "base_digest": "sha256:b", "protected_digest": "sha256:p",
                    "compiler": ("/r", "sha256:c"), "shim": ("/s", "sha256:h"),
                    "runtime_files": [("/h/lib", "/toolchain/lib/x", "sha256:x")], "busctl": "sha256:u",
                    "systemd_run": "sha256:v", "class_grant_id": "g2", "manifest_sha256": "sha256:m",
                    "model": "llama3.2:3b"})
    case("profile runtime row", '  { host = "/h/lib", namespace = "/toolchain/lib/x", sha256 = "sha256:x" },' in text,
         True)
    case("profile native row", text.endswith(f'[native]\nmodel = "llama3.2:3b"\nmanifest_sha256 = "sha256:m"\n'
                                             f'adapter = "{ADAPTER}"\n'), True)
    rules = ["four lines", "terminal", "stale_accepted", "same=false", "dirty", "exe digest", "units",
             "P-rows all pass", "R0 netns"]
    ran = sum(rule in cases for rule in rules)
    print(f"control verdict=PASS cases={ran}/{len(rules)} assertions={len(cases)}")
    return 0 if ran == len(rules) else 1


if __name__ == "__main__":
    if sys.argv[1:] != ["--control"]:
        sys.exit("usage: f06_decide.py --control")
    sys.exit(control())
