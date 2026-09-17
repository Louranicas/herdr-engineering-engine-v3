#!/usr/bin/env python3
"""Focused failure controls for the TH-DEV quality integration."""

from importlib.machinery import SourceFileLoader
from importlib.util import module_from_spec, spec_from_loader
from pathlib import Path
import gzip
import hashlib
import io
import shutil
import stat
import tarfile
import tempfile
import time
import unittest
from unittest import mock
import sys
import zipfile


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
loader = SourceFileLoader("hee3_check_quality", str(ROOT / "tools/check-quality"))
spec = spec_from_loader(loader.name, loader)
quality = module_from_spec(spec)
loader.exec_module(quality)


class QualityIntegrationControls(unittest.TestCase):
    def test_expired_work_deadline_refuses_phase(self):
        with self.assertRaisesRegex(TimeoutError, "subject copy"):
            quality.require_work_time(time.monotonic() - 1, "subject copy")

    def test_inventory_bound_refuses_excess_files(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            source = root / "source"
            source.mkdir()
            (source / "one.rs").write_text("")
            (source / "two.rs").write_text("")
            with mock.patch.object(quality, "INVENTORY_FILE_LIMIT", 1):
                with self.assertRaisesRegex(ValueError, "inventory bound"):
                    quality.tree_files(root, "source", time.monotonic() + 1)

    def test_unsupported_cargo_input_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "examples").mkdir()
            with self.assertRaisesRegex(ValueError, "examples"):
                quality.refuse_unsupported_cargo_inputs(root)

    def test_formatter_inventory_refuses_escaping_symlink(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            packages = root / "packages"
            packages.mkdir()
            outside = root / "outside"
            outside.write_text("private")
            (packages / "escape").symlink_to(outside)
            with self.assertRaisesRegex(ValueError, "Symlink is unsupported"):
                quality.tree_files(
                    root, "packages", time.monotonic() + 1,
                    allow_internal_symlinks=True,
                )

    def test_missing_exception_observation_is_fail_closed(self):
        row = {}
        self.assertFalse(quality.merge_exception_observation(row, OSError("flush failed")))
        self.assertTrue(row["observation_missing"])

    def test_cleanup_failure_writes_receipt_and_preserves_original(self):
        failed_cleanup = {
            "started": True, "exit_code": 1, "cleanup_complete": True,
            "timed_out": False, "output_limit_exceeded": False,
            "descendants_detected": False,
        }
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            for case, original in (("no-original", None), ("with-original", ValueError("primary"))):
                output = root / case
                output.mkdir()
                work = root / (case + "-work")
                work.mkdir()
                report = {"status": "pass scoped development checks"}
                with mock.patch.object(quality, "run_bounded", return_value=failed_cleanup):
                    if original is None:
                        with self.assertRaisesRegex(RuntimeError, "Workspace cleanup"):
                            quality.finalize_run(report, output, work, True, time.monotonic() + 1, time.monotonic(), None)
                    else:
                        quality.finalize_run(report, output, work, True, time.monotonic() + 1, time.monotonic(), original)
                self.assertTrue((output / "results.json").is_file())
                self.assertEqual(report["status"], "failed workspace cleanup")
                self.assertFalse(report["workspace_cleanup_complete"])

    def test_julia_precompile_classifier_rejects_diagnostic(self):
        with self.assertRaisesRegex(ValueError, "Unexpected Julia baseline stderr"):
            quality.validate_julia_precompile_stderr("Precompiling packages...\nERROR: forged\n")

    def test_cold_julia_1127_precompile_progress_is_allowed(self):
        observed = """Precompiling packages...
  57754.0 ms  ✓ Pkg (serial)
  1 dependency successfully precompiled in 58 seconds
Precompiling packages...
   1556.8 ms  ✓ InteractiveUtils
  3 dependencies successfully precompiled in 4 seconds. 5 already precompiled.
"""
        quality.validate_julia_precompile_stderr(observed)

    def sqlite_archive_fixture(self, root):
        # Fictional byte fixtures exercise setup; they are not upstream SQLite.
        source = quality.sqlite_static
        payloads = {"sqlite3.c": b"/* synthetic C */\n", "sqlite3.h": b"/* header */\n",
                    "sqlite3ext.h": b"/* extension declarations */\n", "shell.c": b"unused shell\n"}
        rows = [(source.PREFIX, b"", stat.S_IFDIR)] + [
            (source.PREFIX + name, data, stat.S_IFREG) for name, data in payloads.items()
        ]
        archive = root / "synthetic.zip"
        self.write_sqlite_zip(archive, rows)
        pins = {"ARCHIVE_SHA256": hashlib.sha256(archive.read_bytes()).hexdigest(),
                "ARCHIVE_BYTES": archive.stat().st_size,
                "ARCHIVE_LAYOUT": {name: len(data) for name, data, _ in rows},
                "MEMBERS": {name: (len(payloads[name]), hashlib.sha256(payloads[name]).hexdigest())
                            for name in ("sqlite3.c", "sqlite3.h", "sqlite3ext.h")}}
        return archive, rows, pins

    def write_sqlite_zip(self, archive, rows):
        with zipfile.ZipFile(archive, "w") as target:
            for name, data, kind in rows:
                entry = zipfile.ZipInfo(name)
                entry.create_system = 3
                entry.external_attr = (kind | 0o600) << 16
                target.writestr(entry, data)

    def sqlite_fake_build(self, root, fault=None):
        source = quality.sqlite_static
        archive, _, pins = self.sqlite_archive_fixture(root)
        tools = []
        for name in ("cc-fixture", "ar-fixture"):
            path = root / name
            path.write_bytes(b"synthetic executable identity; never launched\n")
            path.chmod(0o700)
            tools.append(path)
        pins.update(CC=tools[0], AR=tools[1])
        destination = root / "private"
        calls = []
        def run(label, argv):
            calls.append((label, argv))
            if fault == "outcome":
                # The real run wrapper propagates failed exit, signal, timeout,
                # missing observation and cleanup failures before returning.
                raise ValueError("Synthetic bounded producer failed")
            if fault == "diagnostic":
                return {"stdout": "", "stderr": "warning: synthetic warning\n"}
            if label == "sqlite-compile" and fault != "missing-object":
                (destination / "sqlite3.o").write_bytes(b"\x7fELF synthetic object; not real machine code")
            if label == "sqlite-archive":
                (destination / "libsqlite3.a").write_bytes(
                    b"wrong format" if fault == "bad-archive" else b"!<arch>\nsynthetic library"
                )
            return {"stdout": "", "stderr": ""}
        return archive, destination, pins, calls, run

    def test_sqlite_exact_archive_bytes_refuse_missing_corruption_and_alias(self):
        source = quality.sqlite_static
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            archive, _, pins = self.sqlite_archive_fixture(root)
            original = archive.read_bytes()
            with mock.patch.multiple(source, **pins):
                archive.unlink()
                with self.assertRaises(FileNotFoundError):
                    source.extract(archive, root / "missing", time.monotonic() + 5)
                archive.write_bytes(bytes([original[0] ^ 1]) + original[1:])
                with self.assertRaisesRegex(ValueError, "archive checksum/size"):
                    source.extract(archive, root / "corrupt", time.monotonic() + 5)
                archive.write_bytes(original[:-1])
                with self.assertRaisesRegex(ValueError, "archive checksum/size"):
                    source.extract(archive, root / "truncated", time.monotonic() + 5)
                archive.write_bytes(original)
                alias = root / "alias.zip"
                alias.symlink_to(archive)
                with self.assertRaisesRegex(ValueError, "not a regular file"):
                    source.extract(alias, root / "alias", time.monotonic() + 5)
                result = source.extract(archive, root / "benign", time.monotonic() + 5)
                self.assertEqual(set(result["files"]), {"sqlite3.c", "sqlite3.h", "sqlite3ext.h"})
                self.assertFalse((root / "benign/shell.c").exists())

    def test_sqlite_closed_members_and_member_digest_are_required(self):
        source = quality.sqlite_static
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            archive, rows, pins = self.sqlite_archive_fixture(root)
            faults = {"missing": rows[:-1],
                      "escape": rows[:-1] + [(source.PREFIX + "../escape", b"bad", stat.S_IFREG)],
                      "link": [rows[0], (rows[1][0], rows[1][1], stat.S_IFLNK), *rows[2:]]}
            for name, changed in faults.items():
                self.write_sqlite_zip(archive, changed)
                changed_pins = {**pins, "ARCHIVE_SHA256": hashlib.sha256(archive.read_bytes()).hexdigest(),
                                "ARCHIVE_BYTES": archive.stat().st_size}
                # Deliberate synthetic whole-archive pin reaches the independent
                # closed member/type guard; it does not change production pins.
                with self.subTest(fault=name), mock.patch.multiple(source, **changed_pins):
                    with self.assertRaisesRegex(ValueError, "archive member"):
                        source.extract(archive, root / name, time.monotonic() + 5)
                    self.assertFalse((root / name).exists())
            self.write_sqlite_zip(archive, rows)
            wrong = dict(pins["MEMBERS"])
            wrong["sqlite3.h"] = (wrong["sqlite3.h"][0], "0" * 64)
            with mock.patch.multiple(source, **{**pins, "MEMBERS": wrong}):
                with self.assertRaisesRegex(ValueError, "member checksum mismatch: sqlite3.h"):
                    source.extract(archive, root / "wrong-member", time.monotonic() + 5)
            with mock.patch.multiple(source, **pins):
                source.extract(archive, root / "benign", time.monotonic() + 5)

    def test_sqlite_byte_limits_and_chunk_deadline_refuse_before_unbounded_work(self):
        source = quality.sqlite_static
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            archive, _, pins = self.sqlite_archive_fixture(root)
            with mock.patch.multiple(source, **pins):
                with self.assertRaises(TimeoutError):
                    source.extract(archive, root / "expired", time.monotonic() - 1)
                with mock.patch.object(source, "ARCHIVE_LIMIT", 1):
                    with self.assertRaisesRegex(ValueError, "byte bound"):
                        source.extract(archive, root / "archive-limit", time.monotonic() + 5)
                with mock.patch.object(source, "SOURCE_LIMIT", 1):
                    with self.assertRaisesRegex(ValueError, "expanded byte bound"):
                        source.extract(archive, root / "expanded-limit", time.monotonic() + 5)
                self.assertFalse((root / "expanded-limit").exists())
                source.extract(archive, root / "benign", time.monotonic() + 5)
            payload = root / "chunked"
            payload.write_bytes(b"123456")
            with mock.patch.object(source, "CHUNK_BYTES", 2), mock.patch.object(source.time, "monotonic", side_effect=[0, 0, 2]):
                with self.assertRaisesRegex(TimeoutError, "original work deadline"):
                    source.file_identity(payload, 10, 1)
            self.assertEqual(source.file_identity(payload, 10, time.monotonic() + 5)["bytes"], 6)

    def test_sqlite_setup_requires_clean_bounded_outcome_and_real_output_paths(self):
        source = quality.sqlite_static
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            for fault in ("outcome", "diagnostic", "missing-object", "bad-archive", None):
                case = root / str(fault)
                case.mkdir()
                archive, destination, pins, calls, run = self.sqlite_fake_build(case, fault)
                inventory = {}
                deadline = time.monotonic() + 5
                with self.subTest(fault=fault), mock.patch.multiple(source, **pins):
                    if fault:
                        expected = FileNotFoundError if fault == "missing-object" else ValueError
                        with self.assertRaises(expected):
                            source.build(archive, destination, deadline, run, inventory)
                        self.assertNotEqual(inventory["status"], "built private static development input")
                    else:
                        with mock.patch.object(source, "require_time", wraps=source.require_time) as clock:
                            env = source.build(archive, destination, deadline, run, inventory)
                        self.assertTrue(all(call.args == (deadline,) for call in clock.call_args_list))
                        self.assertEqual(env, {"SQLITE3_NO_PKG_CONFIG": "1", "SQLITE3_STATIC": "1",
                                              "SQLITE3_LIB_DIR": str(destination), "SQLITE3_INCLUDE_DIR": str(destination),
                                              "LIBSQLITE3_SYS_USE_PKG_CONFIG": "0"})
                        self.assertEqual(calls[0][1][1:7], ["-O2", "-fPIC", "-DSQLITE_THREADSAFE=1",
                                                         "-DSQLITE_DEFAULT_FOREIGN_KEYS=1", "-DSQLITE_DQS=0",
                                                         "-DSQLITE_OMIT_LOAD_EXTENSION"])
                        self.assertEqual(calls[1][1][1], "rcsD")
                        self.assertEqual(set(inventory["files"]), {"sqlite3.c", "sqlite3.h", "sqlite3ext.h", "sqlite3.o", "libsqlite3.a"})

    def test_sqlite_setup_inputs_and_tools_must_stay_exact_after_build(self):
        source = quality.sqlite_static
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            archive, destination, pins, _, run = self.sqlite_fake_build(root)
            inventory = {}
            with mock.patch.multiple(source, **pins):
                source.build(archive, destination, time.monotonic() + 5, run, inventory)
                for path in (destination / "sqlite3.c", destination / "libsqlite3.a", pins["CC"]):
                    before = path.read_bytes()
                    path.write_bytes(before + b"changed")
                    with self.subTest(path=path.name), self.assertRaisesRegex(ValueError, "changed"):
                        source.verify(destination, inventory, time.monotonic() + 5)
                    path.write_bytes(before)
                    source.verify(destination, inventory, time.monotonic() + 5)
                (destination / "unexpected").mkdir()
                with self.assertRaisesRegex(ValueError, "inventory changed"):
                    source.verify(destination, inventory, time.monotonic() + 5)

    def test_sqlite_feature_resolution_refuses_unselected_sqlite_and_lookup_modes(self):
        import copy
        source = quality.sqlite_static
        metadata = {"packages": [{"id": "rusqlite-fixture", "name": "rusqlite", "version": "0.40.2"},
                                 {"id": "sys-fixture", "name": "libsqlite3-sys", "version": "0.38.2"}],
                    "resolve": {"nodes": [{"id": "rusqlite-fixture", "features": ["backup", "hooks", "modern_sqlite"]},
                                          {"id": "sys-fixture", "features": ["default", "min_sqlite_version_3_34_1", "pkg-config", "vcpkg", "bundled_bindings"]}]}}
        source.validate_features(metadata)
        for name in ("bundled", "loadable_extension", "sqlcipher", "buildtime_bindgen"):
            wrong = copy.deepcopy(metadata)
            wrong["resolve"]["nodes"][1]["features"].append(name)
            with self.subTest(feature=name), self.assertRaisesRegex(ValueError, "feature selection"):
                source.validate_features(wrong)
        wrong = copy.deepcopy(metadata)
        wrong["resolve"]["nodes"][0]["features"].remove("hooks")
        with self.assertRaisesRegex(ValueError, "feature selection"):
            source.validate_features(wrong)
        wrong = copy.deepcopy(metadata)
        wrong["packages"][1]["version"] = "0.38.1"
        with self.assertRaisesRegex(ValueError, "version/inventory"):
            source.validate_features(wrong)

    def t04_manifest_fixture(self, root):
        manifest = self.t03_manifest_fixture(root)
        (root / "Cargo.toml").write_text(manifest + '\n[dependencies]\nrusqlite="=0.40.2"\n')
        for name in quality.T04_IMPLEMENTATION_PATHS:
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("// Synthetic setup subject; no storage behavior executed.\n")

    def test_t04_sources_archive_and_pending_reviewed_count_are_explicit(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            self.t04_manifest_fixture(root)
            self.assertTrue(quality.has_t04(root))
            names = set(quality.quality_subject_paths(root, time.monotonic() + 5, True))
            for name in ("src/store.rs", "src/store/artifact.rs", "src/store/backup.rs", "src/store/schema.rs",
                         "migrations/001.sql", "tests/t04_store.rs", "tools/sqlite-static.py",
                         "tools/sqlite-inputs/sqlite-amalgamation-3530400.zip"):
                self.assertIn(name, names)
            with mock.patch.object(quality, "T04_TEST_COUNT", None):
                with self.assertRaisesRegex(ValueError, "T04 library-unit test count is pending"):
                    quality.rust_test_expectations(root)

    def test_t04_source_removal_cannot_silently_lower_the_profile(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            self.t04_manifest_fixture(root)
            for name in ("tests/t04_store.rs", "migrations/001.sql", "src/store/artifact.rs"):
                path = root / name
                before = path.read_bytes()
                path.unlink()
                with self.subTest(path=name), self.assertRaisesRegex(ValueError, "required T04 source"):
                    quality.rust_test_expectations(root)
                self.assertTrue(quality.has_t04(root))
                path.write_bytes(before)
            expected = quality.rust_test_expectations(root)
            self.assertEqual(expected["test_counts"], [24, 27, 60, 34, 68])
            self.assertEqual(expected["doctest_count"], 2)
            (root / "tests/t04_store.rs").write_text("")
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [24, 27, 60, 34, 68])
            def summary(count):
                return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
            ordinary = "".join(summary(count) for count in [24, 27, 60, 34, 68])
            benign = ordinary + "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
            quality.require_rust_test_summaries(benign, expected, "T04-benign")
            for fault in (benign.replace(summary(68), ""), benign.replace("68 passed", "67 passed")):
                with self.assertRaisesRegex(ValueError, "required Rust test count 68"):
                    quality.require_rust_test_summaries(fault, expected, "T04-fault")

    def t05_manifest_fixture(self, root):
        self.t04_manifest_fixture(root)
        manifest = (root / "Cargo.toml").read_text()
        manifest += 'toml={version="=1.1.5",default-features=false,features=["std","serde","parse","display"]}\n'
        for name in ("t05_roster", "t05_codec"):
            manifest += '[[test]]\nname="' + name + '"\npath="tests/' + name + '.rs"\n'
        (root / "Cargo.toml").write_text(manifest)
        for name in ("src/contracts/roster.rs", "src/roster.rs", "src/store/roster.rs", "src/store/roster/attempts.rs",
                     "tests/t05_roster.rs", "tests/t05_codec.rs", "tests/t05_store.rs", "config/models.toml", "config/agents.toml"):
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("// Synthetic harness input; no roster behavior.\n")
        (root / "src/store.rs").write_text('#[cfg(test)]\n#[path = "../tests/t05_store.rs"]\nmod roster_tests;\n')
        return manifest

    def test_t05_pending_floor_and_benign_fixed_targets_bind_inputs(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            self.t05_manifest_fixture(root)
            with mock.patch.object(quality, "T05_STORE_TEST_COUNT", None):
                with self.assertRaisesRegex(ValueError, "T05 library-unit test count is pending"):
                    quality.rust_test_expectations(root)
            # Three is only a synthetic guard fixture, never an actual T05 floor.
            with mock.patch.object(quality, "T05_STORE_TEST_COUNT", 3):
                expected = quality.rust_test_expectations(root)
                self.assertEqual(expected["test_counts"], [24, 27, 60, 34, 71, 29, 12])
                self.assertEqual(expected["unit_test_counts"], {"store::tests::": 68, "store::roster_tests::": 3})
                self.assertEqual(expected["doctest_count"], 2)
                self.assertTrue(expected["has_t05"])
                names = quality.quality_subject_paths(root, time.monotonic() + 5, True)
                for name in ("config/models.toml", "config/agents.toml", "tests/t05_store.rs",
                             "src/contracts/roster.rs", "src/store/roster/attempts.rs", "tools/check-store-mutations"):
                    self.assertIn(name, names)

    def test_t05_missing_targets_sources_or_unit_include_cannot_lower_profile(self):
        with tempfile.TemporaryDirectory() as raw, mock.patch.object(quality, "T05_STORE_TEST_COUNT", 3):
            root = Path(raw)
            manifest = self.t05_manifest_fixture(root)
            for name in ("t05_roster", "t05_codec"):
                declaration = '[[test]]\nname="' + name + '"\npath="tests/' + name + '.rs"\n'
                for changed in (manifest.replace(declaration, ""), manifest.replace(declaration, declaration + 'harness=false\n')):
                    (root / "Cargo.toml").write_text(changed)
                    with self.assertRaisesRegex(ValueError, "required Rust test target: " + name):
                        quality.rust_test_expectations(root)
                (root / "Cargo.toml").write_text(manifest)
            for name in ("config/models.toml", "src/contracts/roster.rs", "tests/t05_store.rs"):
                path = root / name
                original = path.read_bytes()
                path.unlink()
                with self.assertRaisesRegex(ValueError, "required T05 source"):
                    quality.rust_test_expectations(root)
                path.write_bytes(original)
            path = root / "src/store.rs"
            original = path.read_text()
            path.write_text(original.replace("roster_tests", "other_tests"))
            with self.assertRaisesRegex(ValueError, "T05 library-unit test include"):
                quality.rust_test_expectations(root)
            path.write_text(original)
            self.assertTrue(quality.rust_test_expectations(root)["has_t05"])

    def test_t05_unit_ownership_and_fixed_dto_codec_summaries_are_required(self):
        with tempfile.TemporaryDirectory() as raw, mock.patch.object(quality, "T05_STORE_TEST_COUNT", 3):
            root = Path(raw)
            self.t05_manifest_fixture(root)
            expected = quality.rust_test_expectations(root)
        def summary(count):
            return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        text = "".join(summary(count) for count in [24, 27, 60, 34, 71, 29, 12])
        text += "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        text += "".join(f"test store::tests::fixture_{i} ... ok\n" for i in range(68))
        text += "".join(f"test store::roster_tests::fixture_{i} ... ok\n" for i in range(3))
        quality.require_rust_test_summaries(text, expected, "synthetic ownership benign")
        for invalid in (text.replace(summary(29), ""), text.replace(summary(12), summary(11)),
                        text.replace("test store::roster_tests::fixture_2 ... ok\n", ""),
                        text.replace("test store::tests::fixture_67", "test store::roster_tests::displaced_legacy"),
                        text + "test store::roster_tests::fixture_0 ... ok\n"):
            with self.assertRaisesRegex(ValueError, "required Rust"):
                quality.require_rust_test_summaries(invalid, expected, "synthetic ownership fault")

    def test_t05_resolved_toml_rand_and_transitive_feature_faults_refuse(self):
        import copy
        # Literal selected tuple retained from the actual pinned Cargo metadata;
        # not constructed from the candidate validator's expected dictionary.
        rows = [
            ("equivalent", "1.0.2", []), ("hashbrown", "0.17.1", []), ("indexmap", "2.14.2", ["std"]),
            ("libsqlite3-sys", "0.38.2", ["bundled_bindings", "default", "min_sqlite_version_3_34_1", "pkg-config", "vcpkg"]),
            ("rusqlite", "0.40.2", ["backup", "hooks", "modern_sqlite"]),
            ("rustix", "1.1.4", ["alloc", "default", "fs", "process", "rand", "std"]),
            ("serde_spanned", "1.1.1", ["alloc", "serde", "std"]),
            ("toml", "1.1.5+spec-1.1.0", ["display", "parse", "serde", "std"]),
            ("toml_datetime", "1.1.1+spec-1.1.0", ["alloc", "serde", "std"]),
            ("toml_parser", "1.1.3+spec-1.1.0", ["alloc", "std"]),
            ("toml_writer", "1.1.2+spec-1.1.0", ["alloc", "std"]), ("winnow", "1.0.4", []),
        ]
        metadata = {"packages": [{"id": name, "name": name, "version": version} for name, version, _ in rows],
                    "resolve": {"nodes": [{"id": name, "features": features} for name, _, features in rows]}}
        quality.validate_t05_features(metadata)
        for name, feature, remove in (("toml", "unbounded", False), ("toml", "display", True),
                                      ("rustix", "rand", True), ("toml_writer", "std", True)):
            changed = copy.deepcopy(metadata)
            node = next(n for n in changed["resolve"]["nodes"] if n["id"] == name)
            node["features"].remove(feature) if remove else node["features"].append(feature)
            with self.assertRaisesRegex(ValueError, "T05 resolved features differ"):
                quality.validate_t05_features(changed)
        changed = copy.deepcopy(metadata)
        changed["packages"].append(copy.deepcopy(changed["packages"][7]))
        with self.assertRaisesRegex(ValueError, "T05 resolved features differ"):
            quality.validate_t05_features(changed)
        changed = copy.deepcopy(metadata)
        changed["resolve"]["nodes"].append(copy.deepcopy(changed["resolve"]["nodes"][7]))
        with self.assertRaisesRegex(ValueError, "Duplicate resolved"):
            quality.validate_t05_features(changed)

    def test_roster_mutations_refuse_pending_declarations_and_preserve_store_profile(self):
        loader = SourceFileLoader("hee3_store_mutation_controls", str(ROOT / "tools/check-store-mutations"))
        mutations = module_from_spec(spec_from_loader(loader.name, loader))
        loader.exec_module(mutations)
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            self.t04_manifest_fixture(root)
            old = mutations.selected_profile("store", quality, root)
            self.assertEqual(old["library_test_count"], 68)
            self.assertEqual(old["prefix"], "store::tests::")
            self.assertEqual([row[0] for row in old["mutations"]], ["replay-digest-conflict", "unknown-work-liability",
                "cancellation-intent", "object-content-hash", "migration-history", "atomic-outbox", "backup-logical-inventory"])
            self.t05_manifest_fixture(root)
            with mock.patch.object(quality, "T05_STORE_TEST_COUNT", 3):
                with mock.patch.object(mutations, "ROSTER_MUTATIONS", None), mock.patch.object(mutations, "ROSTER_MUTATION_COUNT", None):
                    with self.assertRaisesRegex(ValueError, "T05 roster mutation declarations are pending"):
                        mutations.selected_profile("roster", quality, root)
                declaration = (("synthetic", "src/store/roster.rs", "fixture", "before", "after", "Synthetic guard control only"),)
                with mock.patch.object(mutations, "ROSTER_MUTATIONS", declaration), mock.patch.object(mutations, "ROSTER_MUTATION_COUNT", 1):
                    selected = mutations.selected_profile("roster", quality, root)
                    self.assertEqual(selected["library_test_count"], 71)
                    self.assertEqual(selected["prefix"], "store::roster_tests::")
                    self.assertEqual(selected["task"], "T05")
                self.assertEqual(mutations.selected_profile("store", quality, root)["library_test_count"], 71)

    def copy_offline_subject(self, root):
        shutil.copyfile(ROOT / "Cargo.lock", root / "Cargo.lock")
        shutil.copytree(ROOT / quality.RUST_OFFLINE_BUNDLE, root / "bundle")
        return root / "Cargo.lock", root / "bundle"

    def test_exact_offline_bundle_and_materialized_vendor_are_accepted(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            inventory = quality.rust_offline.materialize(lock, bundle, root / "vendor", root / "cargo-home")
            self.assertEqual(inventory["package_count"], 43 if quality.has_t05(ROOT) else 34 if quality.has_t04(ROOT) else 11)
            self.assertIn("serde-1.0.229/Cargo.toml", inventory["vendor_files"])
            quality.rust_offline.verify_materialized(root / "vendor", root / "cargo-home", inventory)

    def test_missing_dependency_archive_is_refused_then_restored(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            archive = bundle / "serde-1.0.229.crate"
            original = archive.read_bytes()
            archive.unlink()
            with self.assertRaisesRegex(ValueError, "archive inventory mismatch"):
                quality.rust_offline.validate_bundle(lock, bundle)
            archive.write_bytes(original)
            quality.rust_offline.validate_bundle(lock, bundle)

    def test_substituted_same_size_dependency_fails_checksum(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            archive = bundle / "serde-1.0.229.crate"
            original = archive.read_bytes()
            archive.write_bytes(bytes([original[0] ^ 1]) + original[1:])
            with self.assertRaisesRegex(ValueError, "archive checksum/size mismatch"):
                quality.rust_offline.validate_bundle(lock, bundle)
            archive.write_bytes(original)
            quality.rust_offline.validate_bundle(lock, bundle)

    def test_changed_exact_lock_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            lock.write_bytes(lock.read_bytes() + b"\n")
            with self.assertRaisesRegex(ValueError, "manifest differs from exact lock"):
                quality.rust_offline.validate_bundle(lock, bundle)

    def test_unlisted_dependency_input_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            (bundle / "unlisted.crate").write_bytes(b"unlisted")
            with self.assertRaisesRegex(ValueError, "archive inventory mismatch"):
                quality.rust_offline.validate_bundle(lock, bundle)

    def test_duplicate_offline_manifest_key_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            manifest = bundle / "manifest.json"
            original = manifest.read_text()
            manifest.write_text('{"lock_sha256":"forged",' + original.lstrip()[1:])
            with self.assertRaisesRegex(ValueError, "Duplicate offline manifest key"):
                quality.rust_offline.validate_bundle(lock, bundle)

    def test_post_expansion_dependency_change_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            inventory = quality.rust_offline.materialize(lock, bundle, root / "vendor", root / "cargo-home")
            source = root / "vendor/serde-1.0.229/src/lib.rs"
            original = source.read_bytes()
            source.write_bytes(original + b"\n// substituted source\n")
            with self.assertRaisesRegex(ValueError, "vendor input changed"):
                quality.rust_offline.verify_materialized(root / "vendor", root / "cargo-home", inventory)
            source.write_bytes(original)
            quality.rust_offline.verify_materialized(root / "vendor", root / "cargo-home", inventory)

    def test_archive_path_escape_is_refused_with_regular_benign_control(self):
        package = {"name": "fixture", "version": "1.0.0", "sha256": "0" * 64}
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            for label, member_name in (("benign", "fixture-1.0.0/Cargo.toml"), ("fault", "fixture-1.0.0/../escape")):
                archive = root / (label + ".crate")
                destination = root / label
                destination.mkdir()
                payload = b'[package]\nname="fixture"\nversion="1.0.0"\n'
                with tarfile.open(archive, "w:gz") as tar:
                    member = tarfile.TarInfo(member_name)
                    member.size = len(payload)
                    tar.addfile(member, io.BytesIO(payload))
                if label == "fault":
                    with self.assertRaisesRegex(ValueError, "Unsupported offline archive member"):
                        quality.rust_offline.extract_archive(archive, destination, package)
                    self.assertFalse((root / "escape").exists())
                else:
                    result = quality.rust_offline.extract_archive(archive, destination, package)
                    self.assertIn("Cargo.toml", result)

    def test_t02_data_and_archives_are_explicit_copied_subjects(self):
        paths = set(quality.quality_subject_paths(ROOT, time.monotonic() + 5, True))
        self.assertIn("evidence/implementation/T02/sdk-smoke/metadata-benign.stdout", paths)
        self.assertIn("tests/fixtures/pi/manifest.json", paths)
        self.assertIn("tests/fixtures/pi/records/state-unavailable-sentinel.jsonl", paths)
        self.assertIn("tools/rust-offline-inputs/serde-1.0.229.crate", paths)
        self.assertIn("tools/rust-offline.py", paths)

    def test_ancestor_cargo_config_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            work = root / "work"
            work.mkdir()
            quality.refuse_ancestor_cargo_configuration(work)
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text('[net]\noffline=false\n')
            with self.assertRaisesRegex(ValueError, "Ambient ancestor Cargo configuration"):
                quality.refuse_ancestor_cargo_configuration(work)

    def test_missing_t02_target_does_not_lower_reviewed_count(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "tests").mkdir()
            names = ["t01_contracts", "t01_task", "t02_pi", "t02_transport"]
            for name in names:
                (root / "tests" / (name + ".rs")).write_text("// fixture\n")
            def manifest_for(selected):
                return "".join('[[test]]\nname="' + name + '"\npath="tests/' + name + '.rs"\n' for name in selected)
            (root / "Cargo.toml").write_text(manifest_for(names))
            expectations = quality.rust_test_expectations(root)
            self.assertEqual(expectations["test_counts"], [24, 27, 60])
            self.assertEqual(expectations["doctest_count"], 2)
            self.assertFalse(expectations["has_t03"])
            (root / "Cargo.toml").write_text(manifest_for([name for name in names if name != "t02_pi"]))
            with self.assertRaisesRegex(ValueError, "required Rust test target: t02_pi"):
                quality.rust_test_expectations(root)

    def test_missing_t02_result_or_filtered_count_is_refused(self):
        expectations = {"has_t02": True, "test_counts": [24, 27, 60], "doctest_count": 2}
        def summary(count):
            return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        marker = "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        valid = summary(24) + summary(27) + summary(60) + marker
        quality.require_rust_test_summaries(valid, expectations, "benign")
        for invalid in (summary(24) + summary(27) + marker,
                        valid.replace("60 passed", "59 passed"),
                        valid.replace("0 filtered out", "1 filtered out")):
            with self.assertRaisesRegex(ValueError, "required Rust test count"):
                quality.require_rust_test_summaries(invalid, expectations, "fault")
        with self.assertRaisesRegex(ValueError, "T02 transport controls"):
            quality.require_rust_test_summaries(valid.replace(marker, ""), expectations, "missing-transport")
        docs = {"has_t02": False, "test_counts": [2]}
        quality.require_rust_test_summaries(summary(2), docs, "benign-doctests")
        with self.assertRaisesRegex(ValueError, "required Rust test count 2"):
            quality.require_rust_test_summaries(summary(12), docs, "wrong-doctest-count")

    def test_foundation_only_profile_keeps_one_doctest(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "tests").mkdir()
            lines = []
            for name in ["t01_contracts", "t01_task"]:
                (root / "tests" / (name + ".rs")).write_text("// fixture\n")
                lines.append('[[test]]\nname="' + name + '"\npath="tests/' + name + '.rs"\n')
            (root / "Cargo.toml").write_text("".join(lines))
            expectations = quality.rust_test_expectations(root)
            self.assertFalse(expectations["has_t02"])
            self.assertFalse(expectations["has_t03"])
            self.assertEqual(expectations["test_counts"], [24, 27])
            self.assertEqual(expectations["doctest_count"], 1)

    def t03_manifest_fixture(self, root):
        names = ["t01_contracts", "t01_task", "t02_pi", "t02_transport", "t03_contract"]
        for name in names:
            path = root / "tests" / (name + ".rs")
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("// Synthetic manifest fixture; no Rust tests executed here.\n")
        for name in ("src/worker/mod.rs", "src/worker/inference.rs"):
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("// Synthetic subject fixture.\n")
        manifest = "".join('[[test]]\nname="' + name + '"\npath="tests/' + name + '.rs"\n'
                           for name in names)
        (root / "Cargo.toml").write_text(manifest)
        return manifest

    def test_t03_exact_target_preserves_prior_counts_and_copied_sources(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            self.t03_manifest_fixture(root)
            expected = quality.rust_test_expectations(root)
            self.assertTrue(expected["has_t02"])
            self.assertTrue(expected["has_t03"])
            self.assertEqual(expected["test_counts"], [24, 27, 60, 34])
            self.assertEqual(expected["doctest_count"], 2)
            paths = quality.quality_subject_paths(root, time.monotonic() + 5, True)
            for name in ("tests/t03_contract.rs", "src/worker/mod.rs", "src/worker/inference.rs"):
                self.assertIn(name, paths)
            # Candidate source contents cannot lower the reviewed test count.
            (root / "tests/t03_contract.rs").write_text("")
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [24, 27, 60, 34])

    def test_missing_or_substituted_t03_target_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            manifest = self.t03_manifest_fixture(root)
            target = '[[test]]\nname="t03_contract"\npath="tests/t03_contract.rs"\n'
            for changed in (manifest.replace(target, ""),
                            manifest.replace('path="tests/t03_contract.rs"', 'path="tests/other.rs"'),
                            manifest + target,
                            manifest.replace(target, target + "harness=false\n")):
                (root / "Cargo.toml").write_text(changed)
                with self.assertRaisesRegex(ValueError, "required Rust test target: t03_contract"):
                    quality.rust_test_expectations(root)
            (root / "Cargo.toml").write_text(manifest)
            path = root / "tests/t03_contract.rs"
            original = path.read_bytes()
            path.unlink()
            with self.assertRaisesRegex(ValueError, "required Rust test target: t03_contract"):
                quality.rust_test_expectations(root)
            path.write_bytes(original)
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [24, 27, 60, 34])

    def test_t03_requires_worker_sources_and_t02_prerequisite_targets(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            manifest = self.t03_manifest_fixture(root)
            for name in ("src/worker/mod.rs", "src/worker/inference.rs"):
                path = root / name
                original = path.read_bytes()
                path.unlink()
                with self.assertRaisesRegex(ValueError, "required T03 source"):
                    quality.rust_test_expectations(root)
                path.write_bytes(original)
            # Keeping T03 cannot hide its prerequisite by deleting T02 declarations and files.
            for name in ("t02_pi", "t02_transport"):
                (root / "tests" / (name + ".rs")).unlink()
                manifest = manifest.replace('[[test]]\nname="' + name + '"\npath="tests/' + name + '.rs"\n', "")
            (root / "Cargo.toml").write_text(manifest)
            with self.assertRaisesRegex(ValueError, "required Rust test target: t02_pi"):
                quality.rust_test_expectations(root)
            self.t03_manifest_fixture(root)
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [24, 27, 60, 34])

    def test_missing_wrong_or_filtered_t03_summary_is_refused(self):
        expected = {"has_t02": True, "has_t03": True, "test_counts": [24, 27, 60, 34], "doctest_count": 2}
        def summary(count):
            return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        marker = "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        prior = summary(24) + summary(27) + summary(60) + marker
        valid = prior + summary(34)
        quality.require_rust_test_summaries(valid, expected, "exact-34-benign")
        faults = [prior, prior + summary(33), prior + summary(35), valid + summary(34)]
        faults += [prior + summary(34).replace("0 " + label, "1 " + label)
                   for label in ("failed", "ignored", "measured", "filtered out")]
        for invalid in faults:
            with self.assertRaisesRegex(ValueError, "required Rust test count 34"):
                quality.require_rust_test_summaries(invalid, expected, "t03-summary-fault")
        with self.assertRaisesRegex(ValueError, "T02 transport controls"):
            quality.require_rust_test_summaries(valid.replace(marker, ""), expected, "missing-prerequisite-marker")
        docs = {"has_t02": False, "test_counts": [2]}
        quality.require_rust_test_summaries(summary(2), docs, "unchanged-two-doctests")

    def test_global_extraction_budgets_stop_before_next_package_write(self):
        offline = quality.rust_offline
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            cache = root / "cache"
            cache.mkdir()
            lock_text = 'version=4\n[[package]]\nname="root"\nversion="1.0.0"\n'
            manifests = {}
            for name in ("alpha", "beta"):
                content = f'[package]\nname="{name}"\nversion="1.0.0"\n'.encode()
                manifests[name] = content
                archive = cache / (name + "-1.0.0.crate")
                with archive.open("wb") as raw_archive:
                    with gzip.GzipFile(filename="", fileobj=raw_archive, mode="wb", mtime=0) as compressed:
                        with tarfile.open(fileobj=compressed, mode="w") as tar:
                            for path, payload in (("Cargo.toml", content), ("payload", b"abcd")):
                                member = tarfile.TarInfo(name + "-1.0.0/" + path)
                                member.size = len(payload)
                                tar.addfile(member, io.BytesIO(payload))
                checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
                lock_text += ('[[package]]\nname="' + name + '"\nversion="1.0.0"\nsource="'
                              + offline.REGISTRY + '"\nchecksum="' + checksum + '"\n')
            lock = root / "Cargo.lock"
            lock.write_text(lock_text)
            offline.prepare_bundle(lock, cache, root / "bundle")
            offline.materialize(lock, root / "bundle", root / "baseline", root / "home-baseline")
            first_bytes = sum(p.stat().st_size for p in (root / "baseline/alpha-1.0.0").rglob("*") if p.is_file())
            total_bytes = sum(p.stat().st_size for p in (root / "baseline").rglob("*") if p.is_file())
            # Each tiny package has two archive files plus its generated checksum.
            with mock.patch.object(offline, "FILE_LIMIT", 6), mock.patch.object(offline, "BYTE_LIMIT", total_bytes), mock.patch.object(offline, "MEMBER_LIMIT", 4), mock.patch.object(offline, "ENTRY_LIMIT", 8):
                exact = offline.materialize(lock, root / "bundle", root / "exact", root / "home-exact")
            self.assertEqual(exact["archive_member_count"], 4)
            self.assertEqual(exact["vendor_entry_count"], 8)
            for label, files, byte_limit, absent in (
                ("file", 4, total_bytes, "Cargo.toml"),
                ("byte", 6, first_bytes + len(manifests["beta"]) - 1, "Cargo.toml"),
                ("checksum-byte", 6, total_bytes - 1, ".cargo-checksum.json"),
            ):
                with self.subTest(label=label):
                    vendor = root / label
                    with mock.patch.object(offline, "FILE_LIMIT", files), mock.patch.object(offline, "BYTE_LIMIT", byte_limit):
                        with self.assertRaisesRegex(ValueError, "bound exceeded"):
                            offline.materialize(lock, root / "bundle", vendor, root / ("home-" + label))
                    written = [p for p in vendor.rglob("*") if p.is_file()]
                    self.assertLessEqual(len(written), files)
                    self.assertLessEqual(sum(p.stat().st_size for p in written), byte_limit)
                    self.assertFalse((vendor / "beta-1.0.0" / absent).exists())
            with mock.patch.object(offline, "MEMBER_LIMIT", 3):
                with self.assertRaisesRegex(ValueError, "archive member bound exceeded"):
                    offline.materialize(lock, root / "bundle", root / "member", root / "home-member")
            self.assertFalse((root / "member/beta-1.0.0/payload").exists())
            with mock.patch.object(offline, "ENTRY_LIMIT", 4):
                with self.assertRaisesRegex(ValueError, "vendor entry bound exceeded"):
                    offline.materialize(lock, root / "bundle", root / "entry", root / "home-entry")
            self.assertEqual(len(list((root / "entry").rglob("*"))), 4)
            self.assertFalse((root / "entry/beta-1.0.0").exists())

    def test_empty_directory_members_are_bounded_before_creation(self):
        offline = quality.rust_offline
        package = {"name": "fixture", "version": "1.0.0", "sha256": "0" * 64}
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            for label, directories in (("fault", [f"dir{i}" for i in range(5)]), ("benign", ["empty"])):
                archive = root / (label + ".crate")
                destination = root / label
                destination.mkdir()
                with archive.open("wb") as raw_archive:
                    with gzip.GzipFile(filename="", fileobj=raw_archive, mode="wb", mtime=0) as compressed:
                        with tarfile.open(fileobj=compressed, mode="w") as tar:
                            for name in directories:
                                member = tarfile.TarInfo("fixture-1.0.0/" + name)
                                member.type = tarfile.DIRTYPE
                                tar.addfile(member)
                            payload = b'[package]\nname="fixture"\nversion="1.0.0"\n'
                            member = tarfile.TarInfo("fixture-1.0.0/Cargo.toml")
                            member.size = len(payload)
                            tar.addfile(member, io.BytesIO(payload))
                with mock.patch.object(offline, "MEMBER_LIMIT", 4):
                    if label == "fault":
                        with self.assertRaisesRegex(ValueError, "archive member bound exceeded"):
                            offline.extract_archive(archive, destination, package)
                        self.assertEqual(len(list(destination.iterdir())), 4)
                        self.assertFalse((destination / "dir4").exists())
                    else:
                        inventory = offline.extract_archive(archive, destination, package)
                        self.assertTrue((destination / "empty").is_dir())
                        self.assertIn("Cargo.toml", inventory)

    def test_implicit_parent_directories_consume_entry_budget(self):
        offline = quality.rust_offline
        package = {"name": "fixture", "version": "1.0.0", "sha256": "0" * 64}
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            archive = root / "implicit.crate"
            with archive.open("wb") as raw_archive:
                with gzip.GzipFile(filename="", fileobj=raw_archive, mode="wb", mtime=0) as compressed:
                    with tarfile.open(fileobj=compressed, mode="w") as tar:
                        for path, payload in (("Cargo.toml", b'[package]\nname="fixture"\nversion="1.0.0"\n'),
                                              ("a/b/c/payload", b"abcd")):
                            member = tarfile.TarInfo("fixture-1.0.0/" + path)
                            member.size = len(payload)
                            tar.addfile(member, io.BytesIO(payload))
            for label, limit in (("fault", 4), ("benign", 6)):
                destination = root / label
                destination.mkdir()
                with mock.patch.object(offline, "ENTRY_LIMIT", limit):
                    if label == "fault":
                        with self.assertRaisesRegex(ValueError, "vendor entry bound exceeded"):
                            offline.extract_archive(archive, destination, package)
                        self.assertEqual(len(list(destination.rglob("*"))), 4)
                        self.assertFalse((destination / "a/b/c/payload").exists())
                    else:
                        offline.extract_archive(archive, destination, package)
                        self.assertEqual(len(list(destination.rglob("*"))), 6)
                        self.assertEqual((destination / "a/b/c/payload").read_bytes(), b"abcd")

    def test_post_expansion_empty_directory_is_refused(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            lock, bundle = self.copy_offline_subject(root)
            inventory = quality.rust_offline.materialize(lock, bundle, root / "vendor", root / "cargo-home")
            injected = root / "vendor/extra-empty-directory"
            injected.mkdir()
            with self.assertRaisesRegex(ValueError, "directory inventory changed"):
                quality.rust_offline.verify_materialized(root / "vendor", root / "cargo-home", inventory)
            injected.rmdir()
            quality.rust_offline.verify_materialized(root / "vendor", root / "cargo-home", inventory)

    def test_member_copy_checks_deadline_between_small_chunks(self):
        offline = quality.rust_offline
        clock = [0.0]
        class ExpiringOutput(io.BytesIO):
            def write(self, chunk):
                count = super().write(chunk)
                clock[0] = 2.0
                return count
        with mock.patch.object(offline, "COPY_CHUNK_BYTES", 3), mock.patch.object(offline.time, "monotonic", side_effect=lambda: clock[0]):
            source = io.BytesIO(b"abcdef")
            output = ExpiringOutput()
            with self.assertRaisesRegex(TimeoutError, "deadline reached"):
                offline.copy_member(source, output, 6, deadline=1.0)
            self.assertEqual(output.getvalue(), b"abc")
            self.assertEqual(source.tell(), 3)
            clock[0] = 0.0
            benign = io.BytesIO()
            offline.copy_member(io.BytesIO(b"abcdef"), benign, 6, deadline=1.0)
            self.assertEqual(benign.getvalue(), b"abcdef")
            with self.assertRaisesRegex(ValueError, "Truncated offline archive member"):
                offline.copy_member(io.BytesIO(b"ab"), io.BytesIO(), 3, deadline=1.0)
            class OversizedRead(io.BytesIO):
                def read(self, size=-1):
                    return b"abcdef"
            refused = io.BytesIO()
            with self.assertRaisesRegex(ValueError, "exceeds requested bytes"):
                offline.copy_member(OversizedRead(), refused, 3, deadline=1.0)
            self.assertEqual(refused.getvalue(), b"")


if __name__ == "__main__":
    unittest.main(verbosity=2)
