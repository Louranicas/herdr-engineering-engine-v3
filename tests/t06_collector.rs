// Fixture helpers adapted from the retained independent consistency test source.
// The new builder controls below are author-owned development checks.
//! Independent metadata-consistency oracles; design.md was frozen before bodies.
//! Structural record shapes use the prior schema author's inventory-examples.json.
//! All placeholder refs are replaced by real retained hashes. The memory reader and
//! SHA helper adapt this agent's independent graph fixture, not production logic.
//! Synthetic producer/review assertions establish no custody or authentication.

use habitat_engine::check::consistency::Editable;
use habitat_engine::check::consistency::{CasePlan, Prepared};
use habitat_engine::check::graph::{Error as GraphError, Graph, Objects};
use habitat_engine::check::patch::CandidateBounds;
use habitat_engine::contracts::receipt::RelPath;
use habitat_engine::contracts::receipt::{
    Id, List, Name, ReceiptRecord, Ref, Sha, TypedRef, decode,
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
            editable: Editable {
                path: RelPath::new("candidate.rs").unwrap(),
                bounds: CandidateBounds {
                    bytes: 4096,
                    changed_lines: 200,
                },
            },
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
}

use habitat_engine::check::collector::{
    CaseObservation, Error as BuildError, Observed, Publisher, Sink, SinkError, VerdictBinding,
};
use habitat_engine::contracts::receipt::{
    Address, ArtifactV1, CampaignV1, CasePageV1, CaseV1, CaseV1Outcome, DiagnosticV1,
    LanguageFlagsV1, Maybe, ReceiptV1, Text, VerdictV1, VerdictV1State,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fault {
    None,
    Allocation,
    Registry,
    Publish,
    Id,
    Digest,
    Length,
    Media,
    Schema,
    CorruptRead,
    MissingRead,
    RootPublish,
}
struct MemorySink {
    memory: Memory,
    fault: Fault,
    published: Vec<Ref>,
    allocations: usize,
    collision_at: Option<(usize, Id)>,
}
impl MemorySink {
    fn new(memory: Memory) -> Self {
        Self {
            memory,
            fault: Fault::None,
            published: vec![],
            allocations: 0,
            collision_at: None,
        }
    }
}
impl Objects for MemorySink {
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, GraphError> {
        if self.fault == Fault::MissingRead {
            return Err(GraphError::Missing);
        }
        if self.fault == Fault::CorruptRead {
            return Ok(Box::new(Cursor::new(b"corrupt")));
        }
        self.memory.open(reference)
    }
}
impl Sink for MemorySink {
    fn fresh_id(&mut self) -> Result<Id, SinkError> {
        if self.fault == Fault::Allocation {
            return Err(SinkError::Unavailable);
        }
        self.allocations += 1;
        if let Some((at, id)) = &self.collision_at
            && *at == self.allocations
        {
            return Ok(id.clone());
        }
        Ok(Id::new(format!("09000000-0000-4000-8000-{:012x}", self.allocations)).unwrap())
    }
    fn contains_id(&self, id: &Id) -> Result<bool, SinkError> {
        if self.fault == Fault::Registry {
            return Err(SinkError::Unavailable);
        }
        Ok(self.memory.bytes.contains_key(id.as_str()))
    }
    fn publish(&mut self, reference: &Ref, bytes: &[u8]) -> Result<Ref, SinkError> {
        if self.fault == Fault::Publish
            || self.fault == Fault::RootPublish
                && reference.schema_id.as_str() == ReceiptV1::SCHEMA_ID
        {
            return Err(SinkError::Publication);
        }
        assert!(
            self.memory
                .bytes
                .insert(reference.artifact_id.as_str().to_owned(), bytes.to_vec())
                .is_none()
        );
        self.published.push(reference.clone());
        let mut returned = reference.clone();
        match self.fault {
            Fault::Id => {
                returned.artifact_id = Id::new("19000000-0000-4000-8000-000000000001").unwrap();
            }
            Fault::Digest => {
                returned.sha256 = Sha::new(format!("sha256:{}", "0".repeat(64))).unwrap();
            }
            Fault::Length => returned.byte_length += 1,
            Fault::Media => returned.media_type = Name::new("text/plain").unwrap(),
            Fault::Schema => returned.schema_id = Name::new("hee3.raw/1").unwrap(),
            _ => {}
        }
        Ok(returned)
    }
}
fn flags() -> LanguageFlagsV1 {
    LanguageFlagsV1 {
        language: Name::new("rust").unwrap(),
        argv: List::new(vec![]).unwrap(),
    }
}
fn observed(f: &Fixture) -> Observed {
    let cases = f
        .rows
        .iter()
        .map(|raw| {
            let row: CaseV1 = dto(raw);
            CaseObservation {
                case_id: row.case_id,
                executed: row.executed,
                outcome: row.outcome,
                producer: row.producer_exit_or_signal,
                detector_id: row.detector_id,
                benign_pair_id: row.benign_pair_id,
                raw_evidence_refs: row.raw_evidence_refs,
                reason: row.reason,
            }
        })
        .collect();
    Observed {
        observations: dto(&f.root["observations"]),
        cases,
        diagnostic_baseline: f.root["diagnostics"]["baseline"].as_bool().unwrap(),
        diagnostics: f.diagnostics.iter().map(dto::<DiagnosticV1>).collect(),
        diagnostic_mismatch: Maybe::unavailable(Text::new("no mismatch").unwrap()),
        artifacts: f.artifacts.iter().map(dto::<ArtifactV1>).collect(),
        artifacts_finalized: true,
        campaigns: f.campaigns.iter().map(dto::<CampaignV1>).collect(),
        verdict: {
            let verdict: VerdictV1 = dto(&f.root["verdict"]);
            VerdictBinding {
                oracle_result: verdict.oracle_result,
                intended_detector: verdict.intended_detector,
                benign_pair: verdict.benign_pair,
            }
        },
        availability: dto(&f.root["availability"]),
    }
}
#[test]
fn exact_typed_record_publication_binds_all_metadata_and_readback() {
    let mut sink = MemorySink::new(Memory::default());
    let input = flags();
    let reference = Publisher::new(&mut sink).record(&input).unwrap();
    assert_eq!(
        reference.as_ref().schema_id.as_str(),
        LanguageFlagsV1::SCHEMA_ID
    );
    assert_eq!(
        reference.as_ref().sha256.as_str(),
        digest(&sink.memory.bytes[reference.as_ref().artifact_id.as_str()])
    );
    assert_eq!(
        decode::<LanguageFlagsV1>(&sink.memory.bytes[reference.as_ref().artifact_id.as_str()])
            .unwrap(),
        input
    );
}
#[test]
fn every_substituted_reference_field_is_refused_after_retaining_attempt() {
    for fault in [
        Fault::Id,
        Fault::Digest,
        Fault::Length,
        Fault::Media,
        Fault::Schema,
    ] {
        let mut sink = MemorySink::new(Memory::default());
        sink.fault = fault;
        let mut publisher = Publisher::new(&mut sink);
        assert!(matches!(
            publisher.record(&flags()),
            Err(BuildError::Identity)
        ));
        assert_eq!(publisher.attempted_refs().len(), 1);
        drop(publisher);
        assert_eq!(sink.published.len(), 1);
    }
}
#[test]
fn corrupt_or_missing_readback_cannot_produce_a_typed_reference() {
    for fault in [Fault::CorruptRead, Fault::MissingRead] {
        let mut sink = MemorySink::new(Memory::default());
        sink.fault = fault;
        assert!(Publisher::new(&mut sink).record(&flags()).is_err());
        assert_eq!(sink.published.len(), 1);
    }
}
#[test]
fn allocator_registry_and_publication_errors_propagate() {
    for fault in [Fault::Allocation, Fault::Registry, Fault::Publish] {
        let mut sink = MemorySink::new(Memory::default());
        sink.fault = fault;
        assert!(matches!(
            Publisher::new(&mut sink).record(&flags()),
            Err(BuildError::Sink(_))
        ));
        assert!(sink.published.is_empty());
    }
}
#[test]
fn preexisting_registered_id_is_never_overwritten_or_shadowed() {
    let mut memory = Memory::default();
    let old = memory.raw(b"preexisting");
    let mut sink = MemorySink::new(memory);
    sink.collision_at = Some((1, old.artifact_id.clone()));
    assert!(matches!(
        Publisher::new(&mut sink).record(&flags()),
        Err(BuildError::Identity)
    ));
    assert_eq!(sink.memory.bytes[old.artifact_id.as_str()], b"preexisting");
    assert!(sink.published.is_empty());
}
#[test]
fn reused_session_id_is_refused_even_when_prior_attempt_was_not_published() {
    let mut sink = MemorySink::new(Memory::default());
    sink.fault = Fault::Publish;
    sink.collision_at = Some((2, Id::new("09000000-0000-4000-8000-000000000001").unwrap()));
    let mut publisher = Publisher::new(&mut sink);
    assert!(matches!(
        publisher.record(&flags()),
        Err(BuildError::Sink(_))
    ));
    assert!(matches!(
        publisher.record(&flags()),
        Err(BuildError::Identity)
    ));
    assert_eq!(publisher.attempted_refs().len(), 1);
}
#[test]
fn empty_inventory_is_one_explicit_final_page() {
    let mut sink = MemorySink::new(Memory::default());
    let reference = Publisher::new(&mut sink).case_pages(&[]).unwrap();
    let graph = Graph::resolve(&sink, reference.as_ref()).unwrap();
    assert!(graph.rows(reference.as_ref()).unwrap().is_empty());
    let page: CasePageV1 = decode(graph.get(reference.as_ref()).unwrap().bytes()).unwrap();
    assert_eq!(
        (
            page.page_index,
            page.page_count,
            page.row_count,
            page.total_rows
        ),
        (0, 1, 0, 0)
    );
    assert_eq!(
        page.next.unavailable_reason.unwrap().as_str(),
        "end_of_inventory"
    );
}
#[test]
fn successful_root_uses_exact_prepared_bindings_and_recomputed_counts() {
    let f = Fixture::new();
    let observations = observed(&f);
    let prepared = f.prepared;
    let mut sink = MemorySink::new(f.memory);
    let result = Publisher::new(&mut sink)
        .finalize(&prepared, &pass(), observations)
        .unwrap();
    let root: ReceiptV1 = decode(&result.bytes).unwrap();
    assert_eq!(root.identity, prepared.identity);
    assert_eq!(root.subjects, prepared.subjects);
    assert_eq!(root.invocation, prepared.invocation);
    assert_eq!(
        (
            root.cases.discovered,
            root.cases.selected,
            root.cases.executed,
            root.cases.passed,
            root.cases.primary_credit
        ),
        (1, 1, 1, 1, 1)
    );
    assert!(root.review.value.is_none());
    assert_eq!(result.summary.primary_credit, 1);
    assert_eq!(
        sink.memory.bytes[result.reference.as_ref().artifact_id.as_str()],
        result.bytes
    );
    assert_eq!(root.artifacts.count, 3);
    assert_eq!(root.diagnostics.warning_count, 0);
}
#[test]
fn root_publisher_cannot_bypass_finalization() {
    let f = Fixture::new();
    let mut sink = MemorySink::new(f.memory);
    let root: ReceiptV1 = dto(&f.root);
    assert!(matches!(
        Publisher::new(&mut sink).record(&root),
        Err(BuildError::UnsupportedRoot)
    ));
    assert!(sink.published.is_empty());
}
#[test]
fn duplicate_missing_and_unknown_case_observations_refuse_before_publication() {
    for mode in 0..3 {
        let f = Fixture::new();
        let mut o = observed(&f);
        match mode {
            0 => o.cases.clear(),
            1 => o.cases.push(o.cases[0].clone()),
            _ => o.cases[0].case_id = Name::new("unknown").unwrap(),
        }
        let mut sink = MemorySink::new(f.memory);
        assert!(matches!(
            Publisher::new(&mut sink).finalize(&f.prepared, &pass(), o),
            Err(BuildError::CasePlan)
        ));
        assert!(sink.published.is_empty());
    }
}
#[test]
fn pass_cannot_hide_failed_mandatory_outcome_or_mismatched_producer() {
    for mode in 0..2 {
        let f = Fixture::new();
        let mut o = observed(&f);
        if mode == 0 {
            o.cases[0].outcome = CaseV1Outcome::Failed;
            o.cases[0].reason = Text::new("expected fault").unwrap();
        } else {
            o.cases[0].producer.exit_code = Maybe::present(1);
        }
        let mut sink = MemorySink::new(f.memory);
        assert!(
            Publisher::new(&mut sink)
                .finalize(&f.prepared, &pass(), o)
                .is_err()
        );
        assert!(
            sink.published
                .iter()
                .all(|r| r.schema_id.as_str() != ReceiptV1::SCHEMA_ID)
        );
    }
}
#[test]
fn nonpass_retains_nonempty_plan_with_zero_selected_and_executed() {
    let f = Fixture::new();
    let mut o = observed(&f);
    let mut prepared = f.prepared;
    prepared.cases[0].mandatory = false;
    prepared.cases[0].selected = false;
    o.cases[0].executed = false;
    o.cases[0].outcome = CaseV1Outcome::Unmeasured;
    o.cases[0].reason = Text::new("not dispatched").unwrap();
    let mut sink = MemorySink::new(f.memory);
    let result = Publisher::new(&mut sink)
        .finalize(&prepared, &invalid(), o)
        .unwrap();
    let root: ReceiptV1 = decode(&result.bytes).unwrap();
    assert_eq!(
        (
            root.cases.discovered,
            root.cases.selected,
            root.cases.executed,
            root.cases.unmeasured,
            root.cases.primary_credit
        ),
        (1, 0, 0, 1, 0)
    );
}
#[test]
fn over_256_planned_rows_form_reverse_published_complete_pages() {
    let f = Fixture::new();
    let mut o = observed(&f);
    let mut prepared = f.prepared;
    let original_plan = prepared.cases[0].clone();
    let original_observed = o.cases[0].clone();
    prepared.cases.clear();
    o.cases.clear();
    for index in 0..335 {
        let id = Name::new(format!("synthetic-page-row-{index:03}")).unwrap();
        let mut p = original_plan.clone();
        p.case_id = id.clone();
        p.mandatory = false;
        p.selected = false;
        prepared.cases.push(p);
        let mut c = original_observed.clone();
        c.case_id = id;
        c.executed = false;
        c.outcome = CaseV1Outcome::Unmeasured;
        c.reason = Text::new("page control only").unwrap();
        o.cases.push(c);
    }
    let mut sink = MemorySink::new(f.memory);
    let result = Publisher::new(&mut sink)
        .finalize(&prepared, &invalid(), o)
        .unwrap();
    let root: ReceiptV1 = decode(&result.bytes).unwrap();
    let graph = Graph::resolve(&sink, result.reference.as_ref()).unwrap();
    let rows = graph.rows(root.cases.inventory.as_ref()).unwrap();
    assert_eq!(rows.len(), 335);
    assert_eq!(root.cases.primary_credit, 0);
    let first: CasePageV1 =
        decode(graph.get(root.cases.inventory.as_ref()).unwrap().bytes()).unwrap();
    assert_eq!(
        (first.page_count, first.row_count, first.total_rows),
        (2, 256, 335)
    );
    let last = first.next.value.unwrap();
    let second: CasePageV1 = decode(graph.get(last.as_ref()).unwrap().bytes()).unwrap();
    assert_eq!((second.page_index, second.row_count), (1, 79));
    assert_eq!(sink.published[0].artifact_id, last.as_ref().artifact_id);
}
#[test]
fn excluded_row_counts_only_as_excluded() {
    let f = Fixture::new();
    let mut o = observed(&f);
    let mut p = f.prepared;
    p.cases[0].mandatory = false;
    p.cases[0].selected = false;
    p.cases[0].excluded = true;
    o.cases[0].executed = false;
    o.cases[0].outcome = CaseV1Outcome::Unmeasured;
    o.cases[0].reason = Text::new("excluded by frozen plan").unwrap();
    let mut sink = MemorySink::new(f.memory);
    let result = Publisher::new(&mut sink)
        .finalize(&p, &invalid(), o)
        .unwrap();
    let root: ReceiptV1 = decode(&result.bytes).unwrap();
    assert_eq!((root.cases.excluded, root.cases.unmeasured), (1, 0));
}
#[test]
fn unreviewed_case_cannot_receive_primary_credit_or_mandatory_pass() {
    let f = Fixture::new();
    let mut p = f.prepared.clone();
    p.cases[0].reviewed_design = None;
    let o = observed(&f);
    let mut sink = MemorySink::new(f.memory);
    assert!(Publisher::new(&mut sink).finalize(&p, &pass(), o).is_err());
    assert!(
        sink.published
            .iter()
            .all(|r| r.schema_id.as_str() != ReceiptV1::SCHEMA_ID)
    );
}
#[test]
fn diagnostics_are_recomputed_and_nonpass_preserves_fault_counts() {
    let f = Fixture::new();
    let mut o = observed(&f);
    o.diagnostic_baseline = false;
    o.diagnostics[0].baseline = false;
    o.diagnostics[0].warning_count = 2;
    o.diagnostics[0].error_count = 3;
    o.diagnostics[0].stdout_truncated = true;
    let mut sink = MemorySink::new(f.memory);
    let result = Publisher::new(&mut sink)
        .finalize(&f.prepared, &fail(), o)
        .unwrap();
    let root: ReceiptV1 = decode(&result.bytes).unwrap();
    assert_eq!(
        (root.diagnostics.warning_count, root.diagnostics.error_count),
        (2, 3)
    );
    assert!(root.diagnostics.stdout_truncated);
}
#[test]
fn diagnostic_baseline_disagreement_is_not_silently_rewritten() {
    let f = Fixture::new();
    let mut o = observed(&f);
    o.diagnostics[0].baseline = !o.diagnostic_baseline;
    let mut sink = MemorySink::new(f.memory);
    assert!(matches!(
        Publisher::new(&mut sink).finalize(&f.prepared, &pass(), o),
        Err(BuildError::Consistency(_))
    ));
}
#[test]
fn required_truncated_artifact_blocks_pass_before_root_publication() {
    let f = Fixture::new();
    let mut o = observed(&f);
    o.artifacts[0].truncated = true;
    let mut sink = MemorySink::new(f.memory);
    assert!(
        Publisher::new(&mut sink)
            .finalize(&f.prepared, &pass(), o)
            .is_err()
    );
    assert!(
        sink.published
            .iter()
            .all(|r| r.schema_id.as_str() != ReceiptV1::SCHEMA_ID)
    );
}
#[test]
fn root_id_collision_is_refused_without_shadowing_existing_input() {
    let f = Fixture::new();
    let o = observed(&f);
    let collision = f.refs["log"].artifact_id.clone();
    let before = f.memory.bytes[collision.as_str()].clone();
    let mut sink = MemorySink::new(f.memory);
    sink.collision_at = Some((5, collision.clone()));
    assert!(matches!(
        Publisher::new(&mut sink).finalize(&f.prepared, &pass(), o),
        Err(BuildError::Identity)
    ));
    assert_eq!(sink.memory.bytes[collision.as_str()], before);
    assert_eq!(sink.published.len(), 4);
}
#[test]
fn final_root_sink_failure_returns_no_finalized_root_and_retains_staging() {
    let f = Fixture::new();
    let o = observed(&f);
    let mut sink = MemorySink::new(f.memory);
    sink.fault = Fault::RootPublish;
    let mut publisher = Publisher::new(&mut sink);
    assert!(matches!(
        publisher.finalize(&f.prepared, &pass(), o),
        Err(BuildError::Sink(SinkError::Publication))
    ));
    assert_eq!(publisher.attempted_refs().len(), 5);
    drop(publisher);
    assert_eq!(sink.published.len(), 4);
    assert!(
        sink.published
            .iter()
            .all(|r| r.schema_id.as_str() != ReceiptV1::SCHEMA_ID)
    );
}

use habitat_engine::contracts::receipt::{MutantV1, MutantV1Outcome};
fn with_campaign(mode: u8) -> (MemorySink, Prepared, Observed) {
    let f = Fixture::new();
    let mut o = observed(&f);
    let baseline_o = observed(&f);
    let mut p = f.prepared.clone();
    let refs = f.refs.clone();
    let mut sink = MemorySink::new(f.memory);
    let baseline = Publisher::new(&mut sink)
        .finalize(&p, &pass(), baseline_o)
        .unwrap();
    p.identity.run_id = Id::new("29000000-0000-4000-8000-000000000001").unwrap();
    let diff = sink.memory.raw(b"retained synthetic diff");
    let mut review: Value =
        serde_json::from_slice(&sink.memory.bytes[refs["review"].artifact_id.as_str()]).unwrap();
    review["action"] = json!("mutation_disposition");
    review["subject_sha256"] = value(&diff.sha256);
    let disposition = sink.memory.typed("ReviewV1", &review);
    let mut mutants = Vec::new();
    for (index, outcome) in [
        MutantV1Outcome::Caught,
        MutantV1Outcome::Survived,
        MutantV1Outcome::Timeout,
        MutantV1Outcome::Unviable,
        MutantV1Outcome::ReviewedEquivalent,
        MutantV1Outcome::Excluded,
        MutantV1Outcome::Unmeasured,
    ]
    .into_iter()
    .enumerate()
    {
        let mut raw = sample("MutantV1");
        raw["mutant_id"] = json!(format!("synthetic-mutant-{index}"));
        raw["baseline_subject_sha256"] = value(&refs["result"].sha256);
        raw["diff"] = value(&diff);
        raw["raw_evidence_refs"] = json!([refs["log"]]);
        raw["expected_detector"] = json!(DETECTOR);
        raw["observed_detector"] = present(&json!(DETECTOR));
        raw["outcome"] = value(&outcome);
        raw["reason"] = json!("synthetic disposition control");
        raw["executed"] = json!(matches!(
            outcome,
            MutantV1Outcome::Caught | MutantV1Outcome::Survived | MutantV1Outcome::Timeout
        ));
        if outcome == MutantV1Outcome::ReviewedEquivalent {
            raw["review_ref"] = present(&value(&disposition));
        }
        mutants.push(dto::<MutantV1>(&raw));
    }
    let first = Publisher::new(&mut sink).mutant_pages(&mutants).unwrap();
    let mut raw = sample("CampaignV1");
    raw["tool"] = value(&refs["spec"]);
    raw["config"] = value(&refs["profile"]);
    raw["baseline_receipt"] = value(&baseline.reference);
    raw["mutants"] = value(&first);
    raw["planned_mutants"] = json!(7);
    if mode == 1 {
        raw["planned_mutants"] = json!(6);
    } else if mode == 2 {
        raw["campaign_id"] = json!("wrong-campaign");
    }
    o.campaigns = vec![dto(&raw)];
    (sink, p, o)
}
#[test]
fn mutation_dispositions_recompute_separate_counts_without_case_credit() {
    let (mut sink, p, o) = with_campaign(0);
    let result = Publisher::new(&mut sink).finalize(&p, &pass(), o).unwrap();
    let root: ReceiptV1 = decode(&result.bytes).unwrap();
    let m = root.mutation;
    assert_eq!((m.campaign_count, m.distinct_mutants), (1, 7));
    assert_eq!(
        [
            m.caught,
            m.survived,
            m.timed_out,
            m.unviable,
            m.equivalent,
            m.excluded,
            m.unmeasured
        ],
        [1; 7]
    );
    assert_eq!(root.cases.primary_credit, 1);
}
#[test]
fn mutation_planned_count_and_campaign_identity_mismatches_refuse_final_root() {
    for mode in [1, 2] {
        let (mut sink, p, o) = with_campaign(mode);
        let before = sink
            .published
            .iter()
            .filter(|r| r.schema_id.as_str() == ReceiptV1::SCHEMA_ID)
            .count();
        assert!(matches!(
            Publisher::new(&mut sink).finalize(&p, &pass(), o),
            Err(BuildError::Consistency(_))
        ));
        assert_eq!(
            sink.published
                .iter()
                .filter(|r| r.schema_id.as_str() == ReceiptV1::SCHEMA_ID)
                .count(),
            before
        );
    }
}
#[test]
fn diagnostic_counter_overflow_is_refused_without_wrapping() {
    let f = Fixture::new();
    let mut o = observed(&f);
    o.diagnostics[0].warning_count = u32::MAX;
    let mut second = o.diagnostics[0].clone();
    second.tool_id = Name::new("second-tool").unwrap();
    second.warning_count = 1;
    o.diagnostics.push(second);
    let mut sink = MemorySink::new(f.memory);
    assert!(matches!(
        Publisher::new(&mut sink).finalize(&f.prepared, &fail(), o),
        Err(BuildError::Bound)
    ));
}
#[test]
fn missing_prepared_subject_prevents_root_publication_after_staging() {
    let f = Fixture::new();
    let o = observed(&f);
    let mut memory = f.memory;
    memory.bytes.remove(f.refs["seed"].artifact_id.as_str());
    let mut sink = MemorySink::new(memory);
    assert!(matches!(
        Publisher::new(&mut sink).finalize(&f.prepared, &pass(), o),
        Err(BuildError::Graph(GraphError::Missing))
    ));
    assert!(
        sink.published
            .iter()
            .all(|r| r.schema_id.as_str() != ReceiptV1::SCHEMA_ID)
    );
}
#[test]
fn attempted_object_budget_has_an_exact_benign_boundary() {
    let mut sink = MemorySink::new(Memory::default());
    let mut publisher = Publisher::new(&mut sink);
    for _ in 0..4096 {
        publisher.record(&flags()).unwrap();
    }
    assert_eq!(publisher.attempted_refs().len(), 4096);
    assert!(matches!(publisher.record(&flags()), Err(BuildError::Bound)));
    assert_eq!(publisher.attempted_refs().len(), 4096);
}

use habitat_engine::check::decision::{
    self, CaseObservation as DecidedCase, CasePlan as DecidedPlan, CheckerFact, CleanupFacts,
    Decision, Design, DiagnosticPolicy, DiagnosticState, Diagnostics, EvidenceState, Identity,
    IdentityFact, IdentityState, IncompleteCause, Input, LogState, OracleFact, ProcessFact,
    ProcessState, Selection, Settlement, Streams, Termination, Timing,
};
use habitat_engine::contracts::receipt::ExpectedProducerV1;

const DECIDED_IDENTITIES: [Identity; 11] = [
    Identity::Seed,
    Identity::Result,
    Identity::Fixtures,
    Identity::Oracle,
    Identity::Harness,
    Identity::Collector,
    Identity::Launcher,
    Identity::Locks,
    Identity::Toolchain,
    Identity::Profile,
    Identity::Standards,
];
/// A real `decide` result over one reviewed required case. `seed` is the Seed
/// identity's freshness; `oracle` the independent oracle fact.
fn decided(
    seed: IdentityState,
    oracle: OracleFact,
) -> Result<Decision, Box<dyn std::error::Error>> {
    let expected: ExpectedProducerV1 = serde_json::from_value(
        json!({"status":"exited","exit_code":{"value":0,"unavailable_reason":null},"signal":{"value":null,"unavailable_reason":"ordinary exit"}}),
    )?;
    let identities: Vec<IdentityFact> = DECIDED_IDENTITIES
        .map(|subject| IdentityFact {
            subject,
            state: if subject == Identity::Seed {
                seed
            } else {
                IdentityState::Matched
            },
        })
        .to_vec();
    let plans = [DecidedPlan {
        case_id: Name::new(CASE)?,
        selection: Selection::Required,
        expected_producer: expected.clone(),
        design: Design::Reviewed,
    }];
    let cases = [DecidedCase {
        case_id: Name::new(CASE)?,
        executed: true,
        outcome: CaseV1Outcome::Passed,
        incomplete_cause: IncompleteCause::Unexplained,
        producer: ProcessState::Exited(0),
    }];
    let producer = ProcessFact {
        expected,
        actual: ProcessState::Exited(0),
        termination: Termination::Ordinary,
    };
    Ok(decision::decide(&Input {
        identities: &identities,
        plans: &plans,
        cases: &cases,
        producer: &producer,
        checker: &CheckerFact::InProcessComplete,
        oracle,
        oracle_unavailable_cause: IncompleteCause::Unexplained,
        logs: Streams {
            stdout: LogState::Complete,
            stderr: LogState::Complete,
        },
        diagnostics: Diagnostics {
            policy: DiagnosticPolicy::CleanBaseline,
            state: DiagnosticState::Complete {
                warnings: 0,
                errors: 0,
            },
        },
        cleanup: CleanupFacts {
            descendants: Settlement::Settled,
            resources: Settlement::Settled,
            obligations: Settlement::Settled,
        },
        evidence: EvidenceState::FinalizedComplete,
        timing: Timing {
            work_deadline_ms: 100,
            cleanup_deadline_ms: 200,
            observed_ms: 120,
            decisive_ms: Some(99),
            timeout_intent_ms: None,
            cancellation_intent_ms: None,
        },
    }))
}
// Collector fixtures below follow this file's unwrap convention; the decision
// itself is always a real `decide` result, never a hand-set state.
fn pass() -> Decision {
    decided(IdentityState::Matched, OracleFact::Satisfied).unwrap()
}
fn invalid() -> Decision {
    decided(IdentityState::Changed, OracleFact::Satisfied).unwrap()
}
fn fail() -> Decision {
    decided(IdentityState::Matched, OracleFact::Mismatch).unwrap()
}
fn published(decision: &Decision) -> Result<ReceiptV1, Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let o = observed(&f);
    let mut sink = MemorySink::new(f.memory);
    let result = Publisher::new(&mut sink)
        .finalize(&f.prepared, decision, o)
        .map_err(|e| format!("{e:?}"))?;
    Ok(decode(&result.bytes)?)
}
#[test]
fn a_caller_pass_claim_cannot_outlive_a_changed_identity() -> Result<(), Box<dyn std::error::Error>>
{
    let decision = decided(IdentityState::Changed, OracleFact::Satisfied)?;
    assert_eq!(
        decision.state(),
        VerdictV1State::Invalid,
        "fixture premise: a changed seed identity is Invalid"
    );
    let root = published(&decision)?;
    assert_eq!(
        root.verdict.state,
        decision.state(),
        "the published verdict must be decide's, not the caller's"
    );
    let reasons: Vec<&str> = root
        .verdict
        .reasons
        .as_slice()
        .iter()
        .map(Text::as_str)
        .collect();
    assert_eq!(reasons, ["ChangedIdentity"]);
    Ok(())
}
#[test]
fn every_published_verdict_state_and_reason_list_is_the_decisions()
-> Result<(), Box<dyn std::error::Error>> {
    for (decision, state, reasons) in [
        (pass(), VerdictV1State::PassCandidate, vec![]),
        (fail(), VerdictV1State::Fail, vec!["OracleMismatch"]),
        (
            decided(IdentityState::Unavailable, OracleFact::Mismatch)?,
            VerdictV1State::Invalid,
            vec!["UnavailableIdentity", "OracleMismatch"],
        ),
    ] {
        assert_eq!(decision.state(), state, "fixture premise");
        let root = published(&decision)?;
        assert_eq!(root.verdict.state, state);
        let published: Vec<&str> = root
            .verdict
            .reasons
            .as_slice()
            .iter()
            .map(Text::as_str)
            .collect();
        assert_eq!(published, reasons);
    }
    Ok(())
}

use habitat_engine::check::consistency::{
    Error as PlanError, U64_CASE_ID, U64_CRITERION_ID, U64Attempt, prepare_u64,
};
use habitat_engine::check::u64_oracle;
use habitat_engine::contracts::receipt::{Generation, IdentityV1, InvocationV1};

/// The workload manifest was frozen with the task, independently of the plan code.
const U64_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/evaluation/tasks/WL-U64-PARSE-001/v1/manifest.json"
));
fn u64_attempt(
    f: &Fixture,
    generation: &str,
    ids: [&str; 3],
) -> Result<U64Attempt, Box<dyn std::error::Error>> {
    let p = &f.prepared;
    let review = p.cases[0]
        .reviewed_design
        .clone()
        .ok_or("fixture premise: reviewed design")?;
    Ok(U64Attempt {
        schema_sha256: p.schema_sha256.clone(),
        run_id: Id::new(ids[0])?,
        task_id: Id::new(ids[1])?,
        attempt_id: Id::new(ids[2])?,
        generation: Generation::new(generation)?,
        profile_id: Name::new(format!("profile-{generation}"))?,
        parent_run: Maybe::unavailable(Text::new(format!("parent-{generation}"))?),
        subjects: {
            // A fixtures digest no other subject shares, and a different one per
            // attempt, so the case can only get it from the fixtures subject.
            let mut subjects = p.subjects.clone();
            let mut fixtures = value(&subjects.fixtures);
            fixtures["sha256"] = json!(format!("sha256:{}", generation.repeat(64)));
            subjects.fixtures = serde_json::from_value(fixtures)?;
            subjects
        },
        argv: List::new(vec![Text::new(format!("argv-{generation}"))?])?,
        environment: p.invocation.environment.clone(),
        grants: p.invocation.grants.clone(),
        limits: p.invocation.limits.clone(),
        allowed_effects: p.invocation.allowed_effects.clone(),
        cleanup_contract: p.invocation.cleanup_contract.clone(),
        expectation: p.invocation.expected.clone(),
        case_design_review: review,
    })
}
const IDS_A: [&str; 3] = [
    "31000000-0000-4000-8000-000000000001",
    "31000000-0000-4000-8000-000000000002",
    "31000000-0000-4000-8000-000000000003",
];
const IDS_B: [&str; 3] = [
    "32000000-0000-4000-8000-00000000000a",
    "32000000-0000-4000-8000-00000000000b",
    "32000000-0000-4000-8000-00000000000c",
];
#[test]
fn u64_plan_binds_the_workload_manifest_and_every_typed_input()
-> Result<(), Box<dyn std::error::Error>> {
    let manifest: Value = serde_json::from_str(U64_MANIFEST)?;
    let workload = manifest["workload_id"]
        .as_str()
        .ok_or("manifest workload")?;
    let oracle = manifest["oracle_id"].as_str().ok_or("manifest oracle")?;
    assert_eq!(u64_oracle::ORACLE_ID, oracle);
    assert_eq!(U64_CASE_ID, workload.replace('/', "-"));
    let f = Fixture::new();
    // Two attempts differing in every caller-supplied scalar: no field can be
    // frozen to one fixture's value and still pass both.
    for (generation, ids) in [("1", IDS_A), ("2", IDS_B)] {
        let input = u64_attempt(&f, generation, ids)?;
        let plan = prepare_u64(input.clone()).map_err(|e| format!("{e:?}"))?;
        let criteria = List::new(vec![Name::new(U64_CRITERION_ID)?])?;
        assert_eq!(
            plan.identity,
            IdentityV1 {
                run_id: input.run_id.clone(),
                task_id: input.task_id.clone(),
                attempt_id: input.attempt_id.clone(),
                generation: input.generation.clone(),
                module_id: Name::new("check")?,
                criterion_ids: criteria.clone(),
                profile_id: input.profile_id.clone(),
                parent_run: input.parent_run.clone(),
            }
        );
        assert_eq!(plan.schema_sha256, input.schema_sha256);
        assert_eq!(plan.subjects, input.subjects);
        assert_eq!(
            plan.invocation,
            InvocationV1 {
                argv: input.argv.clone(),
                cwd_logical: Name::new("work")?,
                environment: input.environment.clone(),
                grants: input.grants.clone(),
                expected: input.expectation.clone(),
                oracle_id: Name::new(oracle)?,
                limits: input.limits.clone(),
                allowed_effects: input.allowed_effects.clone(),
                cleanup_contract: input.cleanup_contract.clone(),
            }
        );
        let [case] = plan.cases.as_slice() else {
            return Err("exactly one frozen case".into());
        };
        assert_eq!(
            (
                case.case_id.as_str(),
                case.primary_module_id.as_str(),
                case.oracle_id.as_str(),
                &case.criterion_ids,
                &case.fixture_sha256,
                &case.expected,
                (case.mandatory, case.selected, case.excluded),
                case.reviewed_design.as_ref(),
            ),
            (
                workload.replace('/', "-").as_str(),
                "check",
                oracle,
                &criteria,
                &input.subjects.fixtures.as_ref().sha256,
                &input.expectation,
                (true, true, false),
                Some(&input.case_design_review),
            )
        );
    }
    Ok(())
}
#[test]
fn u64_plan_refuses_a_shared_identity_or_an_empty_argv() -> Result<(), Box<dyn std::error::Error>> {
    let f = Fixture::new();
    let [run, task, attempt] = IDS_A;
    for ids in [[run, run, attempt], [run, task, run], [run, task, task]] {
        assert!(matches!(
            prepare_u64(u64_attempt(&f, "1", ids)?),
            Err(PlanError::Binding)
        ));
    }
    let mut empty = u64_attempt(&f, "1", IDS_A)?;
    empty.argv = List::new(vec![])?;
    assert!(matches!(prepare_u64(empty), Err(PlanError::Binding)));
    assert!(prepare_u64(u64_attempt(&f, "1", IDS_A)?).is_ok());
    Ok(())
}
