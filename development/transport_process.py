#!/usr/bin/env python3
"""Run eleven actual fake-process T02 controls using the existing TH-DEV supervisor.

This development driver tests pipes plus the explicit Framer/Session boundary.
It does not implement the production worker process owner, T06 custody, T15
hostile isolation, module admission, or external provider communication.
"""

from datetime import datetime, timezone
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import time

BASE = Path(__file__).resolve().parents[1]
TOOLCHAIN = Path('/var/home/Louranicas/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/bin')
BUNDLE = 'tools/rust-offline-inputs'
FINITE = ('fragmented-lf-utf8', 'coalesced-records', 'stderr-empty-benign',
          'stderr-warning-fault', 'truncated-eof')
SUPERVISED = ('timeout-fault', 'timeout-benign', 'stdout-flood-fault',
              'stdout-flood-benign', 'stderr-flood-fault', 'stderr-flood-benign')
STREAM_LIMIT = 16_384


def digest(path):
    with path.open('rb') as source:
        return hashlib.file_digest(source, 'sha256').hexdigest()


def describe(path, relative_to):
    return {'path': str(path.relative_to(relative_to)),
            'byte_length': path.stat().st_size, 'sha256': digest(path)}


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    origin = time.monotonic()
    deadline = origin + 300
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    output = BASE / 'evidence/implementation/T02/transport-controls' / stamp
    output.mkdir(parents=True, exist_ok=False)
    subject = output / 'subjects'
    subject.mkdir()
    report = {
        'format': 'hee3.t02-development-transport-controls/1',
        'scope': 'Eleven distinct TH-DEV fake-process controls; five adapter boundary controls and six process-supervisor fault/benign controls',
        'module_admission': False, 'protected_collector_qualification': False,
        'hostile_execution_qualification': False, 'production_transport_implemented': False,
        'provider_requests': 0, 'status': 'incomplete', 'inputs': {}, 'commands': [],
        'controls': [], 'wall_budget_seconds': 300,
        'limits': 'Retained stream bytes are capped separately. Timeout/flood controls intentionally refuse. The existing supervisor retains process-group/subreaper cleanup observations; same-UID local custody is not hostile proof.',
    }
    paths = ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
             'tests/t02_transport.rs', 'development/transport_process.py',
             'tests/fixtures/pi/records/unicode-delta.jsonl',
             'tests/fixtures/pi/records/state-empty.jsonl',
             'tools/development_process.py', 'tools/rust-offline.py']
    paths += [str(p.relative_to(BASE)) for p in sorted((BASE / 'src').rglob('*.rs'))]
    paths += [str(p.relative_to(BASE)) for p in sorted((BASE / BUNDLE).iterdir())]
    try:
        for name in paths:
            source = BASE / name
            if source.is_symlink() or not source.is_file():
                raise ValueError('nonregular subject: ' + name)
            target = subject / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
            report['inputs'][name] = describe(target, output)
            if digest(source) != report['inputs'][name]['sha256']:
                raise ValueError('subject changed during snapshot: ' + name)
        # Cargo reads ancestor configuration even with a private CARGO_HOME.
        for ancestor in (subject, *subject.parents):
            if any((ancestor / '.cargo' / name).exists() for name in ('config', 'config.toml')):
                raise ValueError('unexpected ancestor Cargo configuration: ' + str(ancestor))
        supervisor = load_module('t02_development_process', subject / 'tools/development_process.py')
        offline = load_module('t02_rust_offline', subject / 'tools/rust-offline.py')
        vendor = output / 'rust-vendor'
        cargo_home = output / 'cargo-home'
        report['offline_dependencies'] = offline.materialize(
            subject / 'Cargo.lock', subject / BUNDLE, vendor, cargo_home, deadline)
        binaries = {name: TOOLCHAIN / name for name in
                    ('rustc', 'rustdoc', 'cargo', 'rustfmt', 'cargo-clippy', 'clippy-driver')}
        report['tool_executables'] = {
            name: {'path': str(path), 'sha256': digest(path)} for name, path in binaries.items()}
        report['python'] = {'path': sys.executable, 'sha256': digest(Path(sys.executable))}
        environment = {
            'PATH': str(TOOLCHAIN) + ':/usr/bin:/bin', 'LC_ALL': 'C',
            'TMPDIR': str(output), 'CARGO_HOME': str(cargo_home),
            'CARGO_TARGET_DIR': str(output / 'target'), 'CARGO_BUILD_JOBS': '2',
            'CARGO_NET_OFFLINE': 'true', 'RUSTUP_TOOLCHAIN': '1.98.0',
            'RUSTC': str(binaries['rustc']), 'RUSTDOC': str(binaries['rustdoc']),
            'RUSTFMT': str(binaries['rustfmt']), 'RUSTFLAGS': '-Dwarnings',
            'RUSTDOCFLAGS': '-Dwarnings',
        }
        report['environment'] = environment
        expectations = [
            {'id': name, 'producer_exit': 0, 'control_status': 'pass',
             'transport_refusal': {'stderr-warning-fault': 'stderr',
                                   'truncated-eof': 'truncated-eof'}.get(name)}
            for name in FINITE
        ] + [
            {'id': name, 'producer_exit': 'signal' if name.endswith('-fault') else 0,
             'timed_out': name == 'timeout-fault',
             'output_limit_exceeded': 'flood-fault' in name,
             'cleanup_complete': True}
            for name in SUPERVISED
        ]
        expected_path = output / 'predeclared-controls.json'
        expected_path.write_text(json.dumps(expectations, indent=2) + '\n')
        report['predeclared_controls'] = describe(expected_path, output)

        def run(label, argv, timeout=90, limit=2 * 1024 * 1024):
            stdout = output / (label + '.stdout')
            stderr = output / (label + '.stderr')
            row = {'id': label, 'argv': [str(x) for x in argv], 'cwd': str(subject),
                   'command_timeout_seconds': timeout, 'stream_limit_each': limit}
            report['commands'].append(row)
            try:
                row.update(supervisor.run_bounded(
                    row['argv'], subject, environment, stdout, stderr, deadline,
                    command_timeout=timeout, stream_limit=limit))
            except supervisor.ProcessSupervisionError as error:
                row.update(error.result)
                raise
            finally:
                for name, path in (('stdout', stdout), ('stderr', stderr)):
                    if path.is_file():
                        row[name] = describe(path, output)
            return row

        def require_clean(row):
            if (row['exit_code'] != 0 or row['timed_out'] or row['output_limit_exceeded']
                    or not row['cleanup_complete']):
                raise ValueError('bounded producer failed: ' + row['id'])

        version = run('rustc-version', [binaries['rustc'], '--version', '--verbose'])
        require_clean(version)
        if 'release: 1.98.0\n' not in (output / 'rustc-version.stdout').read_text():
            raise ValueError('unexpected compiler version')
        formatting = run('rustfmt', [binaries['rustfmt'], '--check', '--edition', '2024',
                                     'tests/t02_transport.rs'])
        require_clean(formatting)
        lint = run('clippy', [binaries['cargo'], 'clippy', '--offline', '--locked',
                             '--lib', '--test', 't02_transport', '--',
                             '-Dwarnings', '-Wclippy::pedantic'])
        require_clean(lint)
        build = run('build', [binaries['cargo'], 'test', '--offline', '--locked',
                             '--test', 't02_transport', '--no-run', '--message-format=json'])
        require_clean(build)
        artifacts = [json.loads(line) for line in (output / 'build.stdout').read_text().splitlines()]
        executables = [Path(item['executable']) for item in artifacts
                       if item.get('reason') == 'compiler-artifact' and item.get('executable')
                       and item['target']['name'] == 't02_transport']
        if len(executables) != 1:
            raise ValueError('expected one actual transport executable')
        binary = executables[0]
        report['test_executable'] = describe(binary, output)

        for expected in expectations[:len(FINITE)]:
            name = expected['id']
            row = run(name, [binary, '--case', name], timeout=10, limit=65_536)
            require_clean(row)
            if row['stream_bytes']['stderr_observed']:
                raise ValueError('unexpected driver stderr: ' + name)
            observed = json.loads((output / (name + '.stdout')).read_bytes())
            if (observed['case'] != name or observed['status'] != 'pass'
                    or observed['transport_refusal'] != expected['transport_refusal']
                    or observed['child_exit_code'] != 0 or observed['module_admission'] is not False):
                raise ValueError('finite control mismatch: ' + name)
            for field in ('request', 'worker_stdout', 'worker_stderr'):
                path = output / (name + '.' + field)
                path.write_bytes(bytes.fromhex(observed.pop(field + '_hex')))
                observed[field] = describe(path, output)
            report['controls'].append(observed)

        for expected in expectations[len(FINITE):]:
            name = expected['id']
            row = run(name, [binary, '--supervised-case', name],
                      timeout=0.75 if name == 'timeout-fault' else 5,
                      limit=STREAM_LIMIT)
            for key in ('timed_out', 'output_limit_exceeded', 'cleanup_complete'):
                if row[key] != expected[key]:
                    raise ValueError('supervisor outcome mismatch: ' + name + ':' + key)
            if expected['producer_exit'] == 'signal':
                if row['exit_code'] is None or row['exit_code'] >= 0:
                    raise ValueError('fault process did not terminate by signal: ' + name)
            elif row['exit_code'] != 0:
                raise ValueError('benign process failed: ' + name)
            stdout = (output / (name + '.stdout')).read_bytes()
            stderr = (output / (name + '.stderr')).read_bytes()
            if name == 'timeout-fault' and (stdout != b'{"type":"agent_start"}' or stderr):
                raise ValueError('timeout did not retain exact partial frame')
            if name == 'timeout-benign' and (stdout != b'{"type":"agent_start"}\n{"type":"agent_settled"}\n' or stderr):
                raise ValueError('timeout benign stream mismatch')
            if 'flood' in name:
                active, other = (stdout, stderr) if name.startswith('stdout') else (stderr, stdout)
                length = STREAM_LIMIT if name.endswith('-fault') else 8192
                stream = name.split('-')[0]
                if active != b'x' * length or other:
                    raise ValueError('flood capture mismatch: ' + name)
                if name.endswith('-fault') and row['stream_bytes'][stream + '_observed'] <= STREAM_LIMIT:
                    raise ValueError('flood never crossed stream cap: ' + name)
            report['controls'].append({'case': name, 'status': 'pass',
                                       'expectation': expected, 'producer': row['id'],
                                       'module_admission': False})

        default = run('finite-default', [binary], timeout=10, limit=65_536)
        require_clean(default)
        lines = (output / 'finite-default.stdout').read_text().splitlines()
        if (len(lines) != 6 or lines[-1] != 'T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only'
                or [json.loads(line)['case'] for line in lines[:-1]] != list(FINITE)):
            raise ValueError('default harness omitted or added a finite control')
        offline.verify_materialized(vendor, cargo_home, report['offline_dependencies'], deadline)
        for name, identity in report['inputs'].items():
            if digest(subject / name) != identity['sha256']:
                raise ValueError('archived subject changed during execution: ' + name)
        report['live_input_drift'] = [name for name, identity in report['inputs'].items()
                                      if digest(BASE / name) != identity['sha256']]
        # Historical observations remain bound to the exact copied subject. Live
        # edits are disclosed and need a new run before applying the observation.
        report['status'] = 'pass' if not report['live_input_drift'] else 'pass archived subject; live inputs changed'
        report['distinct_controls'] = len(report['controls'])
    except BaseException as error:
        report['status'] = 'failed'
        report['error'] = type(error).__name__ + ': ' + str(error)
        raise
    finally:
        report['elapsed_seconds'] = time.monotonic() - origin
        (output / 'results.json').write_text(json.dumps(report, indent=2) + '\n')
        print(str(output / 'results.json'), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
