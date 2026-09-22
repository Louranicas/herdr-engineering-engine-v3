#!/usr/bin/env python3
"""Bind already completed bounded build/check records; run no commands."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
import tomllib
sys.dont_write_bytecode = True
BASE=Path(__file__).resolve().parent
PROJECT=BASE.parents[2]
RUNTIME=BASE.parent/'task-runtime'
spec=importlib.util.spec_from_file_location('fixed_input_freezer',BASE/'prepare-inputs.py')
freezer=importlib.util.module_from_spec(spec);spec.loader.exec_module(freezer)

def configured_profile():
    expected = {
        'dev': {'opt-level': 0, 'debug-assertions': True, 'overflow-checks': True, 'panic': 'unwind'},
        'test': {'opt-level': 0, 'debug-assertions': True, 'overflow-checks': True},
        'release': {'opt-level': 2, 'overflow-checks': True, 'panic': 'unwind',
                    'lto': 'thin', 'codegen-units': 1, 'debug': 'line-tables-only'},
    }
    for root in (PROJECT, RUNTIME, BASE):
        profiles = tomllib.loads((root/'Cargo.toml').read_text()).get('profile', {})
        for name, fields in expected.items():
            actual = profiles.get(name, {})
            if any(type(actual.get(key)) is not type(value) or actual[key] != value
                   for key, value in fields.items()):
                raise ValueError('RC02 workspace profile mismatch: '+str(root)+' '+name)
        if profiles['release'].get('strip', 'none') != 'none':
            raise ValueError('RC02 release debug information must be retained')
        allowed = {name: dict(fields) for name, fields in expected.items()}
        if root == RUNTIME:
            allowed['test']['debug'] = 0
        if profiles != allowed or (root == RUNTIME and type(profiles['test'].get('debug')) is not int):
            raise ValueError('unreviewed workspace profile override: '+str(root))
    return {'target':'x86_64-unknown-linux-gnu','profile':'release','rust_flags':['-Dwarnings'],
            'opt_level':2,'overflow_checks':True,'panic':'unwind','lto':'thin',
            'codegen_units':1,'debug':'line-tables-only','strip':'none',
            'basis':'validated declared workspace profiles; effective compiler arguments require separate retained build evidence'}

def validate_build_configuration(command, inputs, action):
    """Accept only the recorded, replacement-environment owned runner contract."""
    tool = Path('/var/home/Louranicas/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/bin')
    sqlite = freezer.T06.parent/'T05/sqlite-inputs'
    fixed = {
        'PATH': str(tool)+':/usr/bin:/bin', 'LANG': 'C', 'LC_ALL': 'C',
        'CARGO_NET_OFFLINE': 'true', 'RUSTC': str(tool/'rustc'),
        'RUSTDOC': str(tool/'rustdoc'), 'RUSTUP_TOOLCHAIN': '1.98.0',
        'CARGO_BUILD_JOBS': '2', 'RUST_TEST_THREADS': '2',
        'RUSTFLAGS': '-Dwarnings', 'RUSTDOCFLAGS': '-Dwarnings',
        'SQLITE3_NO_PKG_CONFIG': '1', 'SQLITE3_STATIC': '1',
        'SQLITE3_LIB_DIR': str(sqlite), 'SQLITE3_INCLUDE_DIR': str(sqlite),
        'LIBSQLITE3_SYS_USE_PKG_CONFIG': '0', 'PYTHONDONTWRITEBYTECODE': '1',
    }
    environment = command.get('environment', {})
    if (set(environment) != set(fixed) | {'CARGO_HOME', 'CARGO_TARGET_DIR', 'TMPDIR'}
            or any(environment[key] != value for key, value in fixed.items())
            or any(not isinstance(environment[key], str) or not Path(environment[key]).is_absolute()
                   for key in ('CARGO_HOME', 'CARGO_TARGET_DIR', 'TMPDIR'))):
        raise ValueError('unreviewed compiler/profile environment override')
    argv = [str(tool/'cargo'), action]
    if action == 'fmt':
        argv += ['--', '--check']
    else:
        argv += ['--offline', '--locked', '--quiet', '--target-dir', environment['CARGO_TARGET_DIR']]
        argv += ['--release'] if action == 'build' else ['--all-targets']
        if action == 'clippy':
            argv += ['--', '-D', 'warnings', '-W', 'clippy::pedantic']
    if command.get('argv') != argv or command.get('cwd') != str(BASE):
        raise ValueError('unreviewed compiler/profile command override')
    home = Path(environment['CARGO_HOME'])
    configs = [root/'.cargo'/name for root in (BASE, *BASE.parents)
               for name in ('config', 'config.toml')]
    configs.extend(home/name for name in ('config', 'config.toml'))
    expected = {'net': {'offline': True}, 'source': {
        'crates-io': {'replace-with': 'hee3-closed'},
        'hee3-closed': {'directory': str(freezer.T06.parent/'T05/candidate/rust-vendor')},
    }}
    if not (home/'config.toml').is_file():
        raise ValueError('missing reviewed offline Cargo configuration')
    for path in configs:
        if path.is_symlink():
            raise ValueError('Cargo configuration alias')
        if not path.exists():
            continue
        raw = path.read_bytes()
        pin = inputs.get(str(path))
        if pin != {'sha256': freezer.sha(raw).removeprefix('sha256:'), 'bytes': len(raw)}:
            raise ValueError('unbound Cargo configuration: '+str(path))
        if tomllib.loads(raw.decode('utf-8')) != expected:
            raise ValueError('unreviewed Cargo configuration override: '+str(path))

def validate_binary_derivation(command, result, binary, packaging_record, packaging_sha256):
    if command is None or result is None:
        raise ValueError('successful build observation required')
    raw_path = Path(command['environment']['CARGO_TARGET_DIR'])/'release/hee3-fixed-runtime-frontend'
    raw = freezer.pin(raw_path)
    if result.get('built_binary') != raw:
        raise ValueError('binary was not bound at build completion')
    if packaging_record is None:
        if packaging_sha256 is not None or binary != raw:
            raise ValueError('binary is not the observed build output')
        return {}
    record_pin = freezer.pin(packaging_record)
    if record_pin['sha256'] != packaging_sha256:
        raise ValueError('changed reviewed packaging record')
    raw_record = packaging_record.read_bytes()
    if freezer.sha(raw_record) != packaging_sha256:
        raise ValueError('packaging record changed while reading')
    record = json.loads(raw_record)
    if (record.get('kind') != 'hee3-elf-debug-compression/1' or record.get('completed') is not True
            or record.get('source') != raw or record.get('binary') != binary):
        raise ValueError('packaging does not derive from the observed build output')
    for pin in [record['source_snapshot'], record['tool'], record['verification']['report'],
                record['verification']['verifier']]:
        if freezer.pin(Path(pin['path'])) != pin:
            raise ValueError('packaging evidence drift')
    if record['source_snapshot']['sha256'] != raw['sha256'] or record['source_snapshot']['bytes'] != raw['bytes']:
        raise ValueError('packaging source snapshot differs')
    if record['verification'].get('passed') is not True or not record.get('commands'):
        raise ValueError('packaging proof incomplete')
    for row in record['commands']:
        for pin in row.values():
            if freezer.pin(Path(pin['path'])) != pin:
                raise ValueError('packaging command evidence drift')
        outcome = json.loads(Path(row['result']['path']).read_bytes())
        if (type(outcome.get('exit_code')) is not int or outcome['exit_code'] != 0
                or outcome.get('started') is not True or outcome.get('cleanup_complete') is not True
                or outcome.get('timed_out') is not False or outcome.get('output_limit_exceeded') is not False
                or outcome.get('supervision_error') is not None):
            raise ValueError('packaging command incomplete')
        if Path(row['stderr']['path']).read_bytes():
            raise ValueError('packaging command diagnostics')
    return {'packaging': {'record': record_pin, 'exact_hex': raw_record.hex(),
                          'debug_compression': 'zlib',
                          'scope': 'bound package transformation observations; independent ELF review and native execution remain separate'}}

def collect(labels, records_root, binary_path, packaging_record=None, packaging_sha256=None):
    profile=configured_profile()
    records_root=records_root.resolve(strict=True)
    binary_path=binary_path.resolve(strict=True)
    commands=[]
    build_command=None
    build_result=None
    input_observations={}
    expected_actions={'build':'build','clippy':'clippy','test':'test','fmt':'fmt'}
    for action,label in labels.items():
        directory=records_root/label
        result=json.loads((directory/'result.json').read_bytes())
        command=json.loads((directory/'command.json').read_bytes())
        inputs=json.loads((directory/'inputs.json').read_bytes())
        validate_build_configuration(command, inputs, action)
        if action == 'build':
            build_command, build_result = command, result
        if (type(result.get('exit_code')) is not int or result['exit_code']!=0
                or result.get('started') is not True
                or result.get('cleanup_complete') is not True or result.get('timed_out') is not False
                or result.get('output_limit_exceeded') is not False
                or result.get('postrun_source_drift') != [] or result.get('postrun_tool_dependency_drift') != []):
            raise ValueError('command did not complete successfully')
        for stream in ['stdout','stderr']:
            actual=freezer.pin(directory/stream)
            if actual['sha256'] != 'sha256:'+result[stream+'_sha256']:
                raise ValueError('raw stream identity')
        if (directory/'stderr').read_bytes():
            raise ValueError('unexpected baseline stderr')
        if not any(expected_actions[action] in arg for arg in command['argv']):
            raise ValueError('wrong command role')
        for name,pin in inputs.items():
            p=Path(name)
            actual=freezer.pin(p)
            if actual['sha256']!='sha256:'+pin['sha256'] or actual['bytes']!=pin['bytes']:
                raise ValueError('post-command source/tool drift: '+name)
        input_observations[freezer.sha((directory/'inputs.json').read_bytes())]={'exact_hex':(directory/'inputs.json').read_bytes().hex()}
        commands.append({'actual_invocation_hex':(directory/'command.json').read_bytes().hex(),'actual_result_hex':(directory/'result.json').read_bytes().hex(),'actual_stdout_hex':(directory/'stdout').read_bytes().hex(),'actual_stderr_hex':(directory/'stderr').read_bytes().hex(),'role':action,'result':freezer.pin(directory/'result.json'),'command':freezer.pin(directory/'command.json'),
                         'inputs':freezer.pin(directory/'inputs.json'),'stdout':freezer.pin(directory/'stdout'),'stderr':freezer.pin(directory/'stderr')})
    sources=freezer.current_build_sources()
    binary=freezer.pin(binary_path)
    derivation=validate_binary_derivation(build_command, build_result, binary, packaging_record, packaging_sha256)
    if binary['bytes']>freezer.MAX_FILE:raise ValueError('collector binary exceeds raw bound')
    return {'kind':'hee3-fixed-frontend-build/1','completed':True,'binary':binary,'sources':sources,'commands':commands,'input_observations':input_observations,
            'scope':'local source-bound development compiler/test observations; no runtime execution, review authority or module admission',
            'profile':profile, **derivation}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for key in ['build','clippy','test','fmt']:p.add_argument('--'+key,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--records-root',type=Path,required=True)
    p.add_argument('--binary',type=Path,required=True)
    p.add_argument('--packaging-record',type=Path)
    p.add_argument('--packaging-sha256')
    a=p.parse_args();data=collect({key:getattr(a,key) for key in ['build','clippy','test','fmt']},a.records_root,a.binary,a.packaging_record,a.packaging_sha256)
    freezer.write(a.output,json.dumps(data,indent=2).encode()+b'\n');print(json.dumps(freezer.pin(a.output)))
