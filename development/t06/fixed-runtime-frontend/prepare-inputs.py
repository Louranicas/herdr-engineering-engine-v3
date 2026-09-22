#!/usr/bin/env python3
"""Freeze fixed development inputs; never launch probes, candidates or providers."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import uuid

BASE = Path(__file__).resolve().parent
T06 = Path('/var/home/Louranicas/.cache/hee3-implementation/T06')  # retained reviewed external tooling; not copied engine state
PROJECT = BASE.parents[2]
RUNTIME = BASE.parent/'task-runtime'
ATLAS = Path('/var/home/Louranicas/planning/herdr-engine-vision-20260915')
MAX_FILE = 16 * 1024 * 1024

def sha(data):
    return 'sha256:' + hashlib.sha256(data).hexdigest()

def pin(path):
    path = path.resolve(strict=True)
    before = path.stat()
    if not stat.S_ISREG(before.st_mode):
        raise ValueError('only real regular inputs')
    with path.open('rb') as stream:
        h = hashlib.sha256()
        while chunk := stream.read(65536):
            h.update(chunk)
    after = path.stat()
    fields = lambda s: (s.st_dev, s.st_ino, s.st_mode, s.st_size, s.st_mtime_ns, s.st_ctime_ns)
    if fields(before) != fields(after):
        raise ValueError('input changed while hashing')
    return {'path': str(path), 'sha256': 'sha256:' + h.hexdigest(), 'bytes': before.st_size}

def write(path, data, executable=False):
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    with path.open('xb') as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())
    path.chmod(0o500 if executable else 0o400)
    return pin(path)

def copy(source, target, executable=False):
    original = pin(source)
    if original['bytes'] > MAX_FILE:
        raise ValueError('raw subject file exceeds 16 MiB')
    data = source.resolve(strict=True).read_bytes()
    if sha(data) != original['sha256']:
        raise ValueError('copy source changed')
    frozen = write(target, data, executable)
    if frozen['sha256'] != original['sha256'] or pin(source) != original:
        raise ValueError('copy readback drift')
    return frozen

def tree(root):
    rows = []
    directories = []
    total = 0
    for p in sorted(root.rglob('*'), key=lambda p: str(p.relative_to(root)).encode('utf8')):
        st = p.lstat()
        if p.is_symlink() or not (stat.S_ISDIR(st.st_mode) or stat.S_ISREG(st.st_mode)):
            raise ValueError('role includes alias/special')
        if p.is_dir(): directories.append(str(p.relative_to(root)))
        if p.is_file():
            info = pin(p)
            total += info['bytes']
            rows.append({'path': str(p.relative_to(root)), 'sha256': info['sha256'], 'bytes': info['bytes'], 'executable': bool(st.st_mode & 0o111)})
    if not rows or len(rows) > 4096 or total > 64 * 1024 * 1024:
        raise ValueError('role inventory bound')
    return {'path': str(root), 'directories': directories, 'files': rows}

def copy_sources(source, target):
    for f in sorted(source.rglob('*')):
        if f.is_symlink(): raise ValueError('source alias')
        dest = target / f.relative_to(source)
        if f.is_dir(): dest.mkdir(mode=0o700, parents=True, exist_ok=True)
        elif f.is_file(): copy(f, dest)
        else: raise ValueError('special source input')

def new_id():
    return str(uuid.uuid4())

def identities():
    return {**{k: new_id() for k in ['task', 'store_generation', 'store_epoch', 'submit_key', 'submit_event', 'roster_key']},
            'attempts': [{**{k: new_id() for k in ['run', 'attempt', 'session', 'workspace']}, 'stages': [new_id() for _ in range(3)]} for _ in range(2)]}

def reviewed(path):
    actual = pin(path)
    if actual['sha256'] != 'sha256:830ec81df2258a1774e6420acf2ec2f28fe3443ecbb9655e9e60dde8afbefcb3':
        raise ValueError('changed reviewed input bundle')
    data = json.loads(path.read_bytes())
    if len(data['objects']) != 22:
        raise ValueError('review object count')
    seen = set()
    for row in data['objects']:
        ref = row['reference']; raw = bytes.fromhex(row['hex'])
        if ref['artifact_id'] in seen or sha(raw) != ref['sha256'] or len(raw) != ref['byte_length']:
            raise ValueError('review object identity')
        seen.add(ref['artifact_id'])
    return data

def current_build_sources():
    sources=[]
    for root in [BASE/'src', BASE/'tests', PROJECT/'src', PROJECT/'migrations',
                 RUNTIME/'src', RUNTIME/'tests', PROJECT/'tests']:
        sources.extend(pin(p) for p in sorted(root.rglob('*')) if p.is_file())
    for root,names in [(BASE,['Cargo.toml','Cargo.lock','prepare-inputs.py','write-build-record.py','compare-result.py']),
                       (PROJECT,['Cargo.toml','Cargo.lock']),(RUNTIME,['Cargo.toml','Cargo.lock'])]:
        sources.extend(pin(root/name) for name in names)
    return sources

def build_inputs(path):
    data = json.loads(path.read_bytes())
    if data['kind'] != 'hee3-fixed-frontend-build/1' or data.get('completed') is not True:
        raise ValueError('completed exact frontend build required')
    if data['binary'] != pin(Path(data['binary']['path'])):
        raise ValueError('build binary drift')
    if data.get('sources') != current_build_sources():
        raise ValueError('build sources do not match the current workspace')
    for row in data['sources']:
        if pin(Path(row['path'])) != row:
            raise ValueError('compiled source drift')
    commands = data['commands']
    if len(commands) != 4 or {row['role'] for row in commands} != {'build','test','clippy','fmt'}:
        raise ValueError('exact build/check roles required')
    observed_inputs = {}
    for command in commands:
        raw = {}
        for name in ['result','command','inputs','stdout','stderr']:
            record = command[name]
            record_path = Path(record['path'])
            if record != pin(record_path):
                raise ValueError('build record identity: ' + name)
            raw[name] = record_path.read_bytes()
        result = json.loads(raw['result'])
        if (type(result.get('exit_code')) is not int or result['exit_code'] != 0
                or result.get('started') is not True
                or result.get('cleanup_complete') is not True
                or result.get('timed_out') is not False
                or result.get('output_limit_exceeded') is not False
                or result.get('postrun_source_drift') != []
                or result.get('postrun_tool_dependency_drift') != []):
            raise ValueError('build producer incomplete or drifted')
        for name, field in [('command','actual_invocation_hex'),('result','actual_result_hex'),
                            ('stdout','actual_stdout_hex'),('stderr','actual_stderr_hex')]:
            if command[field] != raw[name].hex():
                raise ValueError('retained build record mismatch: ' + name)
        for name in ['stdout','stderr']:
            if result[name + '_sha256'] != hashlib.sha256(raw[name]).hexdigest():
                raise ValueError('producer stream identity: ' + name)
        if raw['stderr']:
            raise ValueError('unexpected baseline stderr')
        invocation = json.loads(raw['command'])
        argv = invocation['argv']
        if not isinstance(argv,list) or len(argv) < 2 or argv[1] != command['role']:
            raise ValueError('wrong command role')
        inputs = json.loads(raw['inputs'])
        if not isinstance(inputs,dict) or not inputs:
            raise ValueError('build inputs missing')
        for name, recorded in inputs.items():
            actual = pin(Path(name))
            if actual['sha256'] != 'sha256:' + recorded['sha256'] or actual['bytes'] != recorded['bytes']:
                raise ValueError('build input drift: ' + name)
        observed_inputs[sha(raw['inputs'])] = {'exact_hex': raw['inputs'].hex()}
    if data['input_observations'] != observed_inputs:
        raise ValueError('build input observation mismatch')
    # Revalidate the exact current writer contract from retained observations only.
    # The writer imports this helper for hashing; neither module runs commands.
    import importlib.util
    spec = importlib.util.spec_from_file_location('current_build_record', BASE/'write-build-record.py')
    writer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(writer)
    directories = {row['role']: Path(row['result']['path']).parent for row in commands}
    roots = {directory.parent for directory in directories.values()}
    if len(roots) != 1:
        raise ValueError('build commands use different record roots')
    packaging = data.get('packaging')
    record = Path(packaging['record']['path']) if packaging else None
    record_sha = packaging['record']['sha256'] if packaging else None
    replay = writer.collect({role: directory.name for role, directory in directories.items()},
                            roots.pop(), Path(data['binary']['path']), record, record_sha)
    if replay != data:
        raise ValueError('current build record contract mismatch')
    return data

def freeze(output, build_report, runtime_dir):
    build = build_inputs(build_report)
    runtime_dir = runtime_dir.resolve(strict=True)
    if not runtime_dir.is_dir() or runtime_dir.stat().st_uid != os.geteuid():
        raise ValueError('actual owned runtime directory required')
    output = output.absolute()
    if output.exists() or output.parent.resolve(strict=True) != output.parent:
        raise ValueError('fresh canonical output required')
    output.mkdir(mode=0o700)
    role_root = output / 'roles'; role_root.mkdir(mode=0o700)
    roles = {n: role_root / n for n in ['baseline','repaired','protected','fixtures','oracle','harness','collector','launcher']}
    for d in roles.values(): d.mkdir(mode=0o700)
    workload = PROJECT / 'evaluation/tasks/WL-U64-PARSE-001/v1'
    for name, lib in [('baseline', workload/'base/src/lib.rs'), ('repaired', workload/'reference/src/lib.rs')]:
        copy(workload/'base/Cargo.toml',roles[name]/'Cargo.toml')
        copy(lib, roles[name]/'src/lib.rs')
    original_oracle = workload/'oracle/cases.json'
    if not original_oracle.is_file():
        raise ValueError('authoritative oracle/cases.json missing')
    wrapper = T06/'workload-integration/protected/public-wrapper.rs'
    copy(original_oracle,roles['protected']/'oracle.json'); copy(wrapper,roles['protected']/'public-wrapper.rs')
    copy(original_oracle,roles['fixtures']/'cases.json')
    copy(workload/'TASK.md',roles['fixtures']/'TASK.md')
    copy(original_oracle,roles['oracle']/'cases.json')
    copy(PROJECT/'src/check/u64_oracle.rs',roles['oracle']/'u64_oracle.rs')
    copy(wrapper,roles['harness']/'public-wrapper.rs')
    for package, dest in [(PROJECT/'src','candidate-src'),(RUNTIME/'src','runtime-src'),(BASE/'src','frontend-src')]:
        copy_sources(package,roles['collector']/dest)
    for origin,target in [(BASE/'Cargo.toml','frontend-Cargo.toml'),(BASE/'Cargo.lock','frontend-Cargo.lock'),(RUNTIME/'Cargo.toml','runtime-Cargo.toml'),(PROJECT/'Cargo.toml','engine-Cargo.toml')]:
        copy(origin,roles['collector']/target)
    executor = copy(Path(build['binary']['path']),roles['collector']/'fixed-runtime-frontend',True)
    copy(PROJECT/'src/worker/namespace_shim.rs',roles['launcher']/'namespace_shim.rs')
    copy(PROJECT/'src/bin/namespace_shim.rs',roles['launcher']/'bin-entry.rs')
    shim = T06/'task-runtime/tools/namespace-shim'
    shim_provenance = T06/'task-runtime/tools/shim-input.json'
    shim_info = json.loads(shim_provenance.read_bytes())
    actual_shim = pin(shim)
    if actual_shim['sha256'] != 'sha256:a59c8bc2b60f97cccac0e0543b58538919f3c3a91e1f4e11ad7b518d8f756009':
        raise ValueError('current approved FD7 shim mismatch')
    for rel, info in shim_info['sources'].items():
        actual = pin(PROJECT/rel)
        if actual['sha256'] != 'sha256:' + info['sha256'] or actual['bytes'] != info['bytes']:
            raise ValueError('shim source/provenance mismatch')
    frozen_shim = copy(shim,roles['launcher']/'namespace-shim',True)
    support = output/'support';support.mkdir(mode=0o700)
    assets = {}
    sources = {'schema':PROJECT/'schemas/receipts/receipt-v1.schema.json','readiness':ATLAS/'READINESS_CONVENTION_habitat_engine.json',
               'contracts':PROJECT/'docs/contract-decisions.md','cargo_lock':PROJECT/'Cargo.lock','runtime_lock':BASE/'Cargo.lock',
               'build':build_report,'finite_files':T06/'namespace-link/evidence/finite-files.json','review':T06/'task-runtime/reviewed-inputs.json',
               'patch':workload/'reference.patch'}
    reviewed(sources['review'])
    for name, source in sources.items(): assets[name]=copy(source,support/name)
    controller = {'shim_input':shim_info,'shim_build_result':json.loads(Path(shim_info['build_record']).read_bytes()),'frontend_build':build,
                  'namespace_source':pin(roles['collector']/'candidate-src/worker/namespace.rs'),'shim_source':pin(roles['launcher']/'namespace_shim.rs')}
    assets['shim_build']=write(support/'shim_build',json.dumps(controller,separators=(',',':')).encode())
    declarations = {
      'authority':{'kind':'fixed-u64-authority/1','origin':'Luke authorized start coding, continued scoped implementation and zero-touch deployment in active root conversation','profile':'T06-u64-fixed-runtime-THDEV','external_requests':0,'external_cost':0,'tool_execution':'fixed compiler/linker/workload under bounded namespace only'},
      'isolation':{'kind':'fixed-u64-isolation/1','source':'compiled namespace controller and current FD7 shim; actual enforcement reported by runtime','candidate_cpu_percent':200,'candidate_memory_bytes':8589934592,'candidate_swap_bytes':0,'candidate_pids':128,'aggregate_cpu_percent':400,'aggregate_memory_bytes':17179869184,'aggregate_pids':256,'scratch_bytes':4294967296,'stdout_bytes':8388608,'stderr_bytes':8388608,'source_mode':'read-only frozen roles; separate writable tmpfs'},
      'cleanup':{'kind':'fixed-u64-cleanup/1','origin':'single TaskClock from admission/preparation','candidate_cutoff_ms':900000,'verification_cutoff_ms':1190000,'deadline_ms':1200000,'term_grace_ms':5000,'requires':'native process/channel/scratch and aggregate stopped/direct-empty observations; unknown is not settled'} }
    for name,value in declarations.items(): assets[name]=write(support/name,json.dumps(value,separators=(',',':')).encode())
    finite=json.loads(sources['finite_files'].read_bytes()); projections=[];seen=set()
    for entry in finite['entries']:
        ns=entry['namespace']
        if ns.startswith('/frozen/') or ns in ['/toolchain/bin/rustc','/shim/namespace-shim']: continue
        if ns in seen: raise ValueError('duplicate namespace target')
        seen.add(ns);projections.append({'input':pin(Path(entry['host'])),'namespace':ns})
    compiler={'input':pin(Path('/var/home/Louranicas/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/bin/rustc')),'namespace':'/toolchain/bin/rustc'}
    directories=json.loads((T06/'namespace-link/evidence/namespace-directories.json').read_bytes())
    directories=sorted(set([d for d in directories if not d.startswith('/work/')] + ['/toolchain/bin']))
    manifest={'kind':'hee3-fixed-u64-frontend/1','identities':identities(),'roles':{name:tree(path) for name,path in roles.items()},
              'tools':{'bwrap':pin(Path('/usr/bin/bwrap')),'compiler':compiler,'shim':{'input':frozen_shim,'namespace':'/shim/namespace-shim'},
                       'systemd_run':pin(Path('/usr/bin/systemd-run')),'busctl':pin(Path('/usr/bin/busctl')),'uname':pin(Path('/usr/bin/uname')),
                       'runtime_files':projections,'namespace_directories':directories},'assets':assets,'executor':executor,'runtime_dir':str(runtime_dir),
              'owner_id':'Luke','namespace_source_sha256':pin(roles['collector']/'candidate-src/worker/namespace.rs')['sha256'],
              'shim_source_sha256':pin(roles['launcher']/'namespace_shim.rs')['sha256'],'build_target':'x86_64-unknown-linux-gnu','build_profile':'release','rust_flags':['-Dwarnings']}
    frozen=write(output/'manifest.json',json.dumps(manifest,indent=2).encode()+b'\n')
    write(output/'freeze-provenance.json',json.dumps({'source_build_report':pin(build_report),'manifest':frozen,'scope':'input copying/hashing only; no probes, execution, outcome or review authored'},indent=2).encode())
    print(json.dumps({'manifest':frozen,'run_argv':[executor['path'],'run',str(output/'manifest.json'),'<fresh-absolute-output>']}))

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output',type=Path,required=True);p.add_argument('--build-report',type=Path,required=True);p.add_argument('--runtime-dir',type=Path,required=True)
    a=p.parse_args();freeze(a.output,a.build_report,a.runtime_dir)
