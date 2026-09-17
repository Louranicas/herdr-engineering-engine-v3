"""Pinned SQLite source/build inputs for the copied TH-DEV quality subject.

The caller's existing bounded process runner owns process observations and the
original deadline. This helper does not download, invoke Cargo, grant hostile
custody, or attest the compiler's transitive host dependencies.
"""
import hashlib
import io
import os
from pathlib import Path
import stat
import time
import zipfile

ARCHIVE_NAME = "tools/sqlite-inputs/sqlite-amalgamation-3530400.zip"
ARCHIVE_BYTES = 2946650
ARCHIVE_SHA256 = "1e71ddf93849c6a6ecf58b827c0692073d2dd7ee40196158068f7b29f422e87d"
PREFIX = "sqlite-amalgamation-3530400/"
MEMBERS = {
    "sqlite3.c": (9515341, "b1dd5d74ec7f29055a6684fa06fb3c2f6821c87dd38f9a458dfd2e8a1db28189"),
    "sqlite3.h": (690838, "919e7f2e8ed1d8f56ac17b412b8971c76aa5d1a879752cc6058f75e7d5910e1d"),
    "sqlite3ext.h": (39175, "ac9645e5c9ff0cf176efdd6e75cb5e98f46295d38e02db5c4d208826a39ab4be"),
}
# shell.c is not extracted or compiled. The fixed whole-archive digest binds it.
ARCHIVE_LAYOUT = {PREFIX: 0, PREFIX + "shell.c": 1185915,
                  **{PREFIX + name: size for name, (size, _) in MEMBERS.items()}}
ARCHIVE_LIMIT = 4 * 1024 * 1024
SOURCE_LIMIT = 12 * 1024 * 1024
OUTPUT_LIMIT = 32 * 1024 * 1024
CHUNK_BYTES = 64 * 1024
CC = Path("/usr/bin/gcc")
AR = Path("/usr/bin/ar")
CFLAGS = ("-O2", "-fPIC", "-DSQLITE_THREADSAFE=1",
          "-DSQLITE_DEFAULT_FOREIGN_KEYS=1", "-DSQLITE_DQS=0",
          "-DSQLITE_OMIT_LOAD_EXTENSION")
SQLITE_FEATURES = {
    "rusqlite": ("0.40.2", {"backup", "hooks", "modern_sqlite"}),
    # rusqlite's native dependency enables this default transitively. Its
    # pkg-config/vcpkg feature packages are present but runtime lookup is denied.
    # bundled_bindings copies Rust declarations; it does not compile SQLite.
    "libsqlite3-sys": ("0.38.2", {"default", "min_sqlite_version_3_34_1",
                                  "pkg-config", "vcpkg", "bundled_bindings"}),
}


def require_time(deadline):
    if time.monotonic() >= deadline:
        raise TimeoutError("SQLite setup exceeded the original work deadline")


def regular_file(path):
    if path.is_symlink() or not stat.S_ISREG(path.stat().st_mode):
        raise ValueError("SQLite input/output is not a regular file: " + str(path))


def file_identity(path, limit, deadline):
    require_time(deadline)
    regular_file(path)
    size = path.stat().st_size
    if size <= 0 or size > limit:
        raise ValueError("SQLite file byte bound exceeded: " + str(path))
    count = 0
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while True:
            require_time(deadline)
            chunk = stream.read(min(CHUNK_BYTES, size - count + 1))
            if not chunk:
                break
            count += len(chunk)
            if count > size:
                raise ValueError("SQLite file changed while hashing: " + str(path))
            digest.update(chunk)
    if count != size:
        raise ValueError("SQLite file changed while hashing: " + str(path))
    return {"sha256": digest.hexdigest(), "bytes": count}


def extract(archive, destination, deadline):
    """Read only exact pinned bytes; create at most three fixed source paths."""
    identity = file_identity(archive, ARCHIVE_LIMIT, deadline)
    if identity != {"sha256": ARCHIVE_SHA256, "bytes": ARCHIVE_BYTES}:
        raise ValueError("SQLite archive checksum/size mismatch")
    require_time(deadline)
    with archive.open("rb") as stream:
        raw = stream.read(ARCHIVE_LIMIT + 1)
    if len(raw) != ARCHIVE_BYTES or hashlib.sha256(raw).hexdigest() != ARCHIVE_SHA256:
        raise ValueError("SQLite archive changed before extraction")
    require_time(deadline)
    with zipfile.ZipFile(io.BytesIO(raw)) as source:
        entries = source.infolist()
        names = [entry.filename for entry in entries]
        if len(names) != len(ARCHIVE_LAYOUT) or set(names) != set(ARCHIVE_LAYOUT):
            raise ValueError("SQLite archive member inventory differs")
        if sum(entry.file_size for entry in entries) > SOURCE_LIMIT:
            raise ValueError("SQLite archive expanded byte bound exceeded")
        for entry in entries:
            require_time(deadline)
            kind = stat.S_IFMT(entry.external_attr >> 16)
            expected_kind = stat.S_IFDIR if entry.filename == PREFIX else stat.S_IFREG
            if (entry.file_size != ARCHIVE_LAYOUT[entry.filename] or entry.flag_bits != 0
                    or kind != expected_kind or entry.compress_type not in (0, 8)):
                raise ValueError("SQLite archive member shape differs")
        destination.mkdir(mode=0o700)
        result = {}
        for name, (size, expected) in MEMBERS.items():
            require_time(deadline)
            count = 0
            digest = hashlib.sha256()
            with source.open(PREFIX + name) as incoming, (destination / name).open("xb") as outgoing:
                while count < size:
                    require_time(deadline)
                    chunk = incoming.read(min(CHUNK_BYTES, size - count))
                    if not chunk or len(chunk) > size - count:
                        raise ValueError("SQLite member byte count differs")
                    count += len(chunk)
                    digest.update(chunk)
                    outgoing.write(chunk)
                require_time(deadline)
                if incoming.read(1):
                    raise ValueError("SQLite member exceeds its fixed byte bound")
            if digest.hexdigest() != expected:
                raise ValueError("SQLite member checksum mismatch: " + name)
            result[name] = {"sha256": expected, "bytes": count}
    return {"archive": identity, "files": result}


def validate_features(metadata):
    """Refuse different versions or any feature unification outside the pin."""
    packages = metadata.get("packages", [])
    nodes = metadata.get("resolve", {}).get("nodes", [])
    for name, (version, expected) in SQLITE_FEATURES.items():
        matches = [package for package in packages if package.get("name") == name]
        if len(matches) != 1 or matches[0].get("version") != version:
            raise ValueError("SQLite dependency version/inventory differs: " + name)
        selected = [node for node in nodes if node.get("id") == matches[0]["id"]]
        if (len(selected) != 1 or len(selected[0].get("features", [])) != len(expected)
                or set(selected[0].get("features", [])) != expected):
            raise ValueError("SQLite dependency feature selection differs: " + name)


def build(archive, destination, deadline, run, inventory):
    """Run fixed C/ar commands via the caller's same-deadline bounded callback.

``run(label, argv)`` is check-quality's existing wrapper: it must retain raw
streams and refuse nonzero/signal/timeout/cleanup/observation failures before
returning decoded stdout/stderr. Deliberate unit callbacks are synthetic only.
"""
    inventory.update(status="running", flags=list(CFLAGS), commands=[])
    inventory.update(extract(archive, destination, deadline))
    executables = {}
    for name, path in (("cc", CC), ("ar", AR)):
        require_time(deadline)
        resolved = path.resolve(strict=True)
        if not os.access(resolved, os.X_OK):
            raise ValueError("SQLite tool is not executable: " + str(path))
        executables[name] = {"path": str(path), "resolved_path": str(resolved),
                             **file_identity(resolved, OUTPUT_LIMIT, deadline)}
    inventory["executables"] = executables
    commands = [
        ("sqlite-compile", [executables["cc"]["resolved_path"], *CFLAGS,
                            "-c", str(destination / "sqlite3.c"),
                            "-o", str(destination / "sqlite3.o")]),
        ("sqlite-archive", [executables["ar"]["resolved_path"], "rcsD",
                            str(destination / "libsqlite3.a"), str(destination / "sqlite3.o")]),
    ]
    for label, argv in commands:
        require_time(deadline)
        inventory["commands"].append({"id": label, "argv": argv})
        streams = run(label, argv)
        if streams["stdout"] or streams["stderr"]:
            raise ValueError("Unexpected SQLite build diagnostic: " + label)
        output = "sqlite3.o" if label == "sqlite-compile" else "libsqlite3.a"
        identity = file_identity(destination / output, OUTPUT_LIMIT, deadline)
        with (destination / output).open("rb") as stream:
            prefix = stream.read(8)
        if (output == "sqlite3.o" and not prefix.startswith(b"\x7fELF")
                or output == "libsqlite3.a" and (prefix != b"!<arch>\n" or identity["bytes"] <= 8)):
            raise ValueError("SQLite build output format differs: " + output)
        inventory["files"][output] = identity
    verify(destination, inventory, deadline)
    inventory["status"] = "built private static development input"
    return {"SQLITE3_NO_PKG_CONFIG": "1", "SQLITE3_STATIC": "1",
            "SQLITE3_LIB_DIR": str(destination), "SQLITE3_INCLUDE_DIR": str(destination),
            "LIBSQLITE3_SYS_USE_PKG_CONFIG": "0"}


def verify(destination, inventory, deadline):
    require_time(deadline)
    actual = set()
    for path in destination.iterdir():
        require_time(deadline)
        if len(actual) >= len(inventory["files"]):
            raise ValueError("SQLite private build inventory changed")
        actual.add(path.name)
    if actual != set(inventory["files"]):
        raise ValueError("SQLite private build inventory changed")
    for name, expected in inventory["files"].items():
        if file_identity(destination / name, OUTPUT_LIMIT, deadline) != expected:
            raise ValueError("SQLite private build input changed: " + name)
    for expected in inventory["executables"].values():
        if (str(Path(expected["path"]).resolve(strict=True)) != expected["resolved_path"]
                or file_identity(Path(expected["resolved_path"]), OUTPUT_LIMIT, deadline)
                != {key: expected[key] for key in ("sha256", "bytes")}):
            raise ValueError("SQLite build executable changed: " + expected["path"])
