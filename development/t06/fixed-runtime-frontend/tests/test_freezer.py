"""Synthetic metadata/owned-file controls; no runtime or tool probes."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock
import sys
sys.dont_write_bytecode=True
BASE=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('freezer',BASE/'prepare-inputs.py');freezer=importlib.util.module_from_spec(spec);spec.loader.exec_module(freezer)
spec=importlib.util.spec_from_file_location('comparator',BASE/'compare-result.py');comparator=importlib.util.module_from_spec(spec);spec.loader.exec_module(comparator)
spec=importlib.util.spec_from_file_location('build_record',BASE/'write-build-record.py');build_record=importlib.util.module_from_spec(spec);spec.loader.exec_module(build_record)

class FreezerControls(unittest.TestCase):
    def test_current_three_workspaces_declare_required_release_profile(self):
        profile=build_record.configured_profile()
        self.assertEqual((profile['opt_level'],profile['overflow_checks'],profile['lto'],profile['debug']),
                         (2,True,'thin','line-tables-only'))
    def test_separate_workspace_cannot_inherit_missing_parent_release_profile(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'Cargo.toml').write_text('[workspace]\n[profile.test]\ndebug=0\n')
            with mock.patch.object(build_record,'RUNTIME',root),self.assertRaisesRegex(ValueError,'workspace profile mismatch'):
                build_record.configured_profile()
    def test_stripping_debug_information_refuses_even_with_other_profile_fields(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);raw=(BASE/'Cargo.toml').read_text();(root/'Cargo.toml').write_text(raw.replace('[profile.release]','[profile.release]\nstrip="symbols"'))
            with mock.patch.object(build_record,'BASE',root),self.assertRaisesRegex(ValueError,'debug information'):
                build_record.configured_profile()
    def test_only_runtime_test_profile_accepts_original_zero_debug(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);raw=(build_record.RUNTIME/'Cargo.toml').read_text()
            (root/'Cargo.toml').write_text(raw)
            with mock.patch.object(build_record,'RUNTIME',root):
                self.assertEqual(build_record.configured_profile()['debug'],'line-tables-only')
            for field in ('BASE','PROJECT'):
                with self.subTest(field=field),mock.patch.object(build_record,field,root),self.assertRaisesRegex(ValueError,'profile override'):
                    build_record.configured_profile()
            for replacement in ('debug = 1','debug = "none"','debug = false','debug = 0\ncodegen-units = 2'):
                (root/'Cargo.toml').write_text(raw.replace('debug = 0',replacement))
                with self.subTest(replacement=replacement),mock.patch.object(build_record,'RUNTIME',root),self.assertRaisesRegex(ValueError,'profile override'):
                    build_record.configured_profile()
    def binary_fixture(self, root):
        binary=root/'release/hee3-fixed-runtime-frontend';binary.parent.mkdir();binary.write_bytes(b'bound compiler output')
        pin=freezer.pin(binary)
        return {'environment':{'CARGO_TARGET_DIR':str(root)}},{'built_binary':pin},pin
    def test_binary_derivation_accepts_only_observed_build_path(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);command,result,binary=self.binary_fixture(root)
            self.assertEqual(build_record.validate_binary_derivation(command,result,binary,None,None),{})
            alternate=root/'other-binary';alternate.write_bytes(Path(binary['path']).read_bytes())
            with self.assertRaisesRegex(ValueError,'not the observed build output'):
                build_record.validate_binary_derivation(command,result,freezer.pin(alternate),None,None)
    def test_missing_build_output_observation_refuses(self):
        with tempfile.TemporaryDirectory() as d:
            command,_,binary=self.binary_fixture(Path(d))
            with self.assertRaisesRegex(ValueError,'not bound at build completion'):
                build_record.validate_binary_derivation(command,{},binary,None,None)
    def test_packaging_record_requires_exact_reviewed_digest(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);command,result,binary=self.binary_fixture(root)
            record=root/'package.json';record.write_text('{}')
            with self.assertRaisesRegex(ValueError,'changed reviewed packaging'):
                build_record.validate_binary_derivation(command,result,binary,record,'sha256:'+'0'*64)
    def test_packaging_from_another_binary_refuses_even_when_record_is_pinned(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);command,result,binary=self.binary_fixture(root)
            record=root/'package.json';record.write_text(json.dumps({'kind':'hee3-elf-debug-compression/1',
                'completed':True,'source':dict(binary,sha256='sha256:'+'0'*64),'binary':binary}))
            with self.assertRaisesRegex(ValueError,'does not derive'):
                build_record.validate_binary_derivation(command,result,binary,record,freezer.sha(record.read_bytes()))
    def test_freezer_rejects_self_consistent_different_workspace_sources(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);_,_,binary=self.binary_fixture(root)
            record=root/'old-build.json';record.write_text(json.dumps({'kind':'hee3-fixed-frontend-build/1',
                'completed':True,'binary':binary,'sources':[binary]}))
            with self.assertRaisesRegex(ValueError,'current workspace'):
                freezer.build_inputs(record)
    def configuration_fixture(self, root):
        tool='/var/home/Louranicas/.rustup/toolchains/1.98.0-x86_64-unknown-linux-gnu/bin'
        sqlite='/var/home/Louranicas/.cache/hee3-implementation/T05/sqlite-inputs'
        base=root/'subject';base.mkdir();home=root/'cargo-home';home.mkdir()
        config=home/'config.toml'
        config.write_text('[net]\noffline=true\n[source.crates-io]\nreplace-with="hee3-closed"\n'
                          '[source.hee3-closed]\ndirectory="/var/home/Louranicas/.cache/hee3-implementation/T05/candidate/rust-vendor"\n')
        environment={'PATH':tool+':/usr/bin:/bin','LANG':'C','LC_ALL':'C',
                     'CARGO_HOME':str(home),'CARGO_NET_OFFLINE':'true','CARGO_TARGET_DIR':str(root/'target'),
                     'RUSTC':tool+'/rustc','RUSTDOC':tool+'/rustdoc','RUSTUP_TOOLCHAIN':'1.98.0',
                     'CARGO_BUILD_JOBS':'2','RUST_TEST_THREADS':'2','RUSTFLAGS':'-Dwarnings',
                     'RUSTDOCFLAGS':'-Dwarnings','TMPDIR':str(root/'tmp'),
                     'SQLITE3_NO_PKG_CONFIG':'1','SQLITE3_STATIC':'1','SQLITE3_LIB_DIR':sqlite,
                     'SQLITE3_INCLUDE_DIR':sqlite,'LIBSQLITE3_SYS_USE_PKG_CONFIG':'0','PYTHONDONTWRITEBYTECODE':'1'}
        command={'environment':environment,'cwd':str(base),
                 'argv':[tool+'/cargo','build','--offline','--locked','--quiet',
                         '--target-dir',str(root/'target'),'--release']}
        inputs={str(config):{'sha256':freezer.sha(config.read_bytes()).removeprefix('sha256:'),'bytes':config.stat().st_size}}
        return base,config,command,inputs
    def test_owned_build_configuration_accepts_pinned_offline_routing(self):
        with tempfile.TemporaryDirectory() as d:
            base,_,command,inputs=self.configuration_fixture(Path(d))
            with mock.patch.object(build_record,'BASE',base):
                build_record.validate_build_configuration(command,inputs,'build')
    def test_encoded_flags_wrappers_and_target_environment_refuse(self):
        for key in ('CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_RUSTFLAGS','CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS',
                    'CARGO_BUILD_TARGET','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','CARGO_PROFILE_RELEASE_OPT_LEVEL'):
            with self.subTest(key=key),tempfile.TemporaryDirectory() as d:
                base,_,command,inputs=self.configuration_fixture(Path(d))
                command['environment'][key]='unreviewed'
                with mock.patch.object(build_record,'BASE',base),self.assertRaisesRegex(ValueError,'environment override'):
                    build_record.validate_build_configuration(command,inputs,'build')
    def test_command_profile_target_and_config_overrides_refuse(self):
        for args in (['--target','aarch64-unknown-linux-gnu'],['--config=build.rustflags=[]'],['--profile','dev']):
            with self.subTest(args=args),tempfile.TemporaryDirectory() as d:
                base,_,command,inputs=self.configuration_fixture(Path(d));command['argv']+=args
                with mock.patch.object(build_record,'BASE',base),self.assertRaisesRegex(ValueError,'command override'):
                    build_record.validate_build_configuration(command,inputs,'build')
    def test_even_pinned_cargo_config_cannot_override_rustflags(self):
        with tempfile.TemporaryDirectory() as d:
            base,config,command,inputs=self.configuration_fixture(Path(d))
            config.write_text(config.read_text()+'[build]\nrustflags=["-C", "overflow-checks=off"]\n')
            inputs[str(config)]={'sha256':freezer.sha(config.read_bytes()).removeprefix('sha256:'),'bytes':config.stat().st_size}
            with mock.patch.object(build_record,'BASE',base),self.assertRaisesRegex(ValueError,'configuration override'):
                build_record.validate_build_configuration(command,inputs,'build')
    def test_ancestor_cargo_config_requires_input_binding(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);base,config,command,inputs=self.configuration_fixture(root)
            (root/'.cargo').mkdir();(root/'.cargo/config.toml').write_bytes(config.read_bytes())
            with mock.patch.object(build_record,'BASE',base),self.assertRaisesRegex(ValueError,'unbound Cargo'):
                build_record.validate_build_configuration(command,inputs,'build')
    def test_copy_is_fresh_nonalias_exact_and_readonly(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);(p/'source').write_bytes(b'fixed source\n');result=freezer.copy(p/'source',p/'target')
            self.assertEqual(result['sha256'],freezer.sha(b'fixed source\n'))
            self.assertNotEqual((p/'source').stat().st_ino,(p/'target').stat().st_ino)
            self.assertEqual((p/'target').stat().st_mode & 0o777,0o400)
            with self.assertRaises(FileExistsError):freezer.copy(p/'source',p/'target')
    def test_selected_tool_alias_records_real_target_byte_hash(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);(p/'target').write_bytes(b'actual target bytes');(p/'alias').symlink_to(p/'target')
            self.assertEqual(freezer.pin(p/'alias'),freezer.pin(p/'target'))
            self.assertNotEqual(freezer.pin(p/'alias')['sha256'],freezer.sha(os.fsencode(p/'target')))
    def test_tree_binds_empty_directories_and_utf8_byte_order(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);(p/'empty').mkdir();(p/'z').write_bytes(b'z');(p/'é').write_bytes(b'e')
            inventory=freezer.tree(p)
            self.assertEqual(inventory['directories'],['empty'])
            self.assertEqual([r['path'] for r in inventory['files']],['z','é'])
            with tempfile.TemporaryDirectory() as copied:
                freezer.copy_sources(p,Path(copied))
                self.assertEqual(freezer.tree(Path(copied))['directories'],['empty'])
    def test_raw_file_adjacent_bound_refuses_without_publishing(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            with (p/'large').open('wb') as f:f.truncate(freezer.MAX_FILE+1)
            with self.assertRaisesRegex(ValueError,'16 MiB'):freezer.copy(p/'large',p/'target')
            self.assertFalse((p/'target').exists())
    def test_tree_refuses_symlink_content_even_if_target_is_regular(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);(p/'source').write_bytes(b'bytes');(p/'alias').symlink_to(p/'source')
            with self.assertRaisesRegex(ValueError,'alias'):freezer.tree(p)
    def test_every_generated_execution_dimension_is_a_distinct_uuid4(self):
        ids=freezer.identities();allids=[v for k,v in ids.items() if k!='attempts']
        for row in ids['attempts']:
            allids.extend(v for k,v in row.items() if k!='stages');allids.extend(row['stages'])
        self.assertEqual(len(allids),len(set(allids)))
        self.assertTrue(all(freezer.uuid.UUID(v).version==4 for v in allids))
    def test_review_file_substitution_does_not_import_a_new_approval(self):
        with tempfile.TemporaryDirectory() as d:
            path=Path(d)/'bundle';path.write_text('{"objects":[]}')
            with self.assertRaisesRegex(ValueError,'changed reviewed'):freezer.reviewed(path)

    def test_comparator_missing_observations_returns_structured_failure(self):
        with tempfile.TemporaryDirectory() as d:
            result=comparator.compare(Path(d))
            self.assertFalse(result['passed'])
            self.assertEqual(len([v for v in result['failures'] if v.get('reason')=='missing_or_invalid']),5)
    def test_comparator_wrong_json_shape_is_not_a_traceback(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            for name in ['frontend-result','driver-result','runtime-observation','store-readback','retained-graphs']:
                (p/(name+'.json')).write_text('[]')
            self.assertFalse(comparator.compare(p)['passed'])
    def test_comparator_synthetic_complete_pair_and_duplicate_outbox_refusal(self):
        # Synthetic comparator input only; never presented as an executed task.
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            records={'frontend-result':{'operation_returned_ok':True,'returned_error':None},
                     'driver-result':{'ok':True,'outcome':'Accepted','error':None},
                     'runtime-observation':{'executions':[{'receipt':{'state':'FAIL'}},{'receipt':{'state':'PASS_CANDIDATE'}}]},
                     'store-readback':{'head':{'state':'accepted','accepted_event':'event','cancellation':False,'reserved_work_ms':0,'reserved_verify_ms':0},'head_error':None,'outbox_error':None,'outbox':[['event','uid:operator',1]]},
                     'retained-graphs':{'roots':[{'objects':1},{'objects':1}]}}
            for name,value in records.items():(p/(name+'.json')).write_text(json.dumps(value))
            self.assertTrue(comparator.compare(p)['passed'])
            records['store-readback']['outbox']*=2
            (p/'store-readback.json').write_text(json.dumps(records['store-readback']))
            self.assertFalse(comparator.compare(p)['passed'])

if __name__=='__main__':unittest.main()
