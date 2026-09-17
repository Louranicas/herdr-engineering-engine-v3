#!/usr/bin/env python3
"""Prepare/check a closed Cargo archive bundle; never fetch or use an index.

Setup explicitly copies lock-selected .crate archives from one local cache.
Quality runs consume only the copied bundle and expand a private directory source.
This is TH-DEV input integrity, not hostile-build or network custody qualification.
"""

import argparse
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import tarfile
import time
import tomllib


REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'
PACKAGE_LIMIT = 256
FILE_LIMIT = 4096
MEMBER_LIMIT = 8192
ENTRY_LIMIT = 16384
BYTE_LIMIT = 512 * 1024 * 1024
ARCHIVE_LIMIT = 128 * 1024 * 1024
COPY_CHUNK_BYTES = 64 * 1024


def require_time(deadline):
    if deadline is not None and time.monotonic() >= deadline:
        raise TimeoutError('Work deadline reached during offline dependency handling')


def digest(path):
    with path.open('rb') as raw:
        return hashlib.file_digest(raw, 'sha256').hexdigest()


def regular_file(path):
    if path.is_symlink() or not path.is_file():
        raise ValueError('Missing or nonregular offline dependency input: ' + str(path))


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError('Duplicate offline manifest key: ' + key)
        value[key] = item
    return value


def locked_packages(lock_path):
    regular_file(lock_path)
    lock = tomllib.loads(lock_path.read_text(encoding='utf-8'))
    packages = []
    local = []
    seen = set()
    for package in lock.get('package', []):
        if 'source' not in package:
            local.append(package)
            continue
        name, version = package['name'], package['version']
        checksum = package.get('checksum', '')
        if (package['source'] != REGISTRY or not re.fullmatch(r'[A-Za-z0-9_-]+', name)
                or not re.fullmatch(r'[0-9][A-Za-z0-9.+-]*', version)
                or not re.fullmatch(r'[0-9a-f]{64}', checksum)):
            raise ValueError('Unsupported or unchecksummed locked dependency')
        if (name, version) in seen:
            raise ValueError('Duplicate locked dependency')
        seen.add((name, version))
        packages.append({'name': name, 'version': version, 'sha256': checksum,
                         'path': name + '-' + version + '.crate'})
    if len(local) != 1 or len(packages) > PACKAGE_LIMIT:
        raise ValueError('Offline profile requires exactly one local package and bounded registry dependencies')
    return sorted(packages, key=lambda p: (p['name'], p['version']))


def prepare_bundle(lock_path, cache, output):
    """Explicit setup only; refuses existing destination and copies no index/config."""
    packages = locked_packages(lock_path)
    if output.exists() or output.is_symlink():
        raise ValueError('Offline bundle output already exists')
    if cache.is_symlink() or not cache.is_dir():
        raise ValueError('Offline cache must be one explicit regular directory')
    rows = []
    total = 0
    for package in packages:
        archive = cache / package['path']
        regular_file(archive)
        size = archive.stat().st_size
        total += size
        if total > ARCHIVE_LIMIT or digest(archive) != package['sha256']:
            raise ValueError('Offline dependency archive checksum/size mismatch: ' + package['path'])
        rows.append({**package, 'byte_length': size})
    output.mkdir(parents=True)
    for row in rows:
        shutil.copyfile(cache / row['path'], output / row['path'])
    manifest = {'format': 'hee3.rust-offline-bundle/1', 'lock_sha256': digest(lock_path),
                'registry': REGISTRY, 'packages': rows}
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    return validate_bundle(lock_path, output)


def validate_bundle(lock_path, bundle, deadline=None):
    """Reconcile exact lock, manifest and closed archive inventory before extraction."""
    require_time(deadline)
    packages = locked_packages(lock_path)
    if bundle.is_symlink() or not bundle.is_dir():
        raise ValueError('Missing or nonregular offline bundle directory')
    manifest_path = bundle / 'manifest.json'
    regular_file(manifest_path)
    if manifest_path.stat().st_size > 1024 * 1024:
        raise ValueError('Offline bundle manifest bound exceeded')
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'), object_pairs_hook=unique_object)
    if (set(manifest) != {'format', 'lock_sha256', 'registry', 'packages'}
            or manifest['format'] != 'hee3.rust-offline-bundle/1'
            or manifest['registry'] != REGISTRY or manifest['lock_sha256'] != digest(lock_path)
            or not isinstance(manifest['packages'], list) or len(manifest['packages']) != len(packages)):
        raise ValueError('Offline bundle manifest differs from exact lock')
    expected = {'manifest.json'} | {p['path'] for p in packages}
    observed = {p.name for p in bundle.iterdir()}
    if observed != expected:
        raise ValueError('Offline dependency archive inventory mismatch (missing or extra input)')
    total = 0
    for row, package in zip(manifest['packages'], packages, strict=True):
        require_time(deadline)
        if (not isinstance(row, dict) or set(row) != set(package) | {'byte_length'}
                or any(row.get(k) != v for k, v in package.items())
                or type(row['byte_length']) is not int or row['byte_length'] < 0):
            raise ValueError('Offline bundle package row differs from exact lock')
        archive = bundle / package['path']
        regular_file(archive)
        total += archive.stat().st_size
        if (total > ARCHIVE_LIMIT or archive.stat().st_size != row['byte_length']
                or digest(archive) != package['sha256']):
            raise ValueError('Offline dependency archive checksum/size mismatch: ' + package['path'])
    return manifest


def copy_member(source, output, byte_length, deadline):
    """Copy exactly one declared member, checking work time between fixed chunks."""
    remaining = byte_length
    while remaining:
        require_time(deadline)
        requested = min(COPY_CHUNK_BYTES, remaining)
        chunk = source.read(requested)
        if not chunk:
            raise ValueError('Truncated offline archive member')
        if len(chunk) > requested:
            raise ValueError('Offline archive member exceeds requested bytes')
        require_time(deadline)
        written = output.write(chunk)
        if written != len(chunk):
            raise ValueError('Incomplete offline archive write')
        remaining -= written
    require_time(deadline)


def claim_entry(path, kind, entries):
    """Charge one normalized vendor descendant before its creation."""
    relative = str(path.relative_to(entries['root']))
    if relative == '.':
        raise ValueError('Cannot claim offline vendor root as a descendant')
    previous = entries['paths'].get(relative)
    if previous is not None:
        if previous == kind == 'directory':
            return False
        raise ValueError('Duplicate or conflicting offline vendor entry: ' + relative)
    if len(entries['paths']) >= ENTRY_LIMIT:
        raise ValueError('Offline vendor entry bound exceeded')
    if path.exists() or path.is_symlink():
        raise ValueError('Untracked existing offline vendor entry: ' + relative)
    entries['paths'][relative] = kind
    return True


def make_directory(path, entries, deadline):
    current = entries['root']
    for part in path.relative_to(current).parts:
        require_time(deadline)
        current = current / part
        if claim_entry(current, 'directory', entries):
            current.mkdir()


def extract_archive(archive, destination, package, deadline=None, *, file_budget=None, byte_budget=None,
                    member_counter=None, entries=None):
    """Read regular members explicitly; never follow archive links or extractall."""
    file_budget = FILE_LIMIT if file_budget is None else min(FILE_LIMIT, file_budget)
    byte_budget = BYTE_LIMIT if byte_budget is None else min(BYTE_LIMIT, byte_budget)
    if file_budget < 1 or byte_budget < 0:
        raise ValueError('Offline archive extraction bound exceeded')
    # Share one counter across packages. Direct archive controls use their own
    # counter; unlike a per-file cap, this also bounds empty directory headers.
    if member_counter is None:
        member_counter = [0]
    if entries is None:
        entries = {'root': destination, 'paths': {}}
    prefix = package['name'] + '-' + package['version']
    files = {}
    seen = set()
    total = 0
    with tarfile.open(archive, 'r|gz') as tar:
        for member in tar:
            require_time(deadline)
            if member_counter[0] >= MEMBER_LIMIT:
                raise ValueError('Offline archive member bound exceeded')
            member_counter[0] += 1
            parts = member.name.rstrip('/').split('/')
            if (not parts or parts[0] != prefix or any(p in ('', '.', '..') for p in parts)
                    or '\\' in member.name or member.name in seen
                    or not (member.isfile() or member.isdir())):
                raise ValueError('Unsupported offline archive member: ' + member.name)
            seen.add(member.name)
            if len(parts) == 1 and member.isdir():
                continue
            if len(parts) < 2 or parts[-1] == '.cargo-checksum.json':
                raise ValueError('Reserved offline archive member: ' + member.name)
            relative = str(PurePosixPath(*parts[1:]))
            target = destination / relative
            if member.isdir():
                make_directory(target, entries, deadline)
                continue
            # Reserve a file for the generated checksum record. Remaining global
            # capacity is supplied by materialize, so a later package cannot
            # temporarily exceed the whole-vendor bounds before reconciliation.
            if member.size < 0 or member.size > byte_budget - total or len(files) + 2 > file_budget:
                raise ValueError('Offline archive extraction bound exceeded')
            make_directory(target.parent, entries, deadline)
            claim_entry(target, 'file', entries)
            source = tar.extractfile(member)
            if source is None:
                raise ValueError('Missing archive member bytes')
            with source, target.open('xb') as output:
                copy_member(source, output, member.size, deadline)
            if target.stat().st_size != member.size:
                raise ValueError('Truncated offline archive member')
            total += member.size
            target.chmod(0o755 if member.mode & 0o111 else 0o644)
            files[relative] = digest(target)
    if 'Cargo.toml' not in files:
        raise ValueError('Offline archive lacks Cargo.toml')
    checksum = {'files': dict(sorted(files.items())), 'package': package['sha256']}
    checksum_path = destination / '.cargo-checksum.json'
    checksum_bytes = (json.dumps(checksum, sort_keys=True, separators=(',', ':')) + '\n').encode('utf-8')
    if len(files) + 1 > file_budget or len(checksum_bytes) > byte_budget - total:
        raise ValueError('Offline archive extraction bound exceeded before checksum write')
    claim_entry(checksum_path, 'file', entries)
    with io.BytesIO(checksum_bytes) as source, checksum_path.open('xb') as output:
        copy_member(source, output, len(checksum_bytes), deadline)
    files['.cargo-checksum.json'] = digest(checksum_path)
    return {name: {'sha256': value, 'byte_length': (destination / name).stat().st_size,
                   'executable': bool((destination / name).stat().st_mode & 0o111)}
            for name, value in sorted(files.items())}


def materialize(lock_path, bundle, vendor, cargo_home, deadline=None):
    manifest = validate_bundle(lock_path, bundle, deadline)
    vendor.mkdir()
    cargo_home.mkdir()
    inventory = {}
    total_bytes = 0
    member_counter = [0]
    entries = {'root': vendor, 'paths': {}}
    for package in manifest['packages']:
        require_time(deadline)
        name = package['name'] + '-' + package['version']
        destination = vendor / name
        make_directory(destination, entries, deadline)
        extracted = extract_archive(
            bundle / package['path'], destination, package, deadline,
            file_budget=FILE_LIMIT - len(inventory), byte_budget=BYTE_LIMIT - total_bytes,
            member_counter=member_counter,
            entries=entries,
        )
        for relative, identity in extracted.items():
            inventory[name + '/' + relative] = identity
            total_bytes += identity['byte_length']
    # JSON string quoting is also TOML basic-string quoting for this ordinary
    # owned temp path. No shell interpolation or registry/index is involved.
    config = ('[net]\noffline = true\n[source.crates-io]\nreplace-with = "hee3-closed"\n'
              '[source.hee3-closed]\ndirectory = ' + json.dumps(str(vendor.resolve())) + '\n')
    config_path = cargo_home / 'config.toml'
    config_path.write_text(config, encoding='utf-8')
    return {'bundle_manifest_sha256': digest(bundle / 'manifest.json'),
            'lock_sha256': manifest['lock_sha256'], 'vendor_files': inventory,
            'cargo_config_sha256': digest(config_path), 'package_count': len(manifest['packages']),
            'archive_member_count': member_counter[0], 'archive_member_limit': MEMBER_LIMIT,
            'vendor_directories': sorted(name for name, kind in entries['paths'].items() if kind == 'directory'),
            'vendor_entry_count': len(entries['paths']), 'vendor_entry_limit': ENTRY_LIMIT}


def verify_materialized(vendor, cargo_home, inventory, deadline=None):
    current = {}
    total = 0
    directories = []
    for path in vendor.rglob('*'):
        require_time(deadline)
        if len(current) + len(directories) >= ENTRY_LIMIT:
            raise ValueError('Offline vendor entry bound exceeded during verification')
        if path.is_symlink() or not (path.is_dir() or path.is_file()):
            raise ValueError('Nonregular offline vendor input')
        if path.is_file():
            total += path.stat().st_size
            if len(current) >= FILE_LIMIT or total > BYTE_LIMIT:
                raise ValueError('Offline vendor verification bound exceeded')
            current[str(path.relative_to(vendor))] = {
                'sha256': digest(path), 'byte_length': path.stat().st_size,
                'executable': bool(path.stat().st_mode & 0o111),
            }
        else:
            directories.append(str(path.relative_to(vendor)))
    if current != inventory['vendor_files']:
        raise ValueError('Offline vendor input changed')
    if (sorted(directories) != inventory['vendor_directories']
            or len(current) + len(directories) != inventory['vendor_entry_count']):
        raise ValueError('Offline vendor directory inventory changed')
    config = cargo_home / 'config.toml'
    regular_file(config)
    if digest(config) != inventory['cargo_config_sha256']:
        raise ValueError('Offline Cargo configuration changed')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    setup = sub.add_parser('prepare', help='explicit local-only setup; destination must not exist')
    setup.add_argument('--lock', type=Path, required=True)
    setup.add_argument('--cache', type=Path, required=True)
    setup.add_argument('--output', type=Path, required=True)
    check = sub.add_parser('check', help='verify copied bundle against exact lock; no extraction/build')
    check.add_argument('--lock', type=Path, required=True)
    check.add_argument('--bundle', type=Path, required=True)
    arguments = parser.parse_args()
    if arguments.command == 'prepare':
        manifest = prepare_bundle(arguments.lock, arguments.cache, arguments.output)
    else:
        manifest = validate_bundle(arguments.lock, arguments.bundle)
    print(json.dumps({'status': 'verified local offline bundle', 'packages': len(manifest['packages']),
                      'lock_sha256': manifest['lock_sha256'], 'network_requests': 0}, sort_keys=True))


if __name__ == '__main__':
    main()
