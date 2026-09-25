//! Independent metadata-consistency oracles; design.md was frozen before bodies.
//! Structural record shapes use the prior schema author's inventory-examples.json.
//! All placeholder refs are replaced by real retained hashes. The memory reader and
//! SHA helper adapt this agent's independent graph fixture, not production logic.
//! Synthetic producer/review assertions establish no custody or authentication.

use habitat_engine::check::consistency::{
    CasePlan, Error, PatchRefusal, Prepared, Summary, validate,
};
use habitat_engine::check::graph::{Error as GraphError, Graph, Objects};
use habitat_engine::check::patch;
use habitat_engine::contracts::receipt::{
    Id, List, Name, ReceiptRecord, ReceiptV1, Ref, Sha, TypedRef, decode,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{Cursor, Read};
use std::sync::OnceLock;

const CASE: &str = "case-independent-001";
const ORACLE: &str = "fixture-reference";
const DETECTOR: &str = "fixture-equality";
const SCHEMA: &str = "sha256:b926a67e80bc326af582898c3e865fd50618786ef27b71d1fd2f66523776e62b";
const EXAMPLES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/receipts/inventory-examples.json"
));

type Refs = BTreeMap<String, Ref>;

#[derive(Default)]
struct Memory {
    bytes: BTreeMap<String, Vec<u8>>,
    next: u64,
}

fn digest(bytes: &[u8]) -> String {
    let mut text = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

impl Objects for Memory {
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, GraphError> {
        let bytes = self
            .bytes
            .get(reference.artifact_id.as_str())
            .ok_or(GraphError::Missing)?;
        Ok(Box::new(Cursor::new(bytes.as_slice())))
    }
}

impl Memory {
    fn insert(&mut self, schema: &str, bytes: Vec<u8>) -> Ref {
        self.next += 1;
        let reference = Ref {
            artifact_id: Id::new(format!("08000000-0000-4000-8000-{:012x}", self.next)).unwrap(),
            sha256: Sha::new(digest(&bytes)).unwrap(),
            byte_length: u32::try_from(bytes.len()).unwrap(),
            media_type: Name::new(if schema == "hee3.raw/1" {
                "application/octet-stream"
            } else {
                "application/json"
            })
            .unwrap(),
            schema_id: Name::new(schema).unwrap(),
        };
        assert!(
            self.bytes
                .insert(reference.artifact_id.as_str().to_owned(), bytes)
                .is_none()
        );
        reference
    }
    fn raw(&mut self, bytes: &[u8]) -> Ref {
        self.insert("hee3.raw/1", bytes.to_vec())
    }
    fn typed(&mut self, name: &str, value: &Value) -> Ref {
        self.insert(
            &format!("hee3.receipt/1:{name}"),
            serde_json::to_vec(value).unwrap(),
        )
    }
    fn value(&self, reference: &Ref) -> Value {
        serde_json::from_slice(&self.bytes[reference.artifact_id.as_str()]).unwrap()
    }
}

fn sample(name: &str) -> Value {
    static RECORDS: OnceLock<Value> = OnceLock::new();
    RECORDS.get_or_init(|| serde_json::from_str(EXAMPLES).unwrap())["definitions"][name].clone()
}
fn value<T: serde::Serialize>(input: &T) -> Value {
    serde_json::to_value(input).unwrap()
}
fn dto<T: ReceiptRecord>(input: &Value) -> T {
    decode(&serde_json::to_vec(input).unwrap()).unwrap()
}
fn unavailable(reason: &str) -> Value {
    json!({"value":null,"unavailable_reason":reason})
}
fn present(input: &Value) -> Value {
    json!({"value":input,"unavailable_reason":null})
}
fn page(rows: &[Value]) -> Value {
    json!({"page_index":0,"page_count":1,"row_count":rows.len(),"total_rows":rows.len(),"rows":rows,"next":unavailable("end_of_inventory")})
}
fn put(memory: &mut Memory, refs: &mut Refs, key: &str, name: &str, input: &Value) {
    refs.insert(key.to_owned(), memory.typed(name, input));
}
fn producer(log: &Ref, exit: u32) -> Value {
    json!({"status":"exited","exit_code":present(&json!(exit)),"signal":unavailable("exited"),
        "timeout":false,"stdout":present(&value(log)),"stderr":present(&value(log))})
}
fn payload(reference: &Ref) -> Value {
    json!({"object":reference,"role":"produced","required":true,"truncated":false,"availability":"available","reason":""})
}

fn foundations(memory: &mut Memory) -> Refs {
    let mut refs = Refs::new();
    for (name, bytes) in [
        ("log", b"".as_slice()),
        ("patch", b"--- a/candidate.rs\n+++ b/candidate.rs\n@@ -1 +1 @@\n-seed fixture\n+candidate result\n".as_slice()),
        ("seed_bytes", b"seed fixture\n".as_slice()),
        ("result_bytes", b"candidate result\n".as_slice()),
        ("spec", b"independent expectation bytes\n".as_slice()),
        ("profile", b"development metadata fixture only\n".as_slice()),
    ] {
        refs.insert(name.to_owned(), memory.raw(bytes));
    }
    for name in [
        "AssumptionPageV1",
        "FindingPageV1",
        "ObligationPageV1",
        "LockPageV1",
        "ToolPageV1",
        "LanguageFlagsPageV1",
        "StandardPageV1",
        "EnvironmentPageV1",
        "GrantPageV1",
        "EffectPageV1",
        "ResourcePageV1",
        "MissingObjectPageV1",
    ] {
        put(memory, &mut refs, name, name, &page(&[]));
    }
    for (key, raw, path) in [
        ("seed", "seed_bytes", "candidate.rs"),
        ("result", "result_bytes", "candidate.rs"),
    ] {
        let row = json!({"path":path,"kind":"file","content":present(&value(&refs[raw])),"executable":false,
            "link_target":unavailable("regular file"),"origin":"authored","exclusion_reason":unavailable("included")});
        let files = memory.typed("SubjectFilePageV1", &page(&[row]));
        let subject = json!({"subject_id":"18000000-0000-4000-8000-000000000001","files":files,
            "tree_sha256":files.sha256,"dirty_patch":unavailable("clean")});
        put(memory, &mut refs, key, "SubjectV1", &subject);
    }
    let mut expectation = sample("ExpectationV1");
    expectation["oracle_id"] = json!(ORACLE);
    expectation["intended_detector"] = json!(DETECTOR);
    expectation["specification"] = value(&refs["spec"]);
    expectation["shared_assumptions"] = value(&refs["AssumptionPageV1"]);
    expectation["expected_producer"] =
        json!({"status":"exited","exit_code":present(&json!(0)),"signal":unavailable("exited")});
    put(
        memory,
        &mut refs,
        "expectation",
        "ExpectationV1",
        &expectation,
    );
    let mut review = sample("ReviewV1");
    review["reviewer"] = json!("independent synthetic design reviewer");
    review["action"] = json!("case_design_review");
    review["disposition"] = json!("accepted");
    review["subject_sha256"] = value(&refs["expectation"].sha256);
    for (field, name) in [
        ("shared_assumptions", "AssumptionPageV1"),
        ("findings", "FindingPageV1"),
        ("residual_obligations", "ObligationPageV1"),
    ] {
        review[field] = value(&refs[name]);
    }
    review["pre_fix_evidence"] = json!([]);
    review["post_fix_evidence"] = json!([]);
    put(memory, &mut refs, "review", "ReviewV1", &review);
    for name in ["LimitsV1", "HostV1", "BuildProfileV1", "CleanupContractV1"] {
        let mut record = sample(name);
        match name {
            "HostV1" => record["facts"] = value(&refs["profile"]),
            "BuildProfileV1" => record["language_flags"] = value(&refs["LanguageFlagsPageV1"]),
            "CleanupContractV1" => {
                record["obligations"] = value(&refs["ObligationPageV1"]);
                record["readback_specification"] = value(&refs["spec"]);
            }
            _ => {}
        }
        put(memory, &mut refs, name, name, &record);
    }
    refs
}

fn root_bindings(refs: &Refs) -> Value {
    let mut root = sample("ReceiptV1");
    root["schema_sha256"] = json!(SCHEMA);
    root["identity"]["criterion_ids"] = json!(["C1"]);
    root["identity"]["profile_id"] = json!("TH-DEV-consistency-fixture");
    for field in [
        "seed_subject",
        "fixtures",
        "oracle",
        "harness",
        "collector",
        "launcher",
    ] {
        root["subjects"][field] = value(&refs["seed"]);
    }
    for (field, name) in [
        ("locks", "LockPageV1"),
        ("toolchain", "ToolPageV1"),
        ("target_features_build_profile", "BuildProfileV1"),
        ("standards", "StandardPageV1"),
        ("isolation_profile", "profile"),
    ] {
        root["subjects"][field] = value(&refs[name]);
    }
    root["subjects"]["result_subject"] = present(&value(&refs["result"]));
    root["subjects"]["seed_to_result_patch"] = present(&value(&refs["patch"]));
    for (field, name) in [
        ("environment", "EnvironmentPageV1"),
        ("grants", "GrantPageV1"),
        ("expected", "expectation"),
        ("limits", "LimitsV1"),
        ("allowed_effects", "EffectPageV1"),
        ("cleanup_contract", "CleanupContractV1"),
    ] {
        root["invocation"][field] = value(&refs[name]);
    }
    root["invocation"]["oracle_id"] = json!(ORACLE);
    root["observations"]["host"] = value(&refs["HostV1"]);
    root["observations"]["resources"] = value(&refs["ResourcePageV1"]);
    root["observations"]["unresolved_obligations"] = value(&refs["ObligationPageV1"]);
    root["observations"]["producer"] = producer(&refs["log"], 0);
    root["cases"]["primary_credit"] = json!(1);
    root["artifacts"]["count"] = json!(3);
    root["artifacts"]["total_bytes"] =
        json!((refs["result_bytes"].byte_length + refs["patch"].byte_length).to_string());
    root["availability"]["missing_objects"] = value(&refs["MissingObjectPageV1"]);
    root["verdict"]["intended_detector"] = json!(DETECTOR);
    root
}

struct Fixture {
    memory: Memory,
    refs: Refs,
    root: Value,
    prepared: Prepared,
    rows: Vec<Value>,
    diagnostics: Vec<Value>,
    artifacts: Vec<Value>,
    campaigns: Vec<Value>,
}

impl Fixture {
    fn new() -> Self {
        let mut memory = Memory::default();
        let mut refs = foundations(&mut memory);
        let mut root = root_bindings(&refs);
        let mut oracle = sample("OracleResultV1");
        oracle["oracle_id"] = json!(ORACLE);
        oracle["expected"] = value(&refs["expectation"]);
        oracle["detector_id"] = present(&json!(DETECTOR));
        oracle["raw_evidence_refs"] = json!([refs["log"]]);
        put(
            &mut memory,
            &mut refs,
            "oracle_result",
            "OracleResultV1",
            &oracle,
        );
        root["verdict"]["oracle_result"] = value(&refs["oracle_result"]);
        let mut row = sample("CaseV1");
        row["case_id"] = json!(CASE);
        row["criterion_ids"] = json!(["C1"]);
        row["fixture_sha256"] = value(&refs["seed_bytes"].sha256);
        row["expected"] = value(&refs["expectation"]);
        row["oracle_id"] = json!(ORACLE);
        row["detector_id"] = json!(DETECTOR);
        row["producer_exit_or_signal"] = producer(&refs["log"], 0);
        row["benign_pair_id"] = root["verdict"]["benign_pair"].clone();
        row["raw_evidence_refs"] = json!([refs["log"], refs["review"]]);
        let mut diagnostic = sample("DiagnosticV1");
        diagnostic["stdout"] = value(&refs["log"]);
        diagnostic["stderr"] = value(&refs["log"]);
        diagnostic["producer"] = producer(&refs["log"], 0);
        let plan = CasePlan {
            case_id: Name::new(CASE).unwrap(),
            primary_module_id: Name::new("check").unwrap(),
            criterion_ids: List::new(vec![Name::new("C1").unwrap()]).unwrap(),
            fixture_sha256: refs["seed_bytes"].sha256.clone(),
            oracle_id: Name::new(ORACLE).unwrap(),
            expected: TypedRef::new(refs["expectation"].clone()).unwrap(),
            mandatory: true,
            selected: true,
            excluded: false,
            reviewed_design: Some(TypedRef::new(refs["review"].clone()).unwrap()),
        };
        let prepared = Prepared {
            schema_sha256: Sha::new(SCHEMA).unwrap(),
            identity: dto(&root["identity"]),
            subjects: dto(&root["subjects"]),
            invocation: dto(&root["invocation"]),
            cases: vec![plan],
        };
        let artifacts = vec![
            payload(&refs["log"]),
            payload(&refs["result_bytes"]),
            payload(&refs["patch"]),
        ];
        Self {
            memory,
            refs,
            root,
            prepared,
            rows: vec![row],
            diagnostics: vec![diagnostic],
            artifacts,
            campaigns: vec![],
        }
    }

    fn materialize(&mut self) -> TypedRef<ReceiptV1> {
        for (section, field, name, rows) in [
            ("cases", "inventory", "CasePageV1", &self.rows),
            (
                "diagnostics",
                "by_tool",
                "DiagnosticPageV1",
                &self.diagnostics,
            ),
            ("artifacts", "inventory", "ArtifactPageV1", &self.artifacts),
            ("mutation", "campaigns", "CampaignPageV1", &self.campaigns),
        ] {
            self.root[section][field] = value(&self.memory.typed(name, &page(rows)));
        }
        TypedRef::new(self.memory.typed("ReceiptV1", &self.root)).unwrap()
    }

    fn check(mut self) -> Result<Summary, Error> {
        let root = self.materialize();
        let graph = Graph::resolve(&self.memory, root.as_ref())
            .expect("fixture graph must resolve before testing consistency");
        validate(&graph, &root, &self.prepared)
    }

    fn reject(self) {
        assert!(self.check().is_err(), "consistency defect was accepted");
    }

    fn rebind_invocation(&mut self, field: &str, name: &str, record: &Value) {
        self.root["invocation"][field] = value(&self.memory.typed(name, record));
        self.prepared.invocation = dto(&self.root["invocation"]);
    }

    fn replace_review(&mut self, record: &Value) {
        let reference = self.memory.typed("ReviewV1", record);
        self.rows[0]["raw_evidence_refs"][1] = value(&reference);
        self.prepared.cases[0].reviewed_design = Some(TypedRef::new(reference).unwrap());
    }

    fn replace_oracle(&mut self, record: &Value) {
        self.root["verdict"]["oracle_result"] = value(&self.memory.typed("OracleResultV1", record));
    }

    fn optional(&mut self, excluded: bool) {
        let mut row = self.rows[0].clone();
        row["case_id"] = json!("optional-002");
        row["mandatory"] = json!(false);
        row["selected"] = json!(false);
        row["executed"] = json!(false);
        row["excluded"] = json!(excluded);
        row["outcome"] = json!("unmeasured");
        row["reason"] = json!("predeclared optional inventory");
        self.rows.push(row);
        self.prepared.cases.push(CasePlan {
            case_id: Name::new("optional-002").unwrap(),
            primary_module_id: Name::new("check").unwrap(),
            criterion_ids: List::new(vec![Name::new("C1").unwrap()]).unwrap(),
            fixture_sha256: self.refs["seed_bytes"].sha256.clone(),
            oracle_id: Name::new(ORACLE).unwrap(),
            expected: TypedRef::new(self.refs["expectation"].clone()).unwrap(),
            mandatory: false,
            selected: false,
            excluded,
            reviewed_design: None,
        });
        self.root["cases"]["discovered"] = json!(2);
        self.root["cases"][if excluded { "excluded" } else { "unmeasured" }] = json!(1);
    }

    fn recount_artifacts(&mut self) {
        self.root["artifacts"]["count"] = json!(self.artifacts.len());
        let total: u64 = self
            .artifacts
            .iter()
            .map(|row| row["object"]["byte_length"].as_u64().unwrap())
            .sum();
        self.root["artifacts"]["total_bytes"] = json!(total.to_string());
    }

    fn campaign(&mut self) -> (Value, Value, Value) {
        self.materialize();
        let mut baseline = self.root.clone();
        baseline["identity"]["run_id"] = json!("28000000-0000-4000-8000-000000000001");
        let baseline_ref = self.memory.typed("ReceiptV1", &baseline);
        let diff = self
            .memory
            .raw(b"--- a/candidate.rs\n+++ b/candidate.rs\n@@ -1 +1 @@\n-candidate result\n+mutated result\n");
        let mut mutant = sample("MutantV1");
        mutant["baseline_subject_sha256"] = value(&self.refs["result"].sha256);
        mutant["diff"] = value(&diff);
        mutant["raw_evidence_refs"] = json!([self.refs["log"]]);
        mutant["expected_detector"] = json!(DETECTOR);
        mutant["observed_detector"] = present(&json!(DETECTOR));
        let mut campaign = sample("CampaignV1");
        campaign["tool"] = value(&self.refs["spec"]);
        campaign["config"] = value(&self.refs["profile"]);
        campaign["baseline_receipt"] = value(&baseline_ref);
        (campaign, mutant, baseline)
    }

    fn install_campaign(&mut self, mut campaign: Value, mutant: &Value) {
        campaign["mutants"] = value(
            &self
                .memory
                .typed("MutantPageV1", &page(std::slice::from_ref(mutant))),
        );
        self.campaigns = vec![campaign];
        self.root["mutation"]["campaign_count"] = json!(1);
        self.root["mutation"]["distinct_mutants"] = json!(1);
        let field = match mutant["outcome"].as_str().unwrap() {
            "caught" => "caught",
            "reviewed-equivalent" => "equivalent",
            "timeout" => "timed_out",
            other => other,
        };
        self.root["mutation"][field] = json!(1);
    }
}

#[test]
fn complete_real_hash_fixture_matches_independently_prepared_case_catalogue() {
    let summary = Fixture::new().check().unwrap();
    assert_eq!(
        (
            summary.cases,
            summary.primary_credit,
            summary.mandatory_cases
        ),
        (1, 1, 1)
    );
}

#[test]
fn receipt_schema_digest_must_equal_the_frozen_prepared_schema() {
    let mut fixture = Fixture::new();
    fixture.root["schema_sha256"] = value(&fixture.refs["spec"].sha256);
    fixture.reject();
}

#[test]
fn all_root_identity_dimensions_are_bound_to_preparation() {
    for (field, replacement) in [
        ("run_id", json!("38000000-0000-4000-8000-000000000001")),
        ("task_id", json!("38000000-0000-4000-8000-000000000002")),
        ("attempt_id", json!("38000000-0000-4000-8000-000000000003")),
        ("generation", json!("2")),
        ("module_id", json!("store")),
        ("criterion_ids", json!(["C2"])),
        ("profile_id", json!("other-profile")),
    ] {
        let mut fixture = Fixture::new();
        fixture.root["identity"][field] = replacement;
        fixture.reject();
    }
}

#[test]
fn seed_fixture_oracle_harness_collector_and_launcher_subjects_cannot_be_swapped() {
    for field in [
        "seed_subject",
        "fixtures",
        "oracle",
        "harness",
        "collector",
        "launcher",
    ] {
        let mut fixture = Fixture::new();
        fixture.root["subjects"][field] = value(&fixture.refs["result"]);
        fixture.reject();
    }
}

#[test]
fn invocation_arguments_working_scope_and_oracle_identity_are_frozen() {
    for (field, replacement) in [
        ("argv", json!(["different-command"])),
        ("cwd_logical", json!("other-root")),
        ("oracle_id", json!("other-oracle")),
    ] {
        let mut fixture = Fixture::new();
        fixture.root["invocation"][field] = replacement;
        fixture.reject();
    }
}

#[test]
fn missing_extra_and_duplicate_predeclared_cases_are_refused() {
    let mut absent = Fixture::new();
    absent.prepared.cases.clear();
    absent.reject();
    let mut extra = Fixture::new();
    extra.optional(false);
    extra.rows.pop();
    extra.root["cases"]["discovered"] = json!(1);
    extra.root["cases"]["unmeasured"] = json!(0);
    extra.reject();
    let mut duplicate = Fixture::new();
    duplicate.optional(false);
    duplicate.prepared.cases[1].case_id = Name::new(CASE).unwrap();
    duplicate.reject();
}

#[test]
fn case_selection_requiredness_and_exclusion_cannot_change_after_preparation() {
    for field in ["mandatory", "selected", "excluded"] {
        let mut fixture = Fixture::new();
        match field {
            "mandatory" => fixture.prepared.cases[0].mandatory = false,
            "selected" => fixture.prepared.cases[0].selected = false,
            _ => fixture.prepared.cases[0].excluded = true,
        }
        fixture.reject();
    }
}

#[test]
fn case_primary_owner_criteria_fixture_oracle_and_expectation_are_frozen() {
    for field in ["owner", "criteria", "fixture", "oracle", "expectation"] {
        let mut fixture = Fixture::new();
        match field {
            "owner" => fixture.prepared.cases[0].primary_module_id = Name::new("store").unwrap(),
            "criteria" => {
                fixture.prepared.cases[0].criterion_ids =
                    List::new(vec![Name::new("C2").unwrap()]).unwrap();
            }
            "fixture" => {
                fixture.prepared.cases[0].fixture_sha256 =
                    fixture.refs["result_bytes"].sha256.clone();
            }
            "oracle" => fixture.prepared.cases[0].oracle_id = Name::new("other-oracle").unwrap(),
            _ => {
                let record = fixture.memory.value(&fixture.refs["expectation"]);
                let other = fixture.memory.typed("ExpectationV1", &record);
                fixture.prepared.cases[0].expected = TypedRef::new(other).unwrap();
            }
        }
        fixture.reject();
    }
}

#[test]
fn every_case_counter_is_recomputed_from_unique_observed_rows() {
    for field in [
        "discovered",
        "selected",
        "executed",
        "passed",
        "failed",
        "skipped",
        "ignored",
        "broken",
        "timed_out",
        "invalid",
        "excluded",
        "unmeasured",
        "primary_credit",
    ] {
        let mut fixture = Fixture::new();
        let prior = fixture.root["cases"][field].as_u64().unwrap();
        fixture.root["cases"][field] = json!(prior + 1);
        fixture.reject();
    }
}

#[test]
fn unselected_optional_inventory_is_counted_without_becoming_a_required_failure() {
    let mut fixture = Fixture::new();
    fixture.optional(false);
    let summary = fixture.check().unwrap();
    assert_eq!(
        (
            summary.cases,
            summary.primary_credit,
            summary.mandatory_cases
        ),
        (2, 1, 1)
    );
}

#[test]
fn excluded_rows_count_only_as_excluded_not_again_as_unmeasured() {
    let mut benign = Fixture::new();
    benign.optional(true);
    assert_eq!(benign.check().unwrap().cases, 2);
    let mut double_count = Fixture::new();
    double_count.optional(true);
    double_count.root["cases"]["unmeasured"] = json!(1);
    double_count.reject();
}

#[test]
fn pass_requires_a_nonempty_mandatory_selection() {
    let mut fixture = Fixture::new();
    fixture.rows[0]["mandatory"] = json!(false);
    fixture.prepared.cases[0].mandatory = false;
    fixture.reject();
}

#[test]
fn any_nonpassed_mandatory_outcome_refuses_pass_with_truthful_counts() {
    for outcome in [
        "failed",
        "skipped",
        "ignored",
        "broken",
        "timeout",
        "invalid",
        "unmeasured",
    ] {
        let mut fixture = Fixture::new();
        fixture.rows[0]["outcome"] = json!(outcome);
        fixture.rows[0]["reason"] = json!("actual required fixture did not pass");
        fixture.root["cases"]["passed"] = json!(0);
        fixture.root["cases"]["primary_credit"] = json!(0);
        fixture.root["cases"][if outcome == "timeout" {
            "timed_out"
        } else {
            outcome
        }] = json!(1);
        fixture.reject();
    }
}

#[test]
fn unexecuted_mandatory_case_cannot_hide_behind_an_executed_optional_neighbor() {
    let mut fixture = Fixture::new();
    fixture.optional(false);
    fixture.rows[0]["executed"] = json!(false);
    fixture.rows[1]["selected"] = json!(true);
    fixture.rows[1]["executed"] = json!(true);
    fixture.rows[1]["outcome"] = json!("passed");
    fixture.rows[1]["reason"] = json!("");
    fixture.prepared.cases[1].selected = true;
    fixture.root["cases"]["selected"] = json!(2);
    fixture.root["cases"]["passed"] = json!(2);
    fixture.root["cases"]["unmeasured"] = json!(0);
    fixture.root["cases"]["primary_credit"] = json!(0);
    fixture.reject();
}

#[test]
fn mandatory_cases_must_cover_every_declared_root_criterion() {
    let mut fixture = Fixture::new();
    fixture.root["identity"]["criterion_ids"] = json!(["C1", "C2"]);
    fixture.prepared.identity = dto(&fixture.root["identity"]);
    fixture.reject();
}

#[test]
fn absent_design_review_has_no_primary_credit_and_cannot_support_pass() {
    let mut fault = Fixture::new();
    fault.prepared.cases[0].reviewed_design = None;
    fault.root["cases"]["primary_credit"] = json!(0);
    fault.reject();
    let mut nonpass = Fixture::new();
    nonpass.prepared.cases[0].reviewed_design = None;
    nonpass.root["cases"]["primary_credit"] = json!(0);
    nonpass.root["verdict"]["state"] = json!("UNMEASURED");
    nonpass.root["verdict"]["reasons"] = json!(["design review pending"]);
    assert_eq!(nonpass.check().unwrap().primary_credit, 0);
}

#[test]
fn reviewed_design_subject_must_be_the_exact_frozen_expectation_digest() {
    let mut fixture = Fixture::new();
    let mut review = fixture.memory.value(&fixture.refs["review"]);
    review["subject_sha256"] = value(&fixture.refs["seed"].sha256);
    fixture.replace_review(&review);
    fixture.reject();
}

#[test]
fn design_review_must_be_anchored_in_the_case_even_if_elsewhere_in_the_graph() {
    let mut fixture = Fixture::new();
    fixture.rows[0]["raw_evidence_refs"] = json!([fixture.refs["log"]]);
    let mut oracle = fixture.memory.value(&fixture.refs["oracle_result"]);
    oracle["raw_evidence_refs"] = json!([fixture.refs["log"], fixture.refs["review"]]);
    fixture.replace_oracle(&oracle);
    fixture.reject();
}

#[test]
fn review_action_and_disposition_must_authorize_case_design_credit() {
    for (field, text) in [("action", "unrelated_review"), ("disposition", "pending")] {
        let mut fixture = Fixture::new();
        let mut review = fixture.memory.value(&fixture.refs["review"]);
        review[field] = json!(text);
        fixture.replace_review(&review);
        fixture.reject();
    }
}

#[test]
fn material_design_findings_require_actual_closed_dispositions() {
    for (disposition, qualified) in [
        ("open", false),
        ("accepted_residual", false),
        ("fixed", true),
        ("not_reproduced", true),
        ("out_of_scope", true),
    ] {
        for is_pass in [true, false] {
            let mut fixture = Fixture::new();
            let mut finding = sample("FindingV1");
            finding["disposition"] = json!(disposition);
            for field in ["evidence", "pre_fix_evidence", "post_fix_evidence"] {
                finding[field] = json!([fixture.refs["log"]]);
            }
            let inventory = fixture.memory.typed("FindingPageV1", &page(&[finding]));
            let mut review = fixture.memory.value(&fixture.refs["review"]);
            review["findings"] = value(&inventory);
            fixture.replace_review(&review);
            fixture.root["cases"]["primary_credit"] = json!(u32::from(qualified));
            if !is_pass {
                fixture.root["verdict"]["state"] = json!("UNMEASURED");
                fixture.root["verdict"]["reasons"] = json!(["retained design disposition"]);
            }
            assert_eq!(fixture.check().is_ok(), qualified || !is_pass);
        }
    }
}

#[test]
fn material_design_obligations_block_credit_until_settled() {
    for (material, state, qualified) in [
        (true, "open", false),
        (true, "unknown", false),
        (true, "settled", true),
        (false, "open", true),
    ] {
        for is_pass in [true, false] {
            let mut fixture = Fixture::new();
            let mut obligation = sample("ObligationV1");
            obligation["material"] = json!(material);
            obligation["state"] = json!(state);
            obligation["evidence"] = json!([fixture.refs["log"]]);
            let inventory = fixture
                .memory
                .typed("ObligationPageV1", &page(&[obligation]));
            let mut review = fixture.memory.value(&fixture.refs["review"]);
            review["residual_obligations"] = value(&inventory);
            fixture.replace_review(&review);
            fixture.root["cases"]["primary_credit"] = json!(u32::from(qualified));
            if !is_pass {
                fixture.root["verdict"]["state"] = json!("UNMEASURED");
                fixture.root["verdict"]["reasons"] = json!(["retained design obligation"]);
            }
            assert_eq!(fixture.check().is_ok(), qualified || !is_pass);
        }
    }
}

#[test]
fn a_case_owned_by_another_module_contributes_no_primary_credit() {
    let mut benign = Fixture::new();
    benign.rows[0]["primary_module_id"] = json!("store");
    benign.prepared.cases[0].primary_module_id = Name::new("store").unwrap();
    benign.root["cases"]["primary_credit"] = json!(0);
    assert_eq!(benign.check().unwrap().primary_credit, 0);
    let mut fault = Fixture::new();
    fault.rows[0]["primary_module_id"] = json!("store");
    fault.prepared.cases[0].primary_module_id = Name::new("store").unwrap();
    fault.reject();
}

#[test]
fn case_producer_must_match_expected_exit_without_equating_absence_reason_text() {
    let mut wrong_exit = Fixture::new();
    wrong_exit.rows[0]["producer_exit_or_signal"]["exit_code"] = present(&json!(1));
    wrong_exit.reject();
    let mut benign = Fixture::new();
    benign.rows[0]["producer_exit_or_signal"]["signal"]["unavailable_reason"] =
        json!("independently observed normal exit");
    assert_eq!(benign.check().unwrap().cases, 1);
}

#[test]
fn case_oracle_and_detector_must_match_the_frozen_expectation() {
    let mut oracle = Fixture::new();
    oracle.rows[0]["oracle_id"] = json!("wrong-oracle");
    oracle.prepared.cases[0].oracle_id = Name::new("wrong-oracle").unwrap();
    oracle.reject();
    let mut detector = Fixture::new();
    detector.rows[0]["detector_id"] = json!("wrong-detector");
    detector.reject();
}

#[test]
fn root_producer_exit_must_match_its_predeclared_expectation() {
    let mut fixture = Fixture::new();
    fixture.root["observations"]["producer"]["exit_code"] = present(&json!(1));
    fixture.reject();
}

#[test]
fn predeclared_nonzero_exit_is_a_valid_negative_control_not_a_blanket_failure() {
    let mut fixture = Fixture::new();
    let mut expectation = fixture.memory.value(&fixture.refs["expectation"]);
    expectation["expected_producer"]["exit_code"] = present(&json!(1));
    let expected = fixture.memory.typed("ExpectationV1", &expectation);
    fixture.root["invocation"]["expected"] = value(&expected);
    fixture.prepared.invocation = dto(&fixture.root["invocation"]);
    fixture.prepared.cases[0].expected = TypedRef::new(expected.clone()).unwrap();
    fixture.rows[0]["expected"] = value(&expected);
    fixture.rows[0]["producer_exit_or_signal"]["exit_code"] = present(&json!(1));
    fixture.root["observations"]["producer"]["exit_code"] = present(&json!(1));
    let mut review = fixture.memory.value(&fixture.refs["review"]);
    review["subject_sha256"] = value(&expected.sha256);
    fixture.replace_review(&review);
    let mut oracle = fixture.memory.value(&fixture.refs["oracle_result"]);
    oracle["expected"] = value(&expected);
    fixture.replace_oracle(&oracle);
    assert_eq!(fixture.check().unwrap().cases, 1);
}

#[test]
fn root_oracle_result_binds_expected_reference_oracle_and_detector() {
    for field in ["expected", "oracle_id", "detector_id", "result"] {
        let mut fixture = Fixture::new();
        let mut oracle = fixture.memory.value(&fixture.refs["oracle_result"]);
        match field {
            "expected" => {
                let body = fixture.memory.value(&fixture.refs["expectation"]);
                oracle[field] = value(&fixture.memory.typed("ExpectationV1", &body));
            }
            "oracle_id" => oracle[field] = json!("wrong-oracle"),
            "detector_id" => oracle[field] = present(&json!("wrong-detector")),
            _ => {
                oracle[field] = json!("violated");
                oracle["reason"] = json!("literal comparison differs");
            }
        }
        fixture.replace_oracle(&oracle);
        fixture.reject();
    }
    let mut fixture = Fixture::new();
    fixture.root["verdict"]["intended_detector"] = json!("wrong-detector");
    fixture.reject();
}

#[test]
fn diagnostic_totals_are_recomputed_from_all_tool_rows() {
    for field in ["warning_count", "error_count"] {
        let mut fixture = Fixture::new();
        fixture.root["diagnostics"]["baseline"] = json!(false);
        fixture.diagnostics[0]["baseline"] = json!(false);
        fixture.root["diagnostics"][field] = json!(1);
        fixture.reject();
    }
}

#[test]
fn a_fault_diagnostic_row_cannot_be_silently_counted_as_clean_baseline() {
    let mut fixture = Fixture::new();
    fixture.diagnostics[0]["baseline"] = json!(false);
    fixture.reject();
}

#[test]
fn tool_truncation_and_nonzero_tool_producer_refuse_clean_pass() {
    for field in ["stdout_truncated", "stderr_truncated", "producer"] {
        let mut fixture = Fixture::new();
        if field == "producer" {
            fixture.diagnostics[0][field]["exit_code"] = present(&json!(1));
        } else {
            fixture.diagnostics[0][field] = json!(true);
        }
        fixture.reject();
    }
}

#[test]
fn produced_payload_count_and_byte_total_exclude_undeclared_inputs() {
    assert_eq!(Fixture::new().check().unwrap().cases, 1);
    for field in ["count", "total_bytes"] {
        let mut fixture = Fixture::new();
        fixture.root["artifacts"][field] = if field == "count" {
            json!(4)
        } else {
            json!("999")
        };
        fixture.reject();
    }
}

#[test]
fn page_metadata_cannot_be_counted_as_a_produced_payload() {
    let mut fixture = Fixture::new();
    fixture
        .artifacts
        .push(payload(&fixture.refs["AssumptionPageV1"]));
    fixture.recount_artifacts();
    fixture.reject();
}

#[test]
fn required_payload_must_be_available_untruncated_and_marked_required() {
    for field in ["availability", "truncated", "required"] {
        let mut fixture = Fixture::new();
        fixture.artifacts[0][field] = match field {
            "availability" => json!("missing"),
            "truncated" => json!(true),
            _ => json!(false),
        };
        fixture.artifacts[0]["reason"] = json!("independent required-proof fault");
        fixture.reject();
    }
}

#[test]
fn missing_generated_logs_patch_or_result_content_refuses_even_with_correct_counts() {
    for index in [0, 1, 2] {
        let mut fixture = Fixture::new();
        fixture.artifacts.remove(index);
        fixture.recount_artifacts();
        fixture.reject();
    }
}

#[test]
fn empty_mutation_inventory_cannot_claim_campaign_or_mutant_credit() {
    for field in [
        "campaign_count",
        "distinct_mutants",
        "caught",
        "survived",
        "timed_out",
        "unviable",
        "equivalent",
        "excluded",
        "unmeasured",
    ] {
        let mut fixture = Fixture::new();
        fixture.root["mutation"][field] = json!(1);
        fixture.reject();
    }
}

#[test]
fn actual_mutation_campaign_matches_counts_and_exact_baseline_result_subject() {
    let mut fixture = Fixture::new();
    let (campaign, mutant, _) = fixture.campaign();
    fixture.install_campaign(campaign, &mutant);
    assert_eq!(fixture.check().unwrap().cases, 1);
}

#[test]
fn campaign_planned_count_and_mutant_campaign_identity_cannot_disagree() {
    for field in ["planned", "identity"] {
        let mut fixture = Fixture::new();
        let (mut campaign, mut mutant, _) = fixture.campaign();
        if field == "planned" {
            campaign["planned_mutants"] = json!(2);
        } else {
            mutant["campaign_id"] = json!("different-campaign");
        }
        fixture.install_campaign(campaign, &mutant);
        fixture.reject();
    }
}

#[test]
fn mutant_baseline_hash_cannot_name_seed_or_tree_instead_of_result_subject_record() {
    for field in ["seed", "tree"] {
        let mut fixture = Fixture::new();
        let (campaign, mut mutant, _) = fixture.campaign();
        mutant["baseline_subject_sha256"] = if field == "seed" {
            value(&fixture.refs["seed"].sha256)
        } else {
            fixture.memory.value(&fixture.refs["result"])["tree_sha256"].clone()
        };
        fixture.install_campaign(campaign, &mutant);
        fixture.reject();
    }
}

#[test]
fn mutation_baseline_requires_a_distinct_clean_pass_receipt() {
    for field in ["same_run", "nonpass", "not_baseline"] {
        let mut fixture = Fixture::new();
        let (mut campaign, mutant, mut baseline) = fixture.campaign();
        match field {
            "same_run" => {
                baseline["identity"]["run_id"] = fixture.root["identity"]["run_id"].clone();
            }
            "nonpass" => {
                baseline["verdict"]["state"] = json!("FAIL");
                baseline["verdict"]["reasons"] = json!(["actual baseline failure"]);
            }
            _ => baseline["diagnostics"]["baseline"] = json!(false),
        }
        campaign["baseline_receipt"] = value(&fixture.memory.typed("ReceiptV1", &baseline));
        fixture.install_campaign(campaign, &mutant);
        fixture.reject();
    }
}

#[test]
fn reviewed_equivalent_mutant_review_binds_the_exact_retained_diff() {
    for correct in [true, false] {
        let mut fixture = Fixture::new();
        let (campaign, mut mutant, _) = fixture.campaign();
        let mut review = fixture.memory.value(&fixture.refs["review"]);
        review["action"] = json!("mutation_disposition");
        review["subject_sha256"] = if correct {
            mutant["diff"]["sha256"].clone()
        } else {
            value(&fixture.refs["spec"].sha256)
        };
        let reference = fixture.memory.typed("ReviewV1", &review);
        mutant["outcome"] = json!("reviewed-equivalent");
        mutant["review_ref"] = present(&value(&reference));
        mutant["reason"] = json!("independent equivalence disposition");
        fixture.install_campaign(campaign, &mutant);
        assert_eq!(fixture.check().is_ok(), correct);
    }
}

#[test]
fn cleanup_cutoff_and_grace_must_match_both_frozen_records() {
    for field in ["deadline_ms", "term_grace_ms"] {
        let mut fixture = Fixture::new();
        let mut cleanup = fixture.memory.value(&fixture.refs["CleanupContractV1"]);
        cleanup[field] = json!("199");
        fixture.rebind_invocation("cleanup_contract", "CleanupContractV1", &cleanup);
        fixture.reject();
    }
}

#[test]
fn agreed_cleanup_deadline_cannot_exceed_wall_limit() {
    let mut fixture = Fixture::new();
    let mut limits = fixture.memory.value(&fixture.refs["LimitsV1"]);
    let mut cleanup = fixture.memory.value(&fixture.refs["CleanupContractV1"]);
    limits["cleanup_deadline_ms"] = json!("1001");
    cleanup["deadline_ms"] = json!("1001");
    fixture.rebind_invocation("limits", "LimitsV1", &limits);
    fixture.rebind_invocation("cleanup_contract", "CleanupContractV1", &cleanup);
    fixture.reject();
}

#[test]
fn monotonic_elapsed_uses_exact_nanoseconds_with_no_wall_limit_round_down() {
    for (end, succeeds) in [("99", false), ("1000000100", true), ("1000000101", false)] {
        let mut fixture = Fixture::new();
        fixture.root["observations"]["end_monotonic_ns"] = json!(end);
        assert_eq!(fixture.check().is_ok(), succeeds);
    }
}

#[test]
fn material_unresolved_runtime_obligations_refuse_pass_but_nonmaterial_rows_remain_visible() {
    for (material, state, succeeds) in [
        (true, "open", false),
        (true, "unknown", false),
        (true, "settled", true),
        (false, "open", true),
    ] {
        let mut fixture = Fixture::new();
        let mut obligation = sample("ObligationV1");
        obligation["material"] = json!(material);
        obligation["state"] = json!(state);
        obligation["evidence"] = json!([fixture.refs["log"]]);
        fixture.root["observations"]["unresolved_obligations"] = value(
            &fixture
                .memory
                .typed("ObligationPageV1", &page(&[obligation])),
        );
        assert_eq!(fixture.check().is_ok(), succeeds);
    }
}

#[test]
fn complete_availability_cannot_contain_a_missing_object_descriptor() {
    let mut fixture = Fixture::new();
    let lost = &fixture.refs["spec"];
    let descriptor = json!({"artifact_id":lost.artifact_id,"expected_sha256":lost.sha256,
        "expected_byte_length":lost.byte_length,"expected_schema_id":lost.schema_id,"reason":"observed retention loss"});
    fixture.root["availability"]["missing_objects"] = value(
        &fixture
            .memory
            .typed("MissingObjectPageV1", &page(&[descriptor])),
    );
    fixture.reject();
}

/// One subject file row: `path` holding the raw bytes `content` (or none), with `kind` and mode.
fn subject_file(content: Option<&Ref>, path: &str, kind: &str, executable: bool) -> Value {
    json!({"path":path,"kind":kind,
        "content":content.map_or_else(|| unavailable("no content"), |content| present(&value(content))),
        "executable":executable,"link_target":unavailable("regular file"),"origin":"authored",
        "exclusion_reason":unavailable("included")})
}

fn candidate(content: &Ref) -> Value {
    subject_file(Some(content), "candidate.rs", "file", false)
}
fn other(content: &Ref) -> Value {
    subject_file(Some(content), "other.rs", "file", false)
}

impl Fixture {
    /// Publish a `SubjectV1` over `rows` and return its reference.
    fn subject(&mut self, rows: &[Value]) -> Ref {
        let files = self.memory.typed("SubjectFilePageV1", &page(rows));
        let subject = json!({"subject_id":"18000000-0000-4000-8000-000000000009","files":files,
            "tree_sha256":files.sha256,"dirty_patch":unavailable("clean")});
        self.memory.typed("SubjectV1", &subject)
    }

    /// Bind `seed`, `result` and `patch` into the root and the preparation alike (so the plan's
    /// binding holds and the patch binding is what decides), with the artifact inventory naming
    /// what a PASS requires: the log, the result's contents and the patch.
    fn bind(&mut self, seed: &Ref, result: &Ref, patch: &Ref, result_contents: &[&Ref]) {
        self.root["subjects"]["seed_subject"] = value(seed);
        self.root["subjects"]["result_subject"] = present(&value(result));
        self.root["subjects"]["seed_to_result_patch"] = present(&value(patch));
        self.prepared.subjects = dto(&self.root["subjects"]);
        self.artifacts = std::iter::once(&self.refs["log"])
            .chain(result_contents.iter().copied())
            .chain(std::iter::once(patch))
            .map(payload)
            .collect();
        self.recount_artifacts();
    }

    /// Bind a seed and a result each of one `candidate.rs` holding the given text, and `patch`.
    fn bind_texts(&mut self, seed: &[u8], result: &[u8], patch: &[u8]) {
        let (seed_bytes, result_bytes) = (self.memory.raw(seed), self.memory.raw(result));
        let patch = self.memory.raw(patch);
        let seed = self.subject(&[candidate(&seed_bytes)]);
        let result = self.subject(&[candidate(&result_bytes)]);
        self.bind(&seed, &result, &patch, &[&result_bytes]);
    }
}

const SEED: &[u8] = b"seed fixture\n";
const RESULT: &[u8] = b"candidate result\n";
const PATCH: &[u8] =
    b"--- a/candidate.rs\n+++ b/candidate.rs\n@@ -1 +1 @@\n-seed fixture\n+candidate result\n";

/// B14-P4 · the fixture's own triple is accepted because its patch IS the derivation (the two
/// texts re-published under fresh identities, so the binding reads contents, never references),
/// and an unchanged result — the development baseline's shape — is accepted only with an empty
/// patch.
#[test]
fn a_patch_bound_to_its_subjects_is_accepted() {
    let mut fixture = Fixture::new();
    fixture.bind_texts(SEED, RESULT, PATCH);
    assert!(fixture.check().is_ok());
    let mut fixture = Fixture::new();
    fixture.bind_texts(SEED, SEED, b"");
    assert!(fixture.check().is_ok());
    // A nonpass receipt naming no result claims no patch, so none is bound; one naming both is
    // bound whatever its verdict. The patch is well formed, but for another pair.
    let foreign =
        b"--- a/candidate.rs\n+++ b/candidate.rs\n@@ -1 +1 @@\n-seed fixture\n+mutated result\n";
    for (result, verdict) in [
        (false, Ok(())),
        (true, Err(Error::Patch(PatchRefusal::Mismatch))),
    ] {
        let mut fixture = Fixture::new();
        fixture.bind_texts(SEED, RESULT, foreign);
        if !result {
            fixture.root["subjects"]["result_subject"] = unavailable("no result");
            fixture.prepared.subjects = dto(&fixture.root["subjects"]);
        }
        fixture.root["verdict"]["state"] = json!("UNMEASURED");
        fixture.root["verdict"]["reasons"] = json!(["candidate not yet verified"]);
        assert_eq!(
            fixture.check().map(|_| ()),
            verdict,
            "result named: {result}"
        );
    }
}

/// B14-P4 · the seed ↔ result ↔ patch triple is one binding: a patch from another pair, a seed
/// swapped under a correct patch, a result published from the seed under a nonempty patch, and a
/// nonempty patch over an unchanged file are each refused as the one mismatch.
#[test]
fn a_patch_not_derived_from_its_subjects_is_refused() {
    let other =
        b"--- a/candidate.rs\n+++ b/candidate.rs\n@@ -1 +1 @@\n-seed fixture\n+mutated result\n";
    for (seed, result, claimed) in [
        (SEED, RESULT, &other[..]),
        (&b"other seed\n"[..], RESULT, PATCH),
        (SEED, SEED, PATCH),
        (RESULT, RESULT, PATCH),
    ] {
        let mut fixture = Fixture::new();
        fixture.bind_texts(seed, result, claimed);
        assert_eq!(
            fixture.check().map(|_| ()),
            Err(Error::Patch(PatchRefusal::Mismatch)),
            "{:?}",
            String::from_utf8_lossy(seed)
        );
    }
    // The result subject itself published as the seed's: no content differs, so only an empty
    // patch could be bound.
    let mut fixture = Fixture::new();
    let seed = fixture.refs["seed"].clone();
    let patch = fixture.memory.raw(PATCH);
    let seed_bytes = fixture.refs["seed_bytes"].clone();
    fixture.bind(&seed, &seed, &patch, &[&seed_bytes]);
    assert_eq!(
        fixture.check().map(|_| ()),
        Err(Error::Patch(PatchRefusal::Mismatch))
    );
}

/// B14-P4 · a patch too short for its change, or over text that is not text, derives nothing:
/// each refused with the derivation's own cause.
#[test]
fn a_patch_the_subjects_cannot_derive_names_the_cause() {
    for (seed, result, claimed, cause) in [
        (
            SEED,
            RESULT,
            &b"--- a/candidate.rs\n+++ b/candidate.rs\n"[..],
            patch::Error::Changes,
        ),
        (SEED, &b"nul\0\n"[..], PATCH, patch::Error::Encoding),
        (&b"\xff\n"[..], RESULT, PATCH, patch::Error::Encoding),
    ] {
        let mut fixture = Fixture::new();
        fixture.bind_texts(seed, result, claimed);
        assert_eq!(
            fixture.check().map(|_| ()),
            Err(Error::Patch(PatchRefusal::Derivation(cause)))
        );
    }
}

/// B14-P4 · every refusal of the inventory by its own site: a result with an extra entry
/// (`Count`), the changed file's mode, kind, origin, link target or exclusion reason differing
/// (`Entry`, one case each), a second file's contents changed
/// (`TwoChanges`), and a changed entry that is not a regular file (`Editable`: an `other` entry,
/// the one non-file kind a valid record may give content — a file must have it, a directory may
/// not, and a symlink must name its target).
#[test]
fn the_subjects_may_differ_in_one_file_s_contents_only() {
    type Rows = fn(&Ref, &Ref) -> (Vec<Value>, Vec<Value>);
    let cases: [(Rows, PatchRefusal); 8] = [
        (
            |seed, result| (vec![candidate(seed)], vec![candidate(result), other(seed)]),
            PatchRefusal::Count,
        ),
        (
            |seed, result| {
                (
                    vec![subject_file(Some(seed), "candidate.rs", "file", false)],
                    vec![subject_file(Some(result), "candidate.rs", "file", true)],
                )
            },
            PatchRefusal::Entry,
        ),
        (
            |seed, result| {
                (
                    vec![subject_file(Some(seed), "candidate.rs", "file", false)],
                    vec![subject_file(Some(result), "candidate.rs", "other", false)],
                )
            },
            PatchRefusal::Entry,
        ),
        (
            |seed, result| {
                (
                    vec![candidate(seed), other(seed)],
                    vec![candidate(result), other(result)],
                )
            },
            PatchRefusal::TwoChanges,
        ),
        (
            |seed, result| {
                (
                    vec![subject_file(Some(seed), "candidate.rs", "other", false)],
                    vec![subject_file(Some(result), "candidate.rs", "other", false)],
                )
            },
            PatchRefusal::Editable,
        ),
        (
            |seed, result| {
                let mut generated = candidate(result);
                generated["origin"] = json!("generated");
                (vec![candidate(seed)], vec![generated])
            },
            PatchRefusal::Entry,
        ),
        (
            |seed, result| {
                let link = |content: &Ref, target: &str| {
                    let mut row = subject_file(Some(content), "candidate.rs", "symlink", false);
                    row["link_target"] = present(&json!(target));
                    row
                };
                (vec![link(seed, "a")], vec![link(result, "b")])
            },
            PatchRefusal::Entry,
        ),
        (
            |seed, result| {
                let excluded = |content: &Ref, reason: &str| {
                    let mut row = candidate(content);
                    row["origin"] = json!("excluded");
                    row["exclusion_reason"] = present(&json!(reason));
                    row
                };
                (
                    vec![excluded(seed, "vendored")],
                    vec![excluded(result, "generated")],
                )
            },
            PatchRefusal::Entry,
        ),
    ];
    for (rows, refusal) in cases {
        let mut fixture = Fixture::new();
        let (seed_bytes, result_bytes) = (fixture.memory.raw(SEED), fixture.memory.raw(RESULT));
        let (seed_rows, result_rows) = rows(&seed_bytes, &result_bytes);
        let seed = fixture.subject(&seed_rows);
        let result = fixture.subject(&result_rows);
        let patch = fixture.memory.raw(PATCH);
        fixture.bind(&seed, &result, &patch, &[&result_bytes]);
        assert_eq!(
            fixture.check().map(|_| ()),
            Err(Error::Patch(refusal)),
            "{refusal:?}"
        );
    }
}
