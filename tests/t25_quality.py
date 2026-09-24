#!/usr/bin/env python3
"""Focused failure controls for the TH-DEV quality integration."""

from importlib.machinery import SourceFileLoader
from importlib.util import module_from_spec, spec_from_loader
from pathlib import Path
import gzip
import os
import ast
import inspect
import copy
import re
import hashlib
import io
import shutil
import stat
import tarfile
import tempfile
import time
import unittest
from unittest import mock
import subprocess
import sys
import zipfile


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
loader = SourceFileLoader("hee3_check_quality", str(ROOT / "tools/check-quality"))
spec = spec_from_loader(loader.name, loader)
quality = module_from_spec(spec)
loader.exec_module(quality)


class QualityIntegrationControls(unittest.TestCase):
    def test_all_julia_fault_modes_use_uncached_source_loading(self):
        source = ast.parse((ROOT / "tools/check-quality").read_text())
        calls = [node for node in ast.walk(source) if isinstance(node, ast.Call)
                 and isinstance(node.func, ast.Name) and node.func.id == "run"
                 and node.args and ast.unparse(node.args[0]) == "'julia-' + mode"]
        self.assertEqual(len(calls), 1)
        arguments = calls[0].args[1]
        self.assertIsInstance(arguments, ast.List)
        flags = [node.value for node in arguments.elts if isinstance(node, ast.Constant)]
        self.assertEqual(flags.count("--compiled-modules=no"), 1)
        controls = [node.value for node in ast.walk(source) if isinstance(node, ast.Assign)
                    and any(isinstance(target, ast.Name) and target.id == "controls"
                            for target in node.targets)]
        self.assertEqual(len(controls), 1)
        self.assertEqual([node.value for node in controls[0].keys],
                         ["baseline", "assertion", "warning", "skip", "broken",
                          "bounds", "deprecation", "empty"])

    def test_only_tests_main_runs_parallel_and_every_other_partition_stays_serial(self):
        # A recording double: it stores the environment each partition was handed.
        expected = quality.rust_test_expectations(ROOT)
        serve = T06QualityInventoryControls.serve(expected, {})
        handed = {}
        def run(label, argv, command_env=None):
            handed[label] = command_env
            return serve(label, argv)
        parallel = {"PATH": "/usr/bin:/bin", "T13_PROCESS_EVIDENCE": "/x"}
        quality.run_rust_test_partitions(ROOT, run, "cargo", [], "fixture", expected, parallel)
        self.assertEqual(handed.pop("fixture-tests-main"), parallel)
        self.assertTrue(handed, "no other partition was run")
        self.assertTrue(all(value is None for value in handed.values()), handed)
        handed.clear()
        quality.run_rust_test_partitions(ROOT, run, "cargo", [], "fixture", expected)
        self.assertTrue(all(value is None for value in handed.values()), "no main_env, no override anywhere")
        source = (ROOT / "tools/check-quality").read_text()
        self.assertIn('parallel_main = {key: value for key, value in env.items() if key != "RUST_TEST_THREADS"}', source)

    def test_actions_names_principal_through_contracts_not_store(self):
        # A24 (STORE-G10): the actions module's only store import was Principal; it now comes from
        # contracts, the one build dependency every consumer may use.
        sources = [ROOT / "src/actions.rs", *sorted((ROOT / "src/actions").glob("*.rs"))]
        self.assertGreaterEqual(len(sources), 2, sources)
        for path in sources:
            self.assertNotIn("crate::store", path.read_text(), path)

    def test_gate_runs_are_retained_outside_the_repository(self):
        root = quality.gate_runs_root()
        self.assertTrue(root.is_absolute())
        self.assertFalse(root.resolve().is_relative_to(ROOT.resolve()), root)
        self.assertEqual(root, Path.home() / "hee3-evidence" / "gate-runs")
        self.assertIn("output = gate_runs_root() / stamp", (ROOT / "tools/check-quality").read_text())

    def test_gate_build_parallelism_is_the_operator_decision_and_has_one_owner(self):
        # Operator decisions 2026-09-24: from 2 to 8, then the full capacity of the hardware. The rule
        # is "every CPU this process may run on"; the independent source is the kernel's affinity mask.
        self.assertEqual(quality.BUILD_JOBS, str(len(os.sched_getaffinity(0))))
        self.assertGreaterEqual(int(quality.BUILD_JOBS), 8, "fewer than this host's physical cores")
        # check-quality and check-store-mutations are always gate subjects; check-pi-mutations is
        # copied only with its battery, so it is checked wherever it is present.
        examined = []
        for tool in ("tools/check-quality", "tools/check-store-mutations", "tools/check-pi-mutations"):
            if tool == "tools/check-pi-mutations" and not (ROOT / tool).is_file():
                continue
            text = (ROOT / tool).read_text()
            self.assertNotIn('"CARGO_BUILD_JOBS": "', text, tool + " restates the build parallelism")
            examined.append(tool)
        self.assertGreaterEqual(len(examined), 2, examined)

    def test_reviewed_engineering_clock_preserves_other_custody_bounds(self):
        # Independent declared policy for the expanded full regression, not an
        # engine task allocation. The 1200 s window could not fit the measured
        # 1042-control four-profile matrix (two runs killed at 1205 s); the 1500 s
        # window then ran out at 1497.6 s of 1500 (operator decision 2026-09-24: 3000 s).
        self.assertEqual(quality.TASK_LIMIT, 3300)
        self.assertEqual(quality.TASK_LIMIT - quality.CLEANUP_RESERVE, 3000)
        self.assertEqual(quality.CLEANUP_RESERVE, 300)
        self.assertEqual(quality.STREAM_LIMIT, 8 * 1024 * 1024)
        source = ast.parse((ROOT / "tools/check-quality").read_text())
        main = next(node for node in source.body
                    if isinstance(node, ast.FunctionDef) and node.name == "main")
        deadlines = {node.targets[0].id: ast.unparse(node.value)
                     for node in main.body if isinstance(node, ast.Assign)
                     and isinstance(node.targets[0], ast.Name)}
        self.assertEqual(deadlines["work_deadline"], "origin + TASK_LIMIT - CLEANUP_RESERVE")
        self.assertEqual(deadlines["task_deadline"], "origin + TASK_LIMIT")
        calls = [node for node in ast.walk(main) if isinstance(node, ast.Call)
                 and isinstance(node.func, ast.Name) and node.func.id == "run_bounded"]
        self.assertEqual(len(calls), 1)
        keywords = {node.arg: ast.unparse(node.value) for node in calls[0].keywords}
        self.assertNotIn("command_timeout", keywords)
        self.assertEqual(inspect.signature(quality.run_bounded).parameters["command_timeout"].default, 180)
        self.assertEqual(keywords["stream_limit"], "STREAM_LIMIT")

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
            self.assertEqual(expected["test_counts"], [26, 27, 60, 34, 81])
            self.assertEqual(expected["doctest_count"], 2)
            (root / "tests/t04_store.rs").write_text("")
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [26, 27, 60, 34, 81])
            def summary(count):
                return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
            ordinary = "".join(summary(count) for count in [26, 27, 60, 34, 81])
            benign = ordinary + "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
            quality.require_rust_test_summaries(benign, expected, "T04-benign")
            for fault in (benign.replace(summary(81), ""), benign.replace("81 passed", "80 passed")):
                with self.assertRaisesRegex(ValueError, "required Rust test count 81"):
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
                self.assertEqual(expected["test_counts"], [26, 27, 60, 34, 84, 29, 12])
                self.assertEqual(expected["unit_test_counts"], {"store::tests::": 81, "store::roster_tests::": 3})
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
        text = "".join(summary(count) for count in [26, 27, 60, 34, 84, 29, 12])
        text += "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        text += "".join(f"test store::tests::fixture_{i} ... ok\n" for i in range(81))
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

        # T06's actual metadata adds only rustix/event for bounded polling.
        with self.assertRaisesRegex(ValueError, "resolved features differ: rustix"):
            quality.validate_t05_features(metadata, t06=True)
        current = copy.deepcopy(metadata)
        node = next(n for n in current["resolve"]["nodes"] if n["id"] == "rustix")
        node["features"].append("event")
        quality.validate_t05_features(current, t06=True)
        with self.assertRaisesRegex(ValueError, "resolved features differ: rustix"):
            quality.validate_t05_features(current)
        node["features"].append("net")
        with self.assertRaisesRegex(ValueError, "resolved features differ: rustix"):
            quality.validate_t05_features(current, t06=True)
        # IPC01 adds exactly rustix/net (SO_PEERCRED), and only when its socket source is present.
        quality.validate_t05_features(current, t06=True, ipc01=True)
        node["features"].append("xdp")
        with self.assertRaisesRegex(ValueError, "resolved features differ: rustix"):
            quality.validate_t05_features(current, t06=True, ipc01=True)
        node["features"].remove("xdp")
        node["features"].remove("event")
        with self.assertRaisesRegex(ValueError, "resolved features differ: rustix"):
            quality.validate_t05_features(current, t06=True, ipc01=True)

    def test_roster_mutations_refuse_pending_declarations_and_preserve_store_profile(self):
        loader = SourceFileLoader("hee3_store_mutation_controls", str(ROOT / "tools/check-store-mutations"))
        mutations = module_from_spec(spec_from_loader(loader.name, loader))
        loader.exec_module(mutations)
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            self.t04_manifest_fixture(root)
            old = mutations.selected_profile("store", quality, root)
            self.assertEqual(old["library_test_count"], 81)
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
                    self.assertEqual(selected["library_test_count"], 84)
                    self.assertEqual(selected["prefix"], "store::roster_tests::")
                    self.assertEqual(selected["task"], "T05")
                self.assertEqual(mutations.selected_profile("store", quality, root)["library_test_count"], 84)

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

    def test_python_lint_names_each_planted_defect_by_its_rule(self):
        # WF-10's negative control asserts on the rule's own diagnostic (F96/F130): a planted unused
        # import and a planted blind assertRaises must each fail the step and be named, and the
        # benign mirror must print the step's required text.
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "unused.py").write_text("import os\n")
            (root / "blind.py").write_text(
                "import unittest\n\n\nclass T(unittest.TestCase):\n"
                "    def test(self):\n        with self.assertRaises(Exception):\n            pass\n")
            (root / "clean.py").write_text("print('ok')\n")
            planted = subprocess.run(quality.python_lint_argv(quality.RUFF, ["blind.py", "unused.py"]),
                                     cwd=root, capture_output=True, text=True, check=False)
            self.assertEqual(planted.returncode, 1, planted.stderr)
            self.assertIn("unused.py:1:8: F401", planted.stdout)
            self.assertIn("blind.py:6:14: B017", planted.stdout)
            clean = subprocess.run(quality.python_lint_argv(quality.RUFF, ["clean.py"]),
                                   cwd=root, capture_output=True, text=True, check=False)
            self.assertEqual((clean.returncode, clean.stdout.strip()), (0, quality.RUFF_CLEAN))

    def test_python_lint_reads_no_configuration_and_selects_only_defect_families(self):
        self.assertEqual(quality.python_lint_argv(Path("/r"), ["a.py", "b"]),
                         ["/r", "check", "--isolated", "--no-cache", "--select", "F,B",
                          "--output-format", "concise", "--", "a.py", "b"])

    def test_python_subjects_are_every_py_file_and_every_python3_script(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "tools").mkdir()
            (root / "a.py").write_text("x = 1\n")
            (root / "tools/check").write_text("#!/usr/bin/env python3\nx = 1\n")
            (root / "tools/direct").write_text("#!/usr/bin/python3\nx = 1\n")
            (root / "tools/shell").write_text("#!/usr/bin/env bash\ntrue\n")
            (root / "lib.rs").write_text("fn main() {}\n")
            self.assertEqual(
                quality.python_subjects(root, ["lib.rs", "tools/shell", "tools/direct", "a.py", "tools/check"]),
                ["a.py", "tools/check", "tools/direct"])

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
            self.assertEqual(expectations["test_counts"], [26, 27, 60])
            self.assertEqual(expectations["doctest_count"], 2)
            self.assertFalse(expectations["has_t03"])
            (root / "Cargo.toml").write_text(manifest_for([name for name in names if name != "t02_pi"]))
            with self.assertRaisesRegex(ValueError, "required Rust test target: t02_pi"):
                quality.rust_test_expectations(root)

    def test_missing_t02_result_or_filtered_count_is_refused(self):
        expectations = {"has_t02": True, "test_counts": [26, 27, 60], "doctest_count": 2}
        def summary(count):
            return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        marker = "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        valid = summary(26) + summary(27) + summary(60) + marker
        quality.require_rust_test_summaries(valid, expectations, "benign")
        for invalid in (summary(26) + summary(27) + marker,
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
            self.assertEqual(expectations["test_counts"], [26, 27])
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
            self.assertEqual(expected["test_counts"], [26, 27, 60, 34])
            self.assertEqual(expected["doctest_count"], 2)
            paths = quality.quality_subject_paths(root, time.monotonic() + 5, True)
            for name in ("tests/t03_contract.rs", "src/worker/mod.rs", "src/worker/inference.rs"):
                self.assertIn(name, paths)
            # Candidate source contents cannot lower the reviewed test count.
            (root / "tests/t03_contract.rs").write_text("")
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [26, 27, 60, 34])

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
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [26, 27, 60, 34])

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
            self.assertEqual(quality.rust_test_expectations(root)["test_counts"], [26, 27, 60, 34])

    def test_missing_wrong_or_filtered_t03_summary_is_refused(self):
        expected = {"has_t02": True, "has_t03": True, "test_counts": [26, 27, 60, 34], "doctest_count": 2}
        def summary(count):
            return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
        marker = "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        prior = summary(26) + summary(27) + summary(60) + marker
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



class T06QualityInventoryControls(unittest.TestCase):
    @staticmethod
    def summary(count):
        return f"test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n"

    @staticmethod
    def strip_targets(text, names):
        """Remove the named [[test]] registrations; the example inventory travels with t08_contract."""
        for name in names:
            text, count = re.subn(r'\[\[test\]\]\s*name = "' + re.escape(name) + r'"\s*path = "[^"]+"\s*', '', text)
            assert count == 1, name
        if "t08_contract" in names:
            text, count = re.subn(r'\[\[example\]\]\s*name = "[^"]+"\s*path = "[^"]+"\s*', '', text)
            assert count == len(quality.T08_CONTRACT_EXAMPLES)
        return text

    @staticmethod
    def serve(expected, overrides):
        """A run() double: each partition's stdout, the rest of the census on the main command."""
        rows = {"t21_process": 22, "t07_inventory": 51, "t08_native": 23, "recovery": 60, "t08_contract": 24}
        main = T06QualityInventoryControls.synthetic_output(expected)
        for count in rows.values():
            if count in expected["test_counts"]:
                main = main.replace(T06QualityInventoryControls.summary(count), "", 1)
        t06 = "".join(T06QualityInventoryControls.summary(n) for n in quality.T06_TARGET_COUNTS.values())
        for count in quality.T06_TARGET_COUNTS.values():
            main = main.replace(T06QualityInventoryControls.summary(count), "", 1)
        def run(_label, argv):
            if "--lib" in argv:
                return {"stdout": overrides.get("main", main)}
            if argv[-2] == "--test" and argv[-1] in rows:
                return {"stdout": overrides.get(argv[-1], T06QualityInventoryControls.summary(rows[argv[-1]]))}
            return {"stdout": overrides.get("t06", t06)}
        return run

    @staticmethod
    def synthetic_output(expected):
        text = "".join(T06QualityInventoryControls.summary(n) for n in expected["test_counts"])
        text += "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        for prefix, n in expected["unit_test_counts"].items():
            text += "".join(f"test {prefix}case_{i} ... ok\n" for i in range(n))
        return text

    def test_t06_fixed_census_and_subject_fixture_coverage(self):
        expected = quality.rust_test_expectations(ROOT)
        reviewed = [
            173, 0, 0, 26, 27, 60, 34, 12, 29, 22, 3, 29, 45, 40,
            29, 1, 26, 32, 3, 9, 64, 20, 39, 8, 39, 35, 9]
        if quality.has_t13(ROOT):
            # One probe case and the local probe's four classification cases are library units.
            reviewed[0] += 5
            reviewed.extend([51, 18, 17])
        if quality.has_context_battery(ROOT):
            # StepBudget's six library-unit cases land on the library target's own line.
            reviewed[0] += 6
        if quality.has_t21(ROOT):
            reviewed[0] += 1
            reviewed.extend([83, 22])
        if quality.has_t07(ROOT):
            reviewed.append(51)
        if quality.has_t08(ROOT):
            reviewed.append(23)
        if quality.has_recovery(ROOT):
            reviewed.append(60)
        if quality.has_t08_contract(ROOT):
            reviewed.append(24)
        if quality.has_t09(ROOT):
            reviewed.append(72)
        if quality.has_t07_startup(ROOT):
            reviewed.append(98)
        if quality.has_budget_battery(ROOT):
            reviewed.append(81)
        if quality.has_notify_battery(ROOT):
            reviewed.append(77)
        if quality.has_context_battery(ROOT):
            reviewed.append(67)
        if quality.has_cohort_battery(ROOT):
            reviewed.append(67)
        if quality.has_actions_battery(ROOT):
            # APP-07's one case is a library unit of app::tasks: it lands on the library's line.
            reviewed[0] += 1
            reviewed.append(131)
        if quality.has_herdr_battery(ROOT):
            reviewed.append(71)
        self.assertEqual(sorted(expected["test_counts"]), sorted(reviewed))
        self.assertEqual(sum(expected["test_counts"]), 814 + (91 if quality.has_t13(ROOT) else 0) + (106 if quality.has_t21(ROOT) else 0) + (51 if quality.has_t07(ROOT) else 0) + (23 if quality.has_t08(ROOT) else 0) + (60 if quality.has_recovery(ROOT) else 0) + (24 if quality.has_t08_contract(ROOT) else 0) + (72 if quality.has_t09(ROOT) else 0) + (98 if quality.has_t07_startup(ROOT) else 0) + (81 if quality.has_budget_battery(ROOT) else 0) + (77 if quality.has_notify_battery(ROOT) else 0) + (67 if quality.has_context_battery(ROOT) else 0) + (67 if quality.has_cohort_battery(ROOT) else 0) + (131 if quality.has_actions_battery(ROOT) else 0) + (1 if quality.has_actions_battery(ROOT) else 0) + (71 if quality.has_herdr_battery(ROOT) else 0) + (6 if quality.has_context_battery(ROOT) else 0))
        self.assertEqual(sum(expected["unit_test_counts"].values()), 186 if quality.has_t13(ROOT) else 181)
        paths = quality.quality_subject_paths(ROOT, time.monotonic() + 5, True)
        for path in ["tests/fixtures/receipts/inventory-examples.json",
                     "tests/fixtures/receipt-import/preparation.json",
                     "tests/fixtures/receipt-import/nonpass.json",
                     "evaluation/tasks/WL-U64-PARSE-001/v1/reference/src/lib.rs",
                     "evaluation/tasks/WL-U64-PARSE-001/v1/oracle/cases.json",
                     "evaluation/harnesses/u64-public-wrapper.rs"]:
            self.assertIn(path, paths)

    def test_repeated_count_groups_require_exact_multiplicity_and_no_extra_summary(self):
        expected = {"test_counts": [0, 0, 3, 3, 9], "has_t02": False}
        valid = "".join(self.summary(n) for n in [3, 0, 9, 3, 0])
        quality.require_rust_test_summaries(valid, expected, "permuted benign")
        faults = [valid.replace(self.summary(3), "", 1), valid + self.summary(3),
                  valid + self.summary(77), valid.replace(self.summary(0), "", 1),
                  valid.replace("0 ignored", "1 ignored", 1),
                  valid.replace("0 filtered out", "1 filtered out", 1),
                  valid + "test result: FAILED. malformed\n"]
        for text in faults:
            with self.subTest(text=text), self.assertRaisesRegex(ValueError, "required Rust"):
                quality.require_rust_test_summaries(text, expected, "multiplicity fault")

    def test_t06_new_library_namespaces_cannot_be_omitted_or_misattributed(self):
        expected = quality.rust_test_expectations(ROOT)
        text = "".join(self.summary(n) for n in expected["test_counts"])
        text += "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        # Synthetic parser controls, not execution or independent real case credit.
        for prefix, n in expected["unit_test_counts"].items():
            text += "".join(f"test {prefix}case_{i} ... ok\n" for i in range(n))
        quality.require_rust_test_summaries(text, expected, "synthetic census benign")
        for bad in [text.replace("test app::capture::tests::case_0 ... ok\n", ""),
                    text.replace("app::repair::tests::case_0", "wrong::case_0"),
                    text + "test store::staging_tests::case_0 ... ok\n"]:
            with self.assertRaisesRegex(ValueError, "required Rust"):
                quality.require_rust_test_summaries(bad, expected, "namespace fault")

    def test_t06_manifest_target_and_fixture_faults_refuse(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw) / "subject"
            # TMPDIR can be ROOT in the real CLI. Copy only the fixed fixture
            # inventory, never recurse into a temporary child of our own source.
            declared = quality.tomllib.loads((ROOT / "Cargo.toml").read_text())
            inputs = {"Cargo.toml", "src/store.rs", *quality.T03_IMPLEMENTATION_PATHS,
                      *quality.T04_IMPLEMENTATION_PATHS, *quality.T05_IMPLEMENTATION_PATHS,
                      *quality.T06_INPUTS, *(quality.T07_INPUTS if quality.has_t07(ROOT) else ()), *(quality.T13_INPUTS if quality.has_t13(ROOT) else ()),
                      *(quality.T21_INPUTS if quality.has_t21(ROOT) else ()), *(quality.T08_INPUTS if quality.has_t08(ROOT) else ()),
                      *(quality.RECOVERY_INPUTS if quality.has_recovery(ROOT) else ()), *(quality.T08_CONTRACT_INPUTS if quality.has_t08_contract(ROOT) else ()),
                      *(quality.T09_INPUTS if quality.has_t09(ROOT) else ()),
                      *(quality.T07_STARTUP_INPUTS if quality.has_t07_startup(ROOT) else ()),
                      *(quality.BUDGET_INPUTS if quality.has_budget_battery(ROOT) else ()),
                      *(quality.NOTIFY_INPUTS if quality.has_notify_battery(ROOT) else ()),
                      *(quality.CONTEXT_INPUTS if quality.has_context_battery(ROOT) else ()),
                      *(quality.COHORT_INPUTS if quality.has_cohort_battery(ROOT) else ()),
                      *(quality.ACTIONS_INPUTS if quality.has_actions_battery(ROOT) else ()),
                      *(quality.HERDR_INPUTS if quality.has_herdr_battery(ROOT) else ()),
                      *(target["path"] for target in declared["test"])}
            for name in sorted(inputs):
                destination = root / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, destination)
            manifest = root / "Cargo.toml"
            original = manifest.read_text()
            for fault in [original.replace('name = "t06_receipts"', 'name = "renamed_receipts"'),
                          original.replace('path = "tests/t06_receipts.rs"', 'path = "tests/t06_store.rs"'),
                          original.replace('name = "t06_receipts"', 'name = "t06_receipts"\nharness = false'),
                          original + '\n[[test]]\nname="extra"\npath="tests/t01_task.rs"\n']:
                manifest.write_text(fault)
                with self.assertRaisesRegex(ValueError, "T06"):
                    quality.rust_test_expectations(root)
            manifest.write_text(original)
            path = root / "tests/fixtures/receipt-import/nonpass.json"
            before = path.read_bytes()
            path.unlink()
            with self.assertRaisesRegex(ValueError, "required T06 fixture"):
                quality.rust_test_expectations(root)
            path.write_bytes(before)
            self.assertEqual(sum(quality.rust_test_expectations(root)["test_counts"]), sum(quality.rust_test_expectations(ROOT)["test_counts"]))


    def combined_fixture(self, root):
        declared = quality.tomllib.loads((ROOT / "Cargo.toml").read_text())
        inputs = {"Cargo.toml", "src/store.rs", *quality.T03_IMPLEMENTATION_PATHS,
                  *quality.T04_IMPLEMENTATION_PATHS, *quality.T05_IMPLEMENTATION_PATHS,
                  *quality.T06_INPUTS, *(quality.T07_INPUTS if quality.has_t07(ROOT) else ()), *quality.T13_INPUTS, *quality.T21_INPUTS,
                  *(quality.T08_INPUTS if quality.has_t08(ROOT) else ()),
                  *(quality.RECOVERY_INPUTS if quality.has_recovery(ROOT) else ()), *(quality.T08_CONTRACT_INPUTS if quality.has_t08_contract(ROOT) else ()),
                  *(quality.T09_INPUTS if quality.has_t09(ROOT) else ()),
                  *(quality.T07_STARTUP_INPUTS if quality.has_t07_startup(ROOT) else ()),
                  *(quality.BUDGET_INPUTS if quality.has_budget_battery(ROOT) else ()),
                  *(quality.NOTIFY_INPUTS if quality.has_notify_battery(ROOT) else ()),
                  *(quality.CONTEXT_INPUTS if quality.has_context_battery(ROOT) else ()),
                  *(quality.COHORT_INPUTS if quality.has_cohort_battery(ROOT) else ()),
                  *(quality.ACTIONS_INPUTS if quality.has_actions_battery(ROOT) else ()),
                  *(quality.HERDR_INPUTS if quality.has_herdr_battery(ROOT) else ()),
                  *(target["path"] for target in declared["test"])}
        for name in sorted(inputs):
            destination = root / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, destination)
        return root / "Cargo.toml"

    def synthetic_combined_output(self, expected):
        text = "".join(self.summary(n) for n in expected["test_counts"])
        text += "T02_TRANSPORT_FINITE_CONTROLS: 5 passed; TH-DEV only\n"
        for prefix, n in expected["unit_test_counts"].items():
            text += "".join(f"test {prefix}case_{i} ... ok\n" for i in range(n))
        return text

    def test_combined_fixed_1011_census_preserves_both_twenty_two_groups(self):
        expected = quality.rust_test_expectations(ROOT)
        self.assertEqual(sum(expected["test_counts"]), 1011 + (51 if quality.has_t07(ROOT) else 0) + (23 if quality.has_t08(ROOT) else 0) + (60 if quality.has_recovery(ROOT) else 0) + (24 if quality.has_t08_contract(ROOT) else 0) + (72 if quality.has_t09(ROOT) else 0) + (98 if quality.has_t07_startup(ROOT) else 0) + (81 if quality.has_budget_battery(ROOT) else 0) + (77 if quality.has_notify_battery(ROOT) else 0) + (67 if quality.has_context_battery(ROOT) else 0) + (67 if quality.has_cohort_battery(ROOT) else 0) + (131 if quality.has_actions_battery(ROOT) else 0) + (1 if quality.has_actions_battery(ROOT) else 0) + (71 if quality.has_herdr_battery(ROOT) else 0) + (6 if quality.has_context_battery(ROOT) else 0))
        self.assertEqual(len(expected["test_counts"]), 32 + sum(1 for present in (quality.has_t07, quality.has_t08, quality.has_recovery, quality.has_t08_contract, quality.has_t09, quality.has_t07_startup, quality.has_budget_battery, quality.has_notify_battery, quality.has_context_battery, quality.has_cohort_battery, quality.has_actions_battery, quality.has_herdr_battery) if present(ROOT)))
        # t21_process and t06_availability both hold 22: the repeated-count group.
        self.assertEqual(expected["test_counts"].count(22), 2)
        self.assertEqual(sum(expected["unit_test_counts"].values()), 186)
        quality.require_rust_test_summaries(self.synthetic_combined_output(expected), expected, "synthetic complete")

    def test_historical_t06_subject_keeps_814_and_173(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            manifest = self.combined_fixture(root)
            text = manifest.read_text()
            text = re.sub(r'\[\[test\]\]\s*name = "t(?:07|08|09|13|21)_[^"]+"\s*path = "[^"]+"\s*', '', text)
            # t07_startup is already removed by the task-prefix regex above; naming it here
            # too would trip strip_targets' own single-match assertion.
            text = self.strip_targets(text, ["recovery", "accounting", "t11_notify", "t11_context",
                                             "t22_cohort", "t28_actions", "t16_herdr"])
            text = re.sub(r'\[\[example\]\]\s*name = "[^"]+"\s*path = "[^"]+"\s*', '', text)
            manifest.write_text(text)
            for name in {*quality.T07_INPUTS, *quality.T08_INPUTS, *quality.T13_INPUTS, *quality.T21_INPUTS,
                         *quality.RECOVERY_INPUTS, *quality.T08_CONTRACT_OWN_SOURCES, *quality.T09_INPUTS,
                         *quality.T07_STARTUP_INPUTS, *quality.BUDGET_INPUTS, *quality.NOTIFY_INPUTS, \
                         *quality.CONTEXT_INPUTS, *quality.COHORT_INPUTS, *quality.ACTIONS_INPUTS, \
                         *quality.HERDR_INPUTS,
                         *("tests/"+n+".rs" for n in (*quality.T13_TARGET_COUNTS, *quality.T21_TARGET_COUNTS,
                                                      "t07_startup", "accounting", "t11_notify", "t11_context", "t22_cohort", "t28_actions", "t16_herdr"))}:
                path = root / name
                if path.exists() and name not in quality.T03_IMPLEMENTATION_PATHS: path.unlink()
            expected = quality.rust_test_expectations(root)
            self.assertEqual(sum(expected["test_counts"]), 814)
            self.assertEqual(sum(expected["unit_test_counts"].values()), 173)
            self.assertEqual(quality.rust_test_partitions(root), [("tests", ["--all-targets"])])

    def test_combined_missing_duplicate_and_substituted_targets_refuse(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root); original = manifest.read_text()
            for fault in [original.replace('name = "t13_service"','name = "foreign_service"'),
                          original.replace('name = "t21_process"','name = "t21_analysis"'),
                          original.replace('path = "tests/t21_process.rs"','path = "tests/t21_analysis.rs"'),
                          original.replace('name = "t13_probe"','name = "t13_probe"\nharness = false')]:
                manifest.write_text(fault)
                with self.assertRaisesRegex(ValueError, "T06"): quality.rust_test_expectations(root)

    def test_substantive_extension_source_cannot_hide_removed_targets(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root)
            manifest.write_text(re.sub(r'\[\[test\]\]\s*name = "t(?:13|21)_[^"]+"\s*path = "[^"]+"\s*', '', manifest.read_text()))
            self.assertTrue(quality.has_t13(root)); self.assertTrue(quality.has_t21(root))
            with self.assertRaisesRegex(ValueError,"T06"): quality.rust_test_expectations(root)

    def test_t21_rust_only_requires_julia_j01_and_descendant_fixture(self):
        paths=quality.quality_subject_paths(ROOT,time.monotonic()+5,True)
        for name in quality.T21_INPUTS: self.assertIn(name,paths)
        with tempfile.TemporaryDirectory() as raw:
            root=Path(raw);self.combined_fixture(root)
            for name in ["tests/fixtures/t21/J01.json","julia/src/Evaluate.jl","tests/fixtures/t21/descendant-control.py"]:
                path=root/name;before=path.read_bytes();path.unlink()
                with self.assertRaisesRegex(ValueError,"required extension input"):quality.rust_test_expectations(root)
                path.write_bytes(before)

    def test_missing_one_twenty_two_summary_refuses(self):
        expected=quality.rust_test_expectations(ROOT);text=self.synthetic_combined_output(expected)
        with self.assertRaisesRegex(ValueError,"required Rust"):
            quality.require_rust_test_summaries(text.replace(self.summary(22),'',1),expected,"missing repeated22")

    def test_missing_service_unit_namespace_refuses(self):
        expected=quality.rust_test_expectations(ROOT);text=self.synthetic_combined_output(expected)
        with self.assertRaisesRegex(ValueError,"required Rust"):
            quality.require_rust_test_summaries(text.replace('service::probe::tests::case_0','foreign::case_0'),expected,"service unit")

    def test_t21_float_roundtrip_exact_feature_tuple(self):
        rows=dict(quality.T05_FEATURES);rows['rustix']=("1.1.4",["alloc","default","event","fs","process","rand","std"])
        rows['serde_json']=("1.0.151",["default","float_roundtrip","std"])
        metadata={"packages":[{"id":n,"name":n,"version":v}for n,(v,_)in rows.items()],"resolve":{"nodes":[{"id":n,"features":f}for n,(_,f)in rows.items()]}}
        quality.validate_t05_features(metadata,t06=True,t21=True)
        for features in [["default","std"],["default","float_roundtrip","std","arbitrary_precision"]]:
            changed=copy.deepcopy(metadata);next(x for x in changed['resolve']['nodes']if x['id']=='serde_json')['features']=features
            with self.assertRaisesRegex(ValueError,"serde_json"):quality.validate_t05_features(changed,t06=True,t21=True)

    def test_selected_julia_cache_missing_added_changed_refuses(self):
        with tempfile.TemporaryDirectory() as raw:
            root=Path(raw)
            for relative in quality.T21_PACKAGE_PATHS:
                path=root/relative/'src.jl';path.parent.mkdir(parents=True);path.write_text('synthetic package fixture')
            expected=quality.t21_package_inventory(root,time.monotonic()+5)
            quality.verify_t21_package_inventory(root,expected,time.monotonic()+5)
            first=root/quality.T21_PACKAGE_PATHS[0]/'src.jl';first.write_text('changed')
            with self.assertRaisesRegex(ValueError,"cache changed"):quality.verify_t21_package_inventory(root,expected,time.monotonic()+5)
            first.write_text('synthetic package fixture');extra=first.with_name('extra.jl');extra.write_text('extra')
            with self.assertRaisesRegex(ValueError,"cache changed"):quality.verify_t21_package_inventory(root,expected,time.monotonic()+5)
            extra.unlink();first.unlink()
            with self.assertRaisesRegex(ValueError,"Missing selected"):quality.t21_package_inventory(root,time.monotonic()+5)

    def test_t21_numerical_environment_has_separate_owned_report(self):
        with tempfile.TemporaryDirectory() as raw:
            root=Path(raw); captured=root/'julia-j01.stdout';captured.write_bytes(b'original stdout')
            original={"KEEP":"same","T21_JULIA_REPORT":str(captured)}
            selected=quality.t21_analysis_environment(original,root)
            self.assertEqual(selected["KEEP"],"same")
            self.assertEqual(original["T21_JULIA_REPORT"],str(captured))
            self.assertNotEqual(selected["T21_JULIA_REPORT"],str(captured))
            self.assertEqual(selected["T21_JULIA_REPORT"],str(root/'julia-t21-analysis-report.json'))
            self.assertEqual(captured.read_bytes(),b'original stdout')
            destination=Path(selected["T21_JULIA_REPORT"]);destination.write_bytes(b'already owned')
            with self.assertRaisesRegex(ValueError,"already exists"):
                quality.t21_analysis_environment(original,root)
            destination.unlink();destination.symlink_to(root/'missing')
            with self.assertRaisesRegex(ValueError,"already exists"):
                quality.t21_analysis_environment(original,root)

    def test_t21_numerical_report_requires_exact_j01_and_request(self):
        with tempfile.TemporaryDirectory() as raw:
            path=Path(raw)/'report.json';request='a'*64
            expected=quality.json.dumps({"request_sha256":"sha256:"+request,"synthetic":"benign"}).encode()
            with self.assertRaisesRegex(ValueError,"Missing regular"):
                quality.require_t21_analysis_report(path,expected,request)
            path.write_bytes(expected)
            result=quality.require_t21_analysis_report(path,expected,request)
            self.assertEqual(result["sha256"],hashlib.sha256(expected).hexdigest())
            self.assertEqual(result["bytes"],len(expected))
            with self.assertRaisesRegex(ValueError,"request binding"):
                quality.require_t21_analysis_report(path,expected,'b'*64)
            for changed in [b'',expected+b' ',b'x'*65537]:
                path.write_bytes(changed)
                with self.assertRaisesRegex(ValueError,"differs.*bound"):
                    quality.require_t21_analysis_report(path,expected,request)
            path.unlink();path.symlink_to(Path(raw)/'missing')
            with self.assertRaisesRegex(ValueError,"Missing regular"):
                quality.require_t21_analysis_report(path,expected,request)

    def test_julia_numerical_summary_requires_all_three_exact_groups(self):
        benign = ("Test Summary: | Pass Total Time\nT21 descriptive evaluator | 82 82 1.0s\n"
                  "Test Summary: | Pass Total Time\nT21 numerical boundary controls | 10 10 0.1s\n"
                  "Test Summary: | Pass Total Time\nT21 deferred string decoding | 7 7 0.0s\n")
        quality.require_t21_julia_summaries(benign, "")
        for fault in ["", benign.replace("82 82", "0 0"), benign.replace("82 82", "81 82"),
                      benign.replace("Pass Total", "Pass Broken Total", 1),
                      benign.replace("T21 deferred string decoding | 7 7 0.0s\n", ""),
                      benign + "Test Summary: | Pass Total\nT21 extra | 1 1\n",
                      benign.replace("T21 deferred string decoding", "T21 numerical boundary controls"),
                      benign.replace("10 10 0.1s", "10 10 ignored")]:
            with self.assertRaisesRegex(ValueError, "T21 Julia evaluator"):
                quality.require_t21_julia_summaries(fault, "")
        with self.assertRaisesRegex(ValueError, "diagnostics"):
            quality.require_t21_julia_summaries(benign, "Warning: synthetic")
        # With the T22 cohort battery the file carries a fourth group, exactly as declared:
        # it is required when the battery is present and refused when it is not.
        cohort = benign + "Test Summary: | Pass Total Time\nT22 cohort cohesion | 168 168 4.1s\n"
        quality.require_t21_julia_summaries(cohort, "", cohort=True)
        for fault, flag in [(benign, True), (cohort, False),
                            (cohort.replace("168 168", "167 168"), True),
                            (cohort.replace("T22 cohort cohesion", "T22 cohort drift"), True)]:
            with self.subTest(cohort=flag), self.assertRaisesRegex(ValueError, "T21 Julia evaluator"):
                quality.require_t21_julia_summaries(fault, "", cohort=flag)

    def test_each_profile_retains_distinct_process_evidence_roots(self):
        environment={"T13_PROCESS_EVIDENCE":"unused13", "T21_PROCESS_EVIDENCE":"unused21", "KEEP":"same"}
        first=quality.retained_command_environment(environment,Path('/retained'),'default-debug-tests')
        second=quality.retained_command_environment(environment,Path('/retained'),'no-default-release-tests')
        for variable in ['T13_PROCESS_EVIDENCE','T21_PROCESS_EVIDENCE']:
            self.assertNotEqual(first[variable],second[variable])
            self.assertTrue(first[variable].startswith('/retained/'))
            self.assertEqual(first[variable].split('/')[-1],'default-debug-tests')
        self.assertEqual(environment['T13_PROCESS_EVIDENCE'],'unused13')
        self.assertEqual(first['KEEP'],'same')
        with self.assertRaisesRegex(ValueError,'command label'):
            quality.retained_command_environment(environment,Path('/retained'),'../escape')

    def test_t21_partition_selects_each_declared_target_once(self):
        partitions = quality.rust_test_partitions(ROOT)
        self.assertEqual([row[0] for row in partitions], ["tests-main", "tests-t06", "tests-t21-process"] + (["tests-t07-inventory"] if quality.has_t07(ROOT) else []) + (["tests-t08-native"] if quality.has_t08(ROOT) else []) + (["tests-recovery"] if quality.has_recovery(ROOT) else []) + (["tests-t08-contract"] if quality.has_t08_contract(ROOT) else []))
        self.assertEqual(partitions[0][1][:2], ["--lib", "--bins"])
        self.assertEqual(partitions[2][1], ["--test", "t21_process"])
        main = partitions[0][1][2:]
        self.assertEqual(main[::2], ["--test"] * (11 + (1 if quality.has_t09(ROOT) else 0)
                                                  + (1 if quality.has_t07_startup(ROOT) else 0)
                                                  + (1 if quality.has_budget_battery(ROOT) else 0)
                                                  + (1 if quality.has_notify_battery(ROOT) else 0)
                                                  + (1 if quality.has_context_battery(ROOT) else 0)
                                                  + (1 if quality.has_cohort_battery(ROOT) else 0)
                                                  + (1 if quality.has_actions_battery(ROOT) else 0)
                                                  + (1 if quality.has_herdr_battery(ROOT) else 0)))
        expected = {"t01_contracts", "t01_task", "t02_pi", "t02_transport", "t03_contract",
                    "t05_roster", "t05_codec", "t13_service", "t13_probe", "t13_local_probe", "t21_analysis"}
        if quality.has_t09(ROOT):
            # The routing battery is pure and finishes in under a second: it needs no
            # command of its own and runs with the main partition.
            expected.add("t09_route")
        # The startup pass and the six application batteries are likewise pure value
        # controls that finish in well under a second each, so they run with the main
        # partition rather than taking commands of their own.
        if quality.has_t07_startup(ROOT):
            expected.add("t07_startup")
        if quality.has_budget_battery(ROOT):
            expected.add("accounting")
        if quality.has_notify_battery(ROOT):
            expected.add("t11_notify")
        if quality.has_context_battery(ROOT):
            expected.add("t11_context")
        if quality.has_cohort_battery(ROOT):
            expected.add("t22_cohort")
        if quality.has_actions_battery(ROOT):
            expected.add("t28_actions")
        if quality.has_herdr_battery(ROOT):
            expected.add("t16_herdr")
        self.assertEqual(set(main[1::2]), expected)
        self.assertEqual(len(main[1::2]), len(set(main[1::2])))
        t06 = partitions[1][1]
        self.assertEqual(t06[::2], ["--test"] * 18)
        self.assertEqual(set(t06[1::2]), {"t06_receipts", "t06_store", "t06_graph",
                    "t06_driver", "t06_consistency", "t06_workspace", "t06_terminal",
                    "t06_evidence", "t06_collector", "t06_availability", "t06_decision",
                    "t06_staging", "t06_subjects", "t06_process_timing", "t06_workspace_export",
                    "t06_bounded_preflight", "t06_durable_control", "t06_receipt_import"})
        self.assertEqual(len(t06[1::2]), len(set(t06[1::2])))

    def test_t21_partition_refuses_unaccounted_or_conditional_targets(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            manifest = self.combined_fixture(root)
            original = manifest.read_text()
            faults = [original.replace('autotests = false', 'autotests = true'),
                      original.replace('autobins = false', 'autobins = true'),
                      original.replace('autotests = false', ''),
                      original.replace('[workspace]', '[workspace]\nmembers = ["other"]'),
                      original.replace('[lib]', '[lib]\nharness = false'),
                      original + '\n[[example]]\nname="extra"\npath="src/main.rs"\n',
                      original + '\n[[bench]]\nname="extra"\npath="src/main.rs"\n',
                      original.replace('name = "t21_process"', 'required-features = ["hidden"]\nname = "t21_process"'),
                      original.replace('name = "t21_process"', 'test = false\nname = "t21_process"'),
                      original.replace('name = "t21_process"', 'name = "unlisted_process"')]
            for changed in faults:
                self.assertNotEqual(original, changed)
                manifest.write_text(changed)
                with self.subTest(change=changed[-120:]), self.assertRaises(ValueError):
                    quality.rust_test_partitions(root)
            manifest.write_text(original)
            for directory in ("examples", "benches"):
                extra = root / directory
                extra.mkdir()
                with self.assertRaisesRegex(ValueError, "Unsupported Cargo input"):
                    quality.rust_test_partitions(root)
                extra.rmdir()
            self.assertEqual(quality.rust_test_partitions(root), quality.rust_test_partitions(ROOT))

    def test_t21_partition_runs_both_with_unique_labels_and_full_oracle(self):
        expected = quality.rust_test_expectations(ROOT)
        combined = self.synthetic_combined_output(expected)
        process = self.summary(22)
        main = combined.replace(process, "", 1)
        inventory = self.summary(51) if quality.has_t07(ROOT) else ""
        main = main.replace(inventory, "", 1) if inventory else main
        calls = []
        serve = self.serve(expected, {})
        def run(label, argv):
            calls.append((label, argv))
            return serve(label, argv)
        common = ["--workspace", "--locked", "--offline", "--release", "--no-default-features"]
        quality.run_rust_test_partitions(ROOT, run, "/pinned/cargo", common, "no-default-release", expected)
        self.assertEqual([row[0] for row in calls],
                         ["no-default-release-tests-main", "no-default-release-tests-t06", "no-default-release-tests-t21-process"] + (["no-default-release-tests-t07-inventory"] if quality.has_t07(ROOT) else []) + (["no-default-release-tests-t08-native"] if quality.has_t08(ROOT) else []) + (["no-default-release-tests-recovery"] if quality.has_recovery(ROOT) else []) + (["no-default-release-tests-t08-contract"] if quality.has_t08_contract(ROOT) else []))
        for _, argv in calls:
            self.assertEqual(argv[:7], ["/pinned/cargo", "test", *common])
            self.assertNotIn("--all-targets", argv)
        for variable in ("T13_PROCESS_EVIDENCE", "T21_PROCESS_EVIDENCE"):
            environment = {variable: "unused"}
            roots = [quality.retained_command_environment(environment, Path('/retained'), label)[variable]
                     for label, _ in calls]
            self.assertEqual(len(set(roots)), 3 + sum(1 for present in (quality.has_t07, quality.has_t08, quality.has_recovery, quality.has_t08_contract) if present(ROOT)))

    def test_t21_partition_missing_duplicate_or_filtered_result_refuses(self):
        expected = quality.rust_test_expectations(ROOT)
        process = self.summary(22)
        for result in ("", process + process, process.replace("22 passed", "21 passed")):
            with self.subTest(result=result), self.assertRaisesRegex(ValueError, "Rust test count"):
                quality.run_rust_test_partitions(ROOT, self.serve(expected, {"t21_process": result}), "cargo", [], "fixture", expected)

    def test_t07_fixed_51_inventory_census_and_owned_inputs(self):
        expected = quality.rust_test_expectations(ROOT)
        self.assertEqual(sum(expected["test_counts"]), 1840)
        self.assertEqual(len(expected["test_counts"]), 44)
        # t13_service also holds 51, so the census carries two 51-rows.
        self.assertEqual(expected["test_counts"].count(51), 2)
        self.assertEqual(expected["test_counts"].count(43), 0)
        self.assertEqual(sum(expected["unit_test_counts"].values()), 186)
        paths = quality.quality_subject_paths(ROOT, time.monotonic() + 5, True)
        for name in ["src/store/recovery.rs", "tests/t07_inventory.rs"]:
            self.assertIn(name, paths)
        self.assertFalse(any(name.startswith("development/t06/") for name in paths))

    def test_t07_absent_historical_combined_1011_is_preserved(self):
        # The T13/T21-only subject has neither the inventory nor the native target.
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root)
            # t07_startup shares the "t07_" prefix has_t07 reads, so a root with no T07
            # battery must lose it too, along with the six application batteries.
            manifest.write_text(self.strip_targets(manifest.read_text(), ["t07_inventory", "t08_native", "recovery", "t08_contract", "t09_route",
                                                                          "t07_startup", "accounting", "t11_notify", "t11_context", "t22_cohort", "t28_actions", "t16_herdr"]))
            for name in ["src/store/recovery.rs", "tests/t07_inventory.rs", *quality.T08_INPUTS, *quality.RECOVERY_INPUTS, *quality.T08_CONTRACT_OWN_SOURCES, *quality.T09_INPUTS,
                         *quality.T07_STARTUP_INPUTS, *quality.BUDGET_INPUTS, *quality.NOTIFY_INPUTS, *quality.CONTEXT_INPUTS,
                         *quality.COHORT_INPUTS, *quality.ACTIONS_INPUTS, *quality.HERDR_INPUTS]:
                (root / name).unlink()
            self.assertFalse(quality.has_t07(root))
            self.assertFalse(quality.has_t08(root))
            self.assertFalse(quality.has_recovery(root))
            self.assertFalse(quality.has_t08_contract(root))
            self.assertFalse(quality.has_t09(root))
            expected = quality.rust_test_expectations(root)
            self.assertEqual(sum(expected["test_counts"]), 1011)
            self.assertEqual(len(expected["test_counts"]), 32)
            # The context battery is stripped in this fixture, so StepBudget's namespace is
            # not registered and the unit census is the historical 179.
            self.assertEqual(sum(expected["unit_test_counts"].values()), 179)
            quality.require_rust_test_summaries(self.synthetic_combined_output(expected), expected, "historical1008")
            self.assertEqual([n for n,_ in quality.rust_test_partitions(root)], ["tests-main", "tests-t06", "tests-t21-process"])

    def test_t07_missing_or_substituted_owned_inputs_refuse(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); self.combined_fixture(root)
            for name in ["src/store/recovery.rs", "tests/t07_inventory.rs"]:
                path = root/name; before = path.read_bytes(); path.unlink()
                with self.assertRaisesRegex(ValueError, "required extension input"):
                    quality.rust_test_expectations(root)
                path.symlink_to(ROOT/name)
                with self.assertRaisesRegex(ValueError, "required extension input"):
                    quality.rust_test_expectations(root)
                path.unlink(); path.write_bytes(before)
            self.assertEqual(sum(quality.rust_test_expectations(root)["test_counts"]), 1840)

    def test_t07_omitted_changed_or_conditional_target_refuses(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root); original = manifest.read_text()
            faults = [re.sub(r'\[\[test\]\]\s*name = "t07_inventory"\s*path = "tests/t07_inventory.rs"\s*', '', original),
                      original.replace('name = "t07_inventory"','name = "other_inventory"'),
                      original.replace('path = "tests/t07_inventory.rs"','path = "tests/t06_store.rs"'),
                      original.replace('name = "t07_inventory"','name = "t07_inventory"\nharness = false'),
                      original.replace('name = "t07_inventory"','name = "t07_inventory"\nrequired-features = ["hidden"]'),
                      original.replace('name = "t07_inventory"','name = "t07_inventory"\ntest = false'),
                      original + '\n[[test]]\nname="t07_inventory"\npath="tests/t07_inventory.rs"\n']
            for fault in faults:
                self.assertNotEqual(original, fault); manifest.write_text(fault)
                self.assertTrue(quality.has_t07(root))
                with self.subTest(fault=fault[-100:]), self.assertRaises(ValueError):
                    quality.rust_test_partitions(root)
            manifest.write_text(original)
            self.assertEqual(quality.rust_test_partitions(root), quality.rust_test_partitions(ROOT))

    def test_t07_missing_changed_duplicate_or_filtered_census_refuses(self):
        expected = quality.rust_test_expectations(ROOT)
        valid = self.synthetic_combined_output(expected)
        row = self.summary(51)
        for fault in [valid.replace(row, "", 1), valid.replace(row,self.summary(50),1),
                      valid+row, valid.replace(row,row.replace("0 filtered out","1 filtered out"),1),
                      valid.replace(row,row.replace("0 ignored","1 ignored"),1)]:
            with self.subTest(fault=fault[-100:]), self.assertRaisesRegex(ValueError, "required Rust"):
                quality.require_rust_test_summaries(fault, expected, "T07 census fault")
        quality.require_rust_test_summaries(valid, expected, "T07 census benign")

    def test_t07_partition_selects_inventory_once_without_frontend_tests(self):
        rows = quality.rust_test_partitions(ROOT)
        self.assertEqual([name for name,_ in rows], ["tests-main", "tests-t06", "tests-t21-process", "tests-t07-inventory", "tests-t08-native", "tests-recovery", "tests-t08-contract"])
        self.assertEqual(rows[3][1], ["--test", "t07_inventory"])
        targets = [args[i+1] for _,args in rows for i,value in enumerate(args) if value == "--test"]
        self.assertEqual(targets.count("t07_inventory"), 1)
        self.assertEqual(len(targets), 42)
        self.assertEqual(len(set(targets)), 42)
        for name in ["recovery_cancel", "recovery_inspect", "recovery_crash", "recovery_pi_queue", "frontend_controls"]:
            self.assertNotIn(name, targets)

    def test_t07_partition_omitted_or_repeated_return_refuses(self):
        expected = quality.rust_test_expectations(ROOT)
        inventory = self.summary(51)
        for observed in ["", inventory+inventory, inventory.replace("51 passed","50 passed")]:
            with self.subTest(observed=observed), self.assertRaisesRegex(ValueError, "required Rust test count (?:50|51)"):
                quality.run_rust_test_partitions(ROOT, self.serve(expected, {"t07_inventory": observed}), "cargo", [], "T07-return-fault", expected)
        quality.run_rust_test_partitions(ROOT, self.serve(expected, {}), "cargo", [], "T07-return-benign", expected)

    def test_t08_native_fixed_23_census_and_owned_inputs(self):
        expected = quality.rust_test_expectations(ROOT)
        self.assertEqual(sum(expected["test_counts"]), 1840)
        self.assertEqual(len(expected["test_counts"]), 44)
        self.assertEqual(expected["test_counts"].count(23), 1)
        self.assertEqual(expected["test_counts"].count(51), 2)
        self.assertEqual(sum(expected["unit_test_counts"].values()), 186)
        self.assertNotIn("worker::native::", "".join(expected["unit_test_counts"]))
        self.assertEqual(quality.T08_TARGET_COUNTS, {"t08_native": 23})
        self.assertEqual(quality.T08_INPUTS, ("src/worker/native.rs", "tests/t08_native.rs", "tests/fixtures/native/client.py"))
        paths = quality.quality_subject_paths(ROOT, time.monotonic() + 5, True)
        for name in quality.T08_INPUTS:
            self.assertIn(name, paths)
        self.assertFalse(any(name.startswith("development/t06/") for name in paths))
        # Exactly one control-v1 fixture is a gate input: the request table the bash suite's
        # Envelope class replays (review N1). The rest stay unpinned.
        self.assertEqual({name for name in paths if name.startswith("tests/fixtures/native/control-v1/")},
                         {"tests/fixtures/native/control-v1/actions.json",
                      "tests/fixtures/native/control-v1/receiver-oracle.py",
                      "tests/fixtures/native/control-v1/C01-control-valid-cancel.jsonl",
                      "tests/fixtures/native/control-v1/C01-metadata.json"})

    def test_t08_native_battery_cannot_leave_while_the_adapter_stays(self):
        # The adapter source keeps the native battery required; the contract battery reads
        # the same adapter, so a subject with the contract battery and no native battery
        # is not one this recipe accepts. Removing the registration and the battery's own
        # files is refused by name; removing the adapter too breaks the contract inputs.
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root)
            manifest.write_text(self.strip_targets(manifest.read_text(), ["t08_native"]))
            for name in ("tests/t08_native.rs", "tests/fixtures/native/client.py"):
                (root / name).unlink()
            self.assertTrue(quality.has_t08(root))
            self.assertTrue(quality.has_t08_contract(root))
            with self.assertRaisesRegex(ValueError, "required extension input: tests/t08_native.rs"):
                quality.rust_test_expectations(root)
            (root / "tests/t08_native.rs").write_text("// stub\n")
            (root / "tests/fixtures/native/client.py").write_text("#!/usr/bin/python3\n")
            with self.assertRaisesRegex(ValueError, r"missing=\['t08_native'\] extra=\[\] duplicate=\[\]"):
                quality.rust_test_expectations(root)
            (root / "src/worker/native.rs").unlink()
            (root / "tests/t08_native.rs").unlink()
            (root / "tests/fixtures/native/client.py").unlink()
            self.assertFalse(quality.has_t08(root))
            with self.assertRaisesRegex(ValueError, "required extension input: src/worker/native.rs"):
                quality.rust_test_expectations(root)

    def test_t08_missing_or_substituted_owned_inputs_refuse(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); self.combined_fixture(root)
            for name in quality.T08_INPUTS:
                path = root / name; before = path.read_bytes(); path.unlink()
                self.assertTrue(quality.has_t08(root))
                with self.assertRaisesRegex(ValueError, "required extension input: " + re.escape(name)):
                    quality.rust_test_expectations(root)
                path.symlink_to(ROOT / name)
                with self.assertRaisesRegex(ValueError, "required extension input: " + re.escape(name)):
                    quality.rust_test_expectations(root)
                path.unlink(); path.write_bytes(before)
            self.assertEqual(sum(quality.rust_test_expectations(root)["test_counts"]), 1840)

    def test_t08_omitted_changed_or_conditional_target_refuses_by_name(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root); original = manifest.read_text()
            named = [(re.sub(r'\[\[test\]\]\s*name = "t08_native"\s*path = "tests/t08_native.rs"\s*', '', original),
                      r"missing=\['t08_native'\] extra=\[\] duplicate=\[\]"),
                     (original.replace('name = "t08_native"', 'name = "other_native"'),
                      r"missing=\['t08_native'\] extra=\['other_native'\] duplicate=\[\]"),
                     (original + '\n[[test]]\nname="t08_native"\npath="tests/t08_native.rs"\n',
                      r"missing=\[\] extra=\[\] duplicate=\['t08_native'\]")]
            for fault, diagnostic in named:
                self.assertNotEqual(original, fault); manifest.write_text(fault)
                self.assertTrue(quality.has_t08(root))
                with self.subTest(diagnostic=diagnostic), self.assertRaisesRegex(ValueError, diagnostic):
                    quality.rust_test_expectations(root)
            faults = [original.replace('path = "tests/t08_native.rs"', 'path = "tests/t06_store.rs"'),
                      original.replace('name = "t08_native"', 'name = "t08_native"\nharness = false'),
                      original.replace('name = "t08_native"', 'name = "t08_native"\nrequired-features = ["hidden"]'),
                      original.replace('name = "t08_native"', 'name = "t08_native"\ntest = false')]
            for fault, _ in named:
                faults.append(fault)
            for fault in faults:
                self.assertNotEqual(original, fault); manifest.write_text(fault)
                with self.subTest(fault=fault[-100:]), self.assertRaises(ValueError):
                    quality.rust_test_partitions(root)
            manifest.write_text(original)
            self.assertEqual(quality.rust_test_partitions(root), quality.rust_test_partitions(ROOT))

    def test_t08_missing_changed_duplicate_or_filtered_census_refuses(self):
        expected = quality.rust_test_expectations(ROOT)
        valid = self.synthetic_combined_output(expected)
        row = self.summary(23)
        self.assertEqual(valid.count(row), 1)
        # A 20-row already exists (t06_staging); rewriting 21 to 20 is refused as
        # the first wrong multiplicity in census order, which is that group.
        for fault in [valid.replace(row, "", 1), valid.replace(row, self.summary(22), 1), valid + row,
                      valid.replace(row, row.replace("0 filtered out", "1 filtered out"), 1),
                      valid.replace(row, row.replace("0 ignored", "1 ignored"), 1),
                      valid.replace(row, row.replace("0 failed", "1 failed"), 1)]:
            with self.subTest(fault=fault[-100:]), self.assertRaisesRegex(ValueError, "required Rust test count (?:22|23)"):
                quality.require_rust_test_summaries(fault, expected, "T08 census fault")
        quality.require_rust_test_summaries(valid, expected, "T08 census benign")

    def test_t08_partition_selects_native_once_after_inventory(self):
        rows = quality.rust_test_partitions(ROOT)
        self.assertEqual([name for name, _ in rows], ["tests-main", "tests-t06", "tests-t21-process", "tests-t07-inventory", "tests-t08-native", "tests-recovery", "tests-t08-contract"])
        self.assertEqual(rows[4][1], ["--test", "t08_native"])
        self.assertNotIn("t08_native", rows[0][1])
        self.assertNotIn("t08_native", rows[1][1])
        targets = [args[i + 1] for _, args in rows for i, value in enumerate(args) if value == "--test"]
        self.assertEqual(targets.count("t08_native"), 1)
        self.assertEqual(len(targets), 42)
        self.assertEqual(len(set(targets)), 42)
        # The partition world is the manifest's declared test inventory, every target once.
        declared = quality.tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(sorted(targets), sorted(target["name"] for target in declared["test"]))

    def test_t08_partition_omitted_or_repeated_return_refuses(self):
        expected = quality.rust_test_expectations(ROOT)
        native = self.summary(23)
        for observed in ["", native + native, native.replace("23 passed", "22 passed")]:
            with self.subTest(observed=observed), self.assertRaisesRegex(ValueError, "required Rust test count (?:22|23)"):
                quality.run_rust_test_partitions(ROOT, self.serve(expected, {"t08_native": observed}), "cargo", [], "T08-return-fault", expected)
        quality.run_rust_test_partitions(ROOT, self.serve(expected, {}), "cargo", [], "T08-return-benign", expected)

    def test_native_client_interpreter_pin_reads_fixture_shebang(self):
        pin = quality.native_client_interpreter(ROOT)
        first = (ROOT / quality.NATIVE_CLIENT_FIXTURE).read_bytes().split(b"\n", 1)[0]
        self.assertEqual(first, b"#!" + pin["path"].encode())
        self.assertTrue(Path(pin["path"]).is_absolute())
        self.assertEqual(pin["sha256"], hashlib.sha256(Path(pin["path"]).read_bytes()).hexdigest())
        self.assertEqual(pin["resolved"], str(Path(pin["path"]).resolve(strict=True)))
        self.assertEqual(sorted(pin), ["path", "resolved", "sha256"])
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw).resolve()
            fixture = root / quality.NATIVE_CLIENT_FIXTURE
            fixture.parent.mkdir(parents=True)
            interpreter = root / "interpreter"
            interpreter.write_bytes(b"#!/bin/sh\n")
            line = b"#!" + str(interpreter).encode()
            fixture.write_bytes(line + b"\nprint()\n")
            self.assertEqual(quality.native_client_interpreter(root),
                             {"path": str(interpreter), "resolved": str(interpreter),
                              "sha256": hashlib.sha256(b"#!/bin/sh\n").hexdigest()})
            for content, diagnostic in ((b"print()\n", "Unsupported"), (b"#!relative/python\n", "Unsupported"),
                                        (b"#!/usr/bin/env python3\n", "Unsupported"), (line, "Unsupported"),
                                        (b"#!" + str(root / "missing").encode() + b"\n", "Missing"),
                                        (b"#!/" + b"x" * 4096 + b"\n", "Unsupported")):
                fixture.write_bytes(content)
                with self.subTest(content=content[:40]), self.assertRaisesRegex(ValueError, diagnostic + " native client interpreter"):
                    quality.native_client_interpreter(root)
            alias = root / "alias"
            alias.symlink_to(interpreter)
            fixture.write_bytes(b"#!" + str(alias).encode() + b"\n")
            self.assertEqual(quality.native_client_interpreter(root)["resolved"], str(interpreter))
            (root / "directory").mkdir()
            fixture.write_bytes(b"#!" + str(root / "directory").encode() + b"\n")
            with self.assertRaisesRegex(ValueError, "Missing native client interpreter"):
                quality.native_client_interpreter(root)

    def test_t08_interpreter_and_python_pins_are_recorded_and_rechecked(self):
        source = ast.parse((ROOT / "tools/check-quality").read_text())
        main = next(node for node in source.body if isinstance(node, ast.FunctionDef) and node.name == "main")
        text = ast.unparse(main)
        self.assertIn("report['executables']['native-client-interpreter'] = native_client_interpreter(ROOT)", text)
        self.assertIn("if has_t08(ROOT):\n            report['executables']['native-client-interpreter']", text)
        self.assertIn("'resolved': str(Path(sys.executable).resolve(strict=True))", text)
        self.assertIn("Pinned interpreter changed during quality checks", text)
        self.assertEqual(text.count("for name in ('python', 'native-client-interpreter', 'native-daemon-stand-in', 'contract-client-interpreter', 'contract-daemon-stand-in', 'ruff'):"), 1)
        # The recheck reads the pin's own path and digest, after the Rust commands.
        recheck = text.index("Pinned interpreter changed")
        self.assertGreater(recheck, text.index("run_rust_test_partitions(ROOT, run, cargo, common, label, test_expectations, parallel_main)"))
        self.assertIn("required_text='Ran 105 tests' if has_t09(ROOT) else", text)
        self.assertIn("'Ran 93 tests' if has_t08_contract(ROOT) or has_recovery(ROOT) else", text)

    def test_t06_partition_holds_every_t06_target_once_and_nothing_else(self):
        rows = quality.rust_test_partitions(ROOT)
        self.assertEqual(len(rows), 7)
        self.assertEqual(rows[1][0], "tests-t06")
        t06 = rows[1][1]
        self.assertEqual(t06[::2], ["--test"] * (len(t06) // 2))
        declared = quality.tomllib.loads((ROOT / "Cargo.toml").read_text())
        expected = sorted(target["name"] for target in declared["test"] if target["name"].startswith("t06_"))
        self.assertEqual(len(expected), 18)
        self.assertEqual(sorted(t06[1::2]), expected)
        self.assertFalse(any(name.startswith("t06_") for name in rows[0][1][1::2]))
        for _, arguments in rows[2:]:
            self.assertFalse(any(name.startswith("t06_") for name in arguments[1::2]))
        # The split oracle: T06 summaries served by the T06 command, nothing served twice.
        expected_counts = quality.rust_test_expectations(ROOT)
        t06_rows = "".join(self.summary(n) for n in quality.T06_TARGET_COUNTS.values())
        quality.run_rust_test_partitions(ROOT, self.serve(expected_counts, {}), "cargo", [], "T06-split-benign", expected_counts)
        for fault in ("", t06_rows + self.summary(64), t06_rows.replace(self.summary(64), "", 1)):
            with self.subTest(fault=fault[-80:]), self.assertRaisesRegex(ValueError, "required Rust"):
                quality.run_rust_test_partitions(ROOT, self.serve(expected_counts, {"t06": fault}), "cargo", [], "T06-split-fault", expected_counts)

    def test_recovery_and_contract_fixed_1840_census_and_owned_inputs(self):
        expected = quality.rust_test_expectations(ROOT)
        self.assertEqual(sum(expected["test_counts"]), 1840)
        self.assertEqual(len(expected["test_counts"]), 44)
        # Three targets hold 60: the T02 battery, recovery (since T07-RC-59/60) and t22_cohort (since
        # the review D5/N3 cases) -- the multiplicity rule of the 24-rows below.
        # T02 and recovery hold 60; t22_cohort left the group at 67.
        self.assertEqual(expected["test_counts"].count(60), 2)
        # t08_contract (24 since T08C-24) shared 24 with t01_contracts until the store lane moved
        # t01_contracts to 25 (A03) and then 26 (A24, 2026-09-24). At 26 it shares a count with
        # another target, so require_rust_test_summaries still compares a real multiplicity and the
        # census-fault control below still has a shared count to alter.
        self.assertEqual(expected["test_counts"].count(24), 1)
        self.assertEqual(expected["test_counts"].count(25), 0)
        self.assertEqual(expected["test_counts"].count(26), 2)
        self.assertEqual(quality.BUDGET_TARGET_COUNTS, {"accounting": 81})
        self.assertEqual(sum(expected["unit_test_counts"].values()), 186)
        self.assertNotIn("recovery::", "".join(expected["unit_test_counts"]))
        self.assertEqual(quality.RECOVERY_TARGET_COUNTS, {"recovery": 60})
        self.assertEqual(quality.T08_CONTRACT_TARGET_COUNTS, {"t08_contract": 24})
        paths = quality.quality_subject_paths(ROOT, time.monotonic() + 5, True)
        for name in (*quality.RECOVERY_INPUTS, *quality.T08_CONTRACT_INPUTS):
            self.assertIn(name, paths)
        for row in quality.T08_CONTRACT_EXAMPLES:
            self.assertIn(row["path"], quality.T08_CONTRACT_INPUTS)
        # Exactly one control-v1 fixture is a gate input: the request table the bash suite's
        # Envelope class replays (review N1). The rest stay unpinned.
        self.assertEqual({name for name in paths if name.startswith("tests/fixtures/native/control-v1/")},
                         {"tests/fixtures/native/control-v1/actions.json",
                      "tests/fixtures/native/control-v1/receiver-oracle.py",
                      "tests/fixtures/native/control-v1/C01-control-valid-cancel.jsonl",
                      "tests/fixtures/native/control-v1/C01-metadata.json"})
        self.assertNotIn("tests/fixtures/native/README.stub.md", paths)
        self.assertFalse(any(name.startswith("development/t06/") for name in paths))
        # The inputs are what the sources read: every include of the battery is pinned.
        included = re.findall(r'include_(?:str|bytes)!\("([^"]+)"\)', (ROOT / "tests/t08_contract.rs").read_text())
        self.assertTrue(included)
        for relative in included:
            self.assertIn("tests/" + relative, quality.T08_CONTRACT_INPUTS)
        self.assertEqual(sorted(set(re.findall(r"habitat_engine::(\w+)", (ROOT / "tests/recovery.rs").read_text()))), ["recovery"])

    def test_recovery_absent_keeps_1780_and_43_groups(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root)
            manifest.write_text(self.strip_targets(manifest.read_text(), ["recovery"]))
            for name in quality.RECOVERY_INPUTS:
                (root / name).unlink()
            self.assertFalse(quality.has_recovery(root))
            self.assertTrue(quality.has_t08_contract(root))
            expected = quality.rust_test_expectations(root)
            self.assertEqual(sum(expected["test_counts"]), 1780)
            self.assertEqual(len(expected["test_counts"]), 43)
            self.assertEqual(expected["test_counts"].count(60), 1)
            quality.require_rust_test_summaries(self.synthetic_combined_output(expected), expected, "recovery-absent")
            self.assertEqual([n for n, _ in quality.rust_test_partitions(root)], ["tests-main", "tests-t06", "tests-t21-process", "tests-t07-inventory", "tests-t08-native", "tests-t08-contract"])
            self.assertNotIn("src/recovery.rs", quality.quality_subject_paths(root, time.monotonic() + 5, True))

    def test_contract_absent_keeps_1816_and_43_groups_and_no_examples(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root)
            manifest.write_text(self.strip_targets(manifest.read_text(), ["t08_contract"]))
            for name in quality.T08_CONTRACT_OWN_SOURCES:
                (root / name).unlink()
            self.assertFalse(quality.has_t08_contract(root))
            self.assertTrue(quality.has_t08(root))
            self.assertEqual(quality.expected_examples(root), [])
            expected = quality.rust_test_expectations(root)
            self.assertEqual(sum(expected["test_counts"]), 1816)
            self.assertEqual(len(expected["test_counts"]), 43)
            # t08_contract was the only 24-row.
            self.assertEqual(expected["test_counts"].count(24), 0)
            quality.require_rust_test_summaries(self.synthetic_combined_output(expected), expected, "contract-absent")
            self.assertEqual([n for n, _ in quality.rust_test_partitions(root)], ["tests-main", "tests-t06", "tests-t21-process", "tests-t07-inventory", "tests-t08-native", "tests-recovery"])
            self.assertNotIn("tests/fixtures/native/contract-client.py", quality.quality_subject_paths(root, time.monotonic() + 5, True))

    def test_recovery_or_contract_missing_or_substituted_inputs_refuse(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); self.combined_fixture(root)
            for name in (*quality.RECOVERY_INPUTS, *quality.T08_CONTRACT_INPUTS):
                path = root / name; before = path.read_bytes(); path.unlink()
                self.assertTrue(quality.has_recovery(root) and quality.has_t08_contract(root))
                with self.subTest(name=name), self.assertRaisesRegex(ValueError, "required (?:extension input|T03 source|example source): " + re.escape(name)):
                    quality.rust_test_expectations(root)
                path.symlink_to(ROOT / name)
                with self.subTest(name=name, kind="symlink"), self.assertRaisesRegex(ValueError, "required (?:extension input|T03 source|example source): " + re.escape(name)):
                    quality.rust_test_expectations(root)
                path.unlink(); path.write_bytes(before)
            self.assertEqual(sum(quality.rust_test_expectations(root)["test_counts"]), 1840)

    def test_recovery_or_contract_omitted_changed_or_conditional_target_refuses_by_name(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root); original = manifest.read_text()
            named = [(self.strip_targets(original, ["recovery"]), r"missing=\['recovery'\] extra=\[\] duplicate=\[\]"),
                     (original.replace('name = "t08_contract"', 'name = "t08_battery"'), r"missing=\['t08_contract'\] extra=\['t08_battery'\] duplicate=\[\]"),
                     (original + '\n[[test]]\nname="recovery"\npath="tests/recovery.rs"\n', r"missing=\[\] extra=\[\] duplicate=\['recovery'\]")]
            for fault, diagnostic in named:
                self.assertNotEqual(original, fault); manifest.write_text(fault)
                self.assertTrue(quality.has_recovery(root) and quality.has_t08_contract(root))
                with self.subTest(diagnostic=diagnostic), self.assertRaisesRegex(ValueError, diagnostic):
                    quality.rust_test_expectations(root)
            # Removing only the contract registration leaves its sources, so the census still names it.
            contract_gone = re.sub(r'\[\[test\]\]\s*name = "t08_contract"\s*path = "tests/t08_contract.rs"\s*', '', original)
            manifest.write_text(contract_gone)
            with self.assertRaisesRegex(ValueError, r"missing=\['t08_contract'\]"):
                quality.rust_test_expectations(root)
            faults = [original.replace('path = "tests/recovery.rs"', 'path = "tests/t06_store.rs"'),
                      original.replace('name = "t08_contract"', 'name = "t08_contract"\nharness = false'),
                      original.replace('name = "recovery"', 'name = "recovery"\nrequired-features = ["hidden"]'),
                      original.replace('name = "t08_contract"', 'name = "t08_contract"\ntest = false')]
            for fault in faults + [fault for fault, _ in named]:
                self.assertNotEqual(original, fault); manifest.write_text(fault)
                with self.subTest(fault=fault[-100:]), self.assertRaises(ValueError):
                    quality.rust_test_partitions(root)
            manifest.write_text(original)
            self.assertEqual(quality.rust_test_partitions(root), quality.rust_test_partitions(ROOT))

    def test_example_inventory_must_match_the_contract_battery_exactly(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root); original = manifest.read_text()
            self.assertEqual(quality.expected_examples(root), list(quality.T08_CONTRACT_EXAMPLES))
            faults = {"extra": original + '\n[[example]]\nname="extra"\npath="src/main.rs"\n',
                      "renamed": original.replace('name = "t08-contract-cohort"', 'name = "t08-contract-cohorts"'),
                      "repathed": original.replace('path = "tests/fixtures/native/driver.rs"', 'path = "tests/fixtures/native/contract-cohort.rs"'),
                      "removed": re.sub(r'\[\[example\]\]\s*name = "t08-native-profile-driver"\s*path = "[^"]+"\s*', '', original),
                      "reordered": re.sub(r'\[\[example\]\]\s*name = "t08-native-profile-driver"\s*path = "[^"]+"\s*', '', original) + '\n[[example]]\nname = "t08-native-profile-driver"\npath = "tests/fixtures/native/driver.rs"\n'}
            for label, fault in faults.items():
                self.assertNotEqual(original, fault); manifest.write_text(fault)
                with self.subTest(label=label), self.assertRaisesRegex(ValueError, "required example target inventory: manifest="):
                    quality.rust_test_expectations(root)
                with self.subTest(label=label, stage="partition"), self.assertRaisesRegex(ValueError, "required example target inventory"):
                    quality.rust_test_partitions(root)
            manifest.write_text(original)
            source = root / "tests/fixtures/native/contract-cohort.rs"
            before = source.read_bytes(); source.unlink()
            with self.assertRaisesRegex(ValueError, "required extension input: tests/fixtures/native/contract-cohort.rs"):
                quality.rust_test_expectations(root)
            source.write_bytes(before)
            # Examples without the battery that owns them are unaccounted.
            manifest.write_text(self.strip_targets(original, ["t08_contract"]) + "".join(
                f'\n[[example]]\nname = "{row["name"]}"\npath = "{row["path"]}"\n' for row in quality.T08_CONTRACT_EXAMPLES))
            for name in quality.T08_CONTRACT_OWN_SOURCES:
                (root / name).unlink()
            self.assertFalse(quality.has_t08_contract(root))
            with self.assertRaisesRegex(ValueError, r"required example target inventory: manifest=\[\('t08-native-profile-driver'"):
                quality.rust_test_expectations(root)

    def test_recovery_and_contract_census_faults_refuse(self):
        expected = quality.rust_test_expectations(ROOT)
        valid = self.synthetic_combined_output(expected)
        for count in (60, 24):
            row = self.summary(count)
            self.assertEqual(valid.count(row), expected["test_counts"].count(count))
            for fault in [valid.replace(row, "", 1), valid.replace(row, self.summary(count - 1), 1), valid + row,
                          valid.replace(row, row.replace("0 filtered out", "1 filtered out"), 1),
                          valid.replace(row, row.replace("0 ignored", "1 ignored"), 1)]:
                with self.subTest(count=count, fault=fault[-100:]), self.assertRaisesRegex(ValueError, "required Rust test count"):
                    quality.require_rust_test_summaries(fault, expected, "census fault")
        quality.require_rust_test_summaries(valid, expected, "census benign")

    def test_partition_selects_recovery_and_contract_once_after_native(self):
        rows = quality.rust_test_partitions(ROOT)
        self.assertEqual([name for name, _ in rows][-2:], ["tests-recovery", "tests-t08-contract"])
        self.assertEqual(rows[5][1], ["--test", "recovery"])
        self.assertEqual(rows[6][1], ["--test", "t08_contract"])
        targets = [args[i + 1] for _, args in rows for i, value in enumerate(args) if value == "--test"]
        self.assertEqual((targets.count("recovery"), targets.count("t08_contract")), (1, 1))
        declared = quality.tomllib.loads((ROOT / "Cargo.toml").read_text())
        self.assertEqual(sorted(targets), sorted(target["name"] for target in declared["test"]))
        self.assertNotIn("recovery", rows[0][1]); self.assertNotIn("t08_contract", rows[0][1])

    def test_partition_recovery_or_contract_omitted_or_repeated_return_refuses(self):
        expected = quality.rust_test_expectations(ROOT)
        for name, count in (("recovery", 60), ("t08_contract", 24)):
            row = self.summary(count)
            for observed in ["", row + row, row.replace(f"{count} passed", f"{count - 1} passed")]:
                with self.subTest(name=name, observed=observed), self.assertRaisesRegex(ValueError, "required Rust test count"):
                    quality.run_rust_test_partitions(ROOT, self.serve(expected, {name: observed}), "cargo", [], "return-fault", expected)
        quality.run_rust_test_partitions(ROOT, self.serve(expected, {}), "cargo", [], "return-benign", expected)

    def test_contract_client_interpreter_and_daemon_stand_in_pins(self):
        pin = quality.native_client_interpreter(ROOT, quality.CONTRACT_CLIENT_FIXTURE)
        first = (ROOT / quality.CONTRACT_CLIENT_FIXTURE).read_bytes().split(b"\n", 1)[0]
        self.assertEqual(first, b"#!" + pin["path"].encode())
        self.assertEqual(pin["sha256"], hashlib.sha256(Path(pin["path"]).read_bytes()).hexdigest())
        for source in ("tests/t08_contract.rs", "tests/t08_native.rs"):
            stand_in = quality.daemon_stand_in(ROOT, source, required=True)
            declared = re.findall(r"const EXECUTABLE: &'static str = \"(/[^\"]+)\";", (ROOT / source).read_text())
            self.assertEqual(declared, [stand_in["path"]])
            self.assertEqual(stand_in["sha256"], hashlib.sha256(Path(stand_in["path"]).read_bytes()).hexdigest())
            self.assertEqual(stand_in["bytes"], Path(stand_in["path"]).stat().st_size)
            self.assertEqual(stand_in["declared_in"], source)
            self.assertEqual(stand_in["resolved"], str(Path(stand_in["path"]).resolve(strict=True)))
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            source = root / "battery.rs"
            source.write_text("// no declaration\n")
            self.assertIsNone(quality.daemon_stand_in(root, "battery.rs", required=False))
            with self.assertRaisesRegex(ValueError, "Missing or ambiguous daemon stand-in declaration: battery.rs"):
                quality.daemon_stand_in(root, "battery.rs", required=True)
            source.write_text('const EXECUTABLE: &\'static str = "/usr/bin/sleep";\nconst EXECUTABLE: &\'static str = "/usr/bin/true";\n')
            with self.assertRaisesRegex(ValueError, "ambiguous daemon stand-in declaration"):
                quality.daemon_stand_in(root, "battery.rs", required=False)
            source.write_text('const EXECUTABLE: &\'static str = "' + str(root / "absent") + '";\n')
            with self.assertRaisesRegex(ValueError, "Missing daemon stand-in executable"):
                quality.daemon_stand_in(root, "battery.rs", required=True)
        text = ast.unparse(next(node for node in ast.parse((ROOT / "tools/check-quality").read_text()).body if isinstance(node, ast.FunctionDef) and node.name == "main"))
        self.assertIn("report['executables']['contract-client-interpreter'] = native_client_interpreter(ROOT, CONTRACT_CLIENT_FIXTURE)", text)
        self.assertIn("report['executables']['contract-daemon-stand-in'] = daemon_stand_in(ROOT, 'tests/t08_contract.rs', required=True)", text)
        self.assertIn("stand_in = daemon_stand_in(ROOT, 'tests/t08_native.rs', required=False)", text)

    def test_t09_fixed_72_route_census_and_owned_inputs(self):
        expected = quality.rust_test_expectations(ROOT)
        self.assertEqual(sum(expected["test_counts"]), 1840)
        self.assertEqual(len(expected["test_counts"]), 44)
        self.assertEqual(expected["test_counts"].count(72), 1)
        self.assertEqual(sum(expected["unit_test_counts"].values()), 186)
        self.assertNotIn("route::", "".join(expected["unit_test_counts"]))
        self.assertEqual(quality.T09_TARGET_COUNTS, {"t09_route": 72})
        paths = quality.quality_subject_paths(ROOT, time.monotonic() + 5, True)
        for name in quality.T09_INPUTS:
            self.assertIn(name, paths)
        # The inputs are what the battery reads: every include of it is pinned, and the
        # router itself reads no module outside contracts and its own configuration.
        included = re.findall(r'include_str!\("([^"]+)"\)', (ROOT / "tests/t09_route.rs").read_text())
        self.assertTrue(included)
        for relative in included:
            resolved = str((Path("tests") / relative).as_posix()).replace("tests/../", "")
            self.assertIn(resolved, quality.T09_INPUTS)
        self.assertEqual(sorted(set(re.findall(r"habitat_engine::(\w+)", (ROOT / "tests/t09_route.rs").read_text()))), ["contracts", "route"])

    def test_t09_absent_keeps_1768_and_43_groups(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root)
            manifest.write_text(self.strip_targets(manifest.read_text(), ["t09_route"]))
            for name in quality.T09_INPUTS:
                (root / name).unlink()
            self.assertFalse(quality.has_t09(root))
            self.assertTrue(quality.has_t08_contract(root))
            expected = quality.rust_test_expectations(root)
            self.assertEqual(sum(expected["test_counts"]), 1768)
            self.assertEqual(len(expected["test_counts"]), 43)
            self.assertEqual(expected["test_counts"].count(72), 0)
            quality.require_rust_test_summaries(self.synthetic_combined_output(expected), expected, "t09-absent")
            self.assertEqual([n for n, _ in quality.rust_test_partitions(root)],
                             ["tests-main", "tests-t06", "tests-t21-process", "tests-t07-inventory",
                              "tests-t08-native", "tests-recovery", "tests-t08-contract"])
            for name in quality.T09_INPUTS:
                self.assertNotIn(name, quality.quality_subject_paths(root, time.monotonic() + 5, True))

    def test_t09_missing_or_substituted_owned_inputs_refuse(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); self.combined_fixture(root)
            for name in quality.T09_INPUTS:
                path = root / name; before = path.read_bytes(); path.unlink()
                self.assertTrue(quality.has_t09(root))
                with self.subTest(name=name), self.assertRaisesRegex(ValueError, "required extension input: " + re.escape(name)):
                    quality.rust_test_expectations(root)
                path.symlink_to(ROOT / name)
                with self.subTest(name=name, kind="symlink"), self.assertRaisesRegex(ValueError, "required extension input: " + re.escape(name)):
                    quality.rust_test_expectations(root)
                path.unlink(); path.write_bytes(before)
            self.assertEqual(sum(quality.rust_test_expectations(root)["test_counts"]), 1840)

    def test_t09_omitted_changed_or_duplicated_target_refuses_by_name(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw); manifest = self.combined_fixture(root); original = manifest.read_text()
            named = [(self.strip_targets(original, ["t09_route"]), r"missing=\['t09_route'\] extra=\[\] duplicate=\[\]"),
                     (original.replace('name = "t09_route"', 'name = "t09_routing"'), r"missing=\['t09_route'\] extra=\['t09_routing'\] duplicate=\[\]"),
                     (original + '\n[[test]]\nname="t09_route"\npath="tests/t09_route.rs"\n', r"missing=\[\] extra=\[\] duplicate=\['t09_route'\]")]
            for fault, diagnostic in named:
                self.assertNotEqual(original, fault); manifest.write_text(fault)
                # The battery's own sources stay present, so the census still names it.
                self.assertTrue(quality.has_t09(root))
                with self.subTest(diagnostic=diagnostic), self.assertRaisesRegex(ValueError, diagnostic):
                    quality.rust_test_expectations(root)
                with self.subTest(diagnostic=diagnostic, stage="partition"), self.assertRaisesRegex(ValueError, diagnostic):
                    quality.rust_test_partitions(root)
            manifest.write_text(original.replace('path = "tests/t09_route.rs"', 'path = "tests/t01_task.rs"'))
            with self.assertRaisesRegex(ValueError, "Substituted required T06 target: t09_route"):
                quality.rust_test_expectations(root)
            manifest.write_text(original)
            self.assertEqual(sum(quality.rust_test_expectations(root)["test_counts"]), 1840)

    def test_t09_census_faults_refuse(self):
        expected = quality.rust_test_expectations(ROOT)
        valid = self.synthetic_combined_output(expected)
        row = self.summary(72)
        self.assertEqual(valid.count(row), 1)
        for fault in [valid.replace(row, "", 1), valid.replace(row, self.summary(60), 1), valid + row,
                      valid.replace(row, row.replace("0 filtered out", "1 filtered out"), 1),
                      valid.replace(row, row.replace("0 ignored", "1 ignored"), 1)]:
            with self.subTest(fault=fault[-100:]), self.assertRaisesRegex(ValueError, "required Rust test count"):
                quality.require_rust_test_summaries(fault, expected, "t09 census fault")
        quality.require_rust_test_summaries(valid, expected, "t09 census benign")

if __name__ == "__main__":
    unittest.main(verbosity=2)
