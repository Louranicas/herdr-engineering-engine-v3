//! Independent complete-closure tests. See ../design.md for frozen source authority,
//! exact limits, case inventory and the cryptographic back-edge limitation.
//! No production graph implementation was read. Objects are retained memory bytes;
//! references use an independent SHA-256 helper, never a permissive resolver shim.

use habitat_engine::check::graph::{Error, Graph, Objects};
use habitat_engine::contracts::receipt::{Id, Name, Ref, Sha};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{self, Cursor, Read};

const RAW: &str = "hee3.raw/1";
const FLAGS: &str = "hee3.receipt/1:LanguageFlagsV1";
const FLAGS_PAGE: &str = "hee3.receipt/1:LanguageFlagsPageV1";
const FILE_PAGE: &str = "hee3.receipt/1:SubjectFilePageV1";
const ARTIFACT: &str = "hee3.receipt/1:ArtifactV1";
const ARTIFACT_PAGE: &str = "hee3.receipt/1:ArtifactPageV1";
const SUBJECT: &str = "hee3.receipt/1:SubjectV1";

#[derive(Default)]
struct Memory {
    // Separate keys deliberately permit retaining conflicting declarations for
    // one artifact ID; Graph, not this fixture, must reject the identity conflict.
    bytes: BTreeMap<(String, String), Vec<u8>>,
    next: u64,
}

fn digest(bytes: &[u8]) -> String {
    let mut result = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}

fn key(reference: &Ref) -> (String, String) {
    (
        reference.artifact_id.as_str().to_owned(),
        reference.sha256.as_str().to_owned(),
    )
}

impl Memory {
    fn insert(&mut self, schema: &str, bytes: Vec<u8>) -> Ref {
        self.next += 1;
        let artifact_id = Id::new(format!("07000000-0000-4000-8000-{:012x}", self.next)).unwrap();
        self.insert_as(artifact_id, schema, bytes)
    }

    fn insert_as(&mut self, artifact_id: Id, schema: &str, bytes: Vec<u8>) -> Ref {
        let reference = Ref {
            artifact_id,
            sha256: Sha::new(digest(&bytes)).unwrap(),
            byte_length: u32::try_from(bytes.len()).unwrap(),
            media_type: Name::new(if schema == RAW {
                "application/octet-stream"
            } else {
                "application/json"
            })
            .unwrap(),
            schema_id: Name::new(schema).unwrap(),
        };
        assert!(self.bytes.insert(key(&reference), bytes).is_none());
        reference
    }

    fn raw(&mut self, bytes: &[u8]) -> Ref {
        self.insert(RAW, bytes.to_vec())
    }

    fn typed(&mut self, schema: &str, value: &Value) -> Ref {
        self.insert(schema, serde_json::to_vec(value).unwrap())
    }
}

impl Objects for Memory {
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, Error> {
        let bytes = self.bytes.get(&key(reference)).ok_or(Error::Missing)?;
        Ok(Box::new(Cursor::new(bytes.as_slice())))
    }
}

fn unavailable(reason: &str) -> Value {
    json!({"value":null,"unavailable_reason":reason})
}

fn available(reference: &Ref) -> Value {
    json!({"value":reference,"unavailable_reason":null})
}

fn flags(language: &str) -> Value {
    json!({"language":language,"argv":[]})
}

fn page(index: usize, count: usize, total: usize, rows: Vec<Value>, next: Option<&Ref>) -> Value {
    json!({"page_index":index,"page_count":count,"row_count":rows.len(),"total_rows":total,
        "rows":Value::Array(rows),"next":next.map_or_else(||unavailable("end_of_inventory"),available)})
}

fn file(path: &str, content: &Ref) -> Value {
    json!({"path":path,"kind":"file","content":available(content),"executable":false,
        "link_target":unavailable("not_a_link"),"origin":"authored", "exclusion_reason":unavailable("included")})
}

fn artifact(object: &Ref) -> Value {
    json!({"object":object,"role":"fixture","required":true,"truncated":false,"availability":"available","reason":""})
}

fn subject(memory: &mut Memory, first_page: &Ref) -> Ref {
    memory.typed(
        SUBJECT,
        &json!({"subject_id":"07000000-0000-4000-8000-ffffffffffff",
        "files":first_page,"tree_sha256":first_page.sha256,"dirty_patch":unavailable("clean")}),
    )
}

fn file_chain(memory: &mut Memory, rows: Vec<Value>) -> Ref {
    let total = rows.len();
    let mut next = None;
    for (index, row) in rows.into_iter().enumerate().rev() {
        next = Some(memory.typed(
            FILE_PAGE,
            &page(index, total, total, vec![row], next.as_ref()),
        ));
    }
    next.unwrap()
}

fn two_pages(memory: &mut Memory) -> (Ref, Ref) {
    let last = memory.typed(FLAGS_PAGE, &page(1, 2, 2, vec![flags("rust")], None));
    let first = memory.typed(
        FLAGS_PAGE,
        &page(0, 2, 2, vec![flags("julia")], Some(&last)),
    );
    (first, last)
}

#[test]
fn raw_payload_is_exact_opaque_bytes_including_invalid_utf8() {
    let mut memory = Memory::default();
    let bytes = [0, 255, 254, b'\n', b'{'];
    let root = memory.raw(&bytes);
    let graph = Graph::resolve(&memory, &root).unwrap();
    assert_eq!(graph.object_count(), 1);
    assert_eq!(graph.total_bytes(), 5);
    assert_eq!(graph.get(&root).unwrap().bytes(), bytes);
    assert_eq!(
        serde_json::to_value(graph.get(&root).unwrap().reference()).unwrap(),
        serde_json::to_value(&root).unwrap()
    );
}

#[test]
fn typed_record_retains_original_key_order_and_bytes() {
    let mut memory = Memory::default();
    let bytes = br#"{"argv":[],"language":"rust"}"#;
    let root = memory.insert(FLAGS, bytes.to_vec());
    let graph = Graph::resolve(&memory, &root).unwrap();
    assert_eq!(graph.get(&root).unwrap().bytes(), bytes);
    assert_eq!(graph.total_bytes(), u64::try_from(bytes.len()).unwrap());
    assert_eq!(
        root.sha256.as_str(),
        "sha256:e54e68d66775ea5e90b8c1009407882742c245f0ec366007a1bb24dfaae5a04a"
    );
}

#[test]
fn missing_root_cannot_resolve() {
    let mut memory = Memory::default();
    let root = memory.raw(b"retained then lost");
    memory.bytes.remove(&key(&root));
    assert!(Graph::resolve(&memory, &root).is_err());
}

#[test]
fn missing_descendant_refuses_an_otherwise_valid_root() {
    let mut memory = Memory::default();
    let raw = memory.raw(b"missing descendant");
    let root = memory.typed(ARTIFACT, &artifact(&raw));
    memory.bytes.remove(&key(&raw));
    assert!(Graph::resolve(&memory, &root).is_err());
}

struct BrokenReader;
impl Read for BrokenReader {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("independent retained-object read fault"))
    }
}
struct BrokenObjects;
impl Objects for BrokenObjects {
    fn open(&self, _reference: &Ref) -> Result<Box<dyn Read + '_>, Error> {
        Ok(Box::new(BrokenReader))
    }
}

#[test]
fn reader_failure_is_not_treated_as_an_empty_successful_object() {
    let mut memory = Memory::default();
    let root = memory.raw(b"");
    assert!(Graph::resolve(&BrokenObjects, &root).is_err());
    assert_eq!(Graph::resolve(&memory, &root).unwrap().total_bytes(), 0);
}

#[test]
fn same_length_content_substitution_is_rejected_by_actual_hash() {
    let mut memory = Memory::default();
    let root = memory.raw(b"correct");
    memory.bytes.insert(key(&root), b"changed".to_vec());
    assert!(Graph::resolve(&memory, &root).is_err());
}

#[test]
fn both_shorter_and_longer_streams_refuse_declared_byte_length() {
    for actual in [b"ab".as_slice(), b"abcd".as_slice()] {
        let mut memory = Memory::default();
        let mut root = memory.raw(actual);
        // The digest matches the actual bytes, isolating declared length alone.
        root.byte_length = 3;
        assert!(Graph::resolve(&memory, &root).is_err());
    }
}

#[test]
fn unknown_schema_id_cannot_fall_back_to_opaque_raw() {
    let mut memory = Memory::default();
    let root = memory.insert("hee3.receipt/1:FutureRecordV2", b"{}".to_vec());
    assert!(Graph::resolve(&memory, &root).is_err());
    let helper = memory.insert("hee3.receipt/1:MaybeText", b"{}".to_vec());
    assert!(Graph::resolve(&memory, &helper).is_err());
}

#[test]
fn typed_decoder_rejects_malformed_duplicate_and_unknown_record_fields() {
    for bytes in [
        br#"{"language":"rust","argv":[]"#.as_slice(),
        br#"{"language":"rust","language":"rust","argv":[]}"#.as_slice(),
        br#"{"language":"rust","argv":[],"extra":true}"#.as_slice(),
    ] {
        let mut memory = Memory::default();
        let root = memory.insert(FLAGS, bytes.to_vec());
        assert!(Graph::resolve(&memory, &root).is_err());
    }
}

#[test]
fn lookup_matches_all_reference_metadata_not_only_artifact_id() {
    let mut memory = Memory::default();
    let root = memory.raw(b"metadata bound");
    let graph = Graph::resolve(&memory, &root).unwrap();
    let mut mutations = Vec::new();
    let mut changed = root.clone();
    changed.artifact_id = Id::new("07000000-0000-4000-8000-ffffffffffff").unwrap();
    mutations.push(changed);
    let mut changed = root.clone();
    changed.sha256 = Sha::new(digest(b"other bytes")).unwrap();
    mutations.push(changed);
    let mut changed = root.clone();
    changed.byte_length += 1;
    mutations.push(changed);
    let mut changed = root.clone();
    changed.media_type = Name::new("text/plain").unwrap();
    mutations.push(changed);
    let mut changed = root.clone();
    changed.schema_id = Name::new(FLAGS).unwrap();
    mutations.push(changed);
    for changed in mutations {
        assert!(graph.get(&changed).is_err());
    }
    assert_eq!(graph.get(&root).unwrap().bytes(), b"metadata bound");
}

#[test]
fn identical_shared_reference_counts_once_across_distinct_subject_paths() {
    let mut memory = Memory::default();
    let raw = memory.raw(b"shared source bytes");
    let first = memory.typed(
        FILE_PAGE,
        &page(0, 1, 2, vec![file("a.rs", &raw), file("b.rs", &raw)], None),
    );
    let root = subject(&mut memory, &first);
    let graph = Graph::resolve(&memory, &root).unwrap();
    assert_eq!(graph.object_count(), 3);
    let expected_bytes =
        u64::from(root.byte_length) + u64::from(first.byte_length) + u64::from(raw.byte_length);
    assert_eq!(graph.total_bytes(), expected_bytes);
    assert_eq!(graph.rows(&first).unwrap().len(), 2);
}

#[test]
fn conflicting_metadata_for_shared_id_refuses_despite_available_bytes() {
    let mut memory = Memory::default();
    let raw = memory.raw(b"same retained bytes");
    let mut changed = raw.clone();
    changed.media_type = Name::new("text/plain").unwrap();
    let first = memory.typed(
        FILE_PAGE,
        &page(0, 1, 2, vec![file("a", &raw), file("b", &changed)], None),
    );
    let root = subject(&mut memory, &first);
    assert!(Graph::resolve(&memory, &root).is_err());
}

#[test]
fn reference_shaped_json_inside_raw_bytes_does_not_trigger_traversal() {
    let mut memory = Memory::default();
    let absent = memory.raw(b"not actually a dependency of the raw document");
    let root = memory.raw(&serde_json::to_vec(&absent).unwrap());
    memory.bytes.remove(&key(&absent));
    let graph = Graph::resolve(&memory, &root).unwrap();
    assert_eq!(graph.object_count(), 1);
    assert!(graph.get(&absent).is_err());
}

#[test]
fn missing_object_descriptor_does_not_require_its_expected_bytes_to_exist() {
    let mut memory = Memory::default();
    let lost = memory.raw(b"historical missing evidence");
    let root = memory.typed("hee3.receipt/1:MissingObjectV1", &json!({
        "artifact_id":lost.artifact_id,"expected_sha256":lost.sha256,
        "expected_byte_length":lost.byte_length,"expected_schema_id":"old-format/0","reason":"retention loss"}));
    memory.bytes.remove(&key(&lost));
    assert_eq!(Graph::resolve(&memory, &root).unwrap().object_count(), 1);
}

#[test]
fn complete_inventory_returns_all_pages_in_order_and_supports_canonical_empty() {
    let mut memory = Memory::default();
    let (first, last) = two_pages(&mut memory);
    let graph = Graph::resolve(&memory, &first).unwrap();
    assert_eq!(graph.object_count(), 2);
    let languages: Vec<_> = graph
        .rows(&first)
        .unwrap()
        .into_iter()
        .map(|row| row["language"].as_str().unwrap())
        .collect();
    assert_eq!(languages, ["julia", "rust"]);
    assert!(graph.rows(&last).is_err());
    let empty = memory.typed(FLAGS_PAGE, &page(0, 1, 0, vec![], None));
    assert!(
        Graph::resolve(&memory, &empty)
            .unwrap()
            .rows(&empty)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn inventory_root_cannot_start_at_a_nonzero_page_index() {
    let mut memory = Memory::default();
    let (_first, last) = two_pages(&mut memory);
    assert!(Graph::resolve(&memory, &last).is_err());
}

#[test]
fn unavailable_later_page_cannot_yield_a_partial_successful_inventory() {
    let mut memory = Memory::default();
    let (first, last) = two_pages(&mut memory);
    memory.bytes.remove(&key(&last));
    assert!(Graph::resolve(&memory, &first).is_err());
}

#[test]
fn complete_page_indexes_cannot_skip_an_intermediate_index() {
    let mut memory = Memory::default();
    let last = memory.typed(FLAGS_PAGE, &page(2, 3, 2, vec![flags("rust")], None));
    let first = memory.typed(
        FLAGS_PAGE,
        &page(0, 3, 2, vec![flags("julia")], Some(&last)),
    );
    assert!(Graph::resolve(&memory, &first).is_err());
}

#[test]
fn every_page_must_agree_on_page_count() {
    let mut memory = Memory::default();
    let last = memory.typed(FLAGS_PAGE, &page(1, 2, 2, vec![flags("rust")], None));
    let first = memory.typed(
        FLAGS_PAGE,
        &page(0, 3, 2, vec![flags("julia")], Some(&last)),
    );
    assert!(Graph::resolve(&memory, &first).is_err());
}

#[test]
fn every_page_must_agree_on_total_rows() {
    let mut memory = Memory::default();
    let last = memory.typed(FLAGS_PAGE, &page(1, 2, 3, vec![flags("rust")], None));
    let first = memory.typed(
        FLAGS_PAGE,
        &page(0, 2, 2, vec![flags("julia")], Some(&last)),
    );
    assert!(Graph::resolve(&memory, &first).is_err());
}

#[test]
fn summed_inventory_rows_must_equal_the_common_declared_total() {
    let mut memory = Memory::default();
    let last = memory.typed(FLAGS_PAGE, &page(1, 2, 3, vec![flags("rust")], None));
    let first = memory.typed(
        FLAGS_PAGE,
        &page(0, 2, 3, vec![flags("julia")], Some(&last)),
    );
    assert!(Graph::resolve(&memory, &first).is_err());
}

#[test]
fn duplicate_environment_row_identity_across_pages_refuses_different_values_too() {
    let mut memory = Memory::default();
    let row = |text: &str| json!({"name":"LANG","value":{"value":text,"unavailable_reason":null},"secret_handle":unavailable("literal")});
    let schema = "hee3.receipt/1:EnvironmentPageV1";
    let last = memory.typed(schema, &page(1, 2, 2, vec![row("C.UTF-8")], None));
    let first = memory.typed(schema, &page(0, 2, 2, vec![row("C")], Some(&last)));
    assert!(Graph::resolve(&memory, &first).is_err());
}

#[test]
fn subject_paths_are_unique_and_sorted_by_utf8_bytes_across_pages() {
    for (paths, succeeds) in [
        (vec!["Z", "a", "é"], true),
        (vec!["a", "a"], false),
        (vec!["é", "z"], false),
    ] {
        let mut memory = Memory::default();
        let raw = memory.raw(b"source");
        let first = file_chain(
            &mut memory,
            paths.iter().map(|path| file(path, &raw)).collect(),
        );
        let root = subject(&mut memory, &first);
        let result = Graph::resolve(&memory, &root);
        assert_eq!(result.is_ok(), succeeds);
    }
}

#[test]
fn subject_symlink_target_is_descriptive_and_never_opened_as_a_dependency() {
    let mut memory = Memory::default();
    let row = json!({"path":"external-link","kind":"symlink","content":unavailable("not followed"),"executable":false,
        "link_target":{"value":"/etc/passwd","unavailable_reason":null},"origin":"authored","exclusion_reason":unavailable("included")});
    let first = memory.typed(FILE_PAGE, &page(0, 1, 1, vec![row], None));
    let root = subject(&mut memory, &first);
    assert_eq!(Graph::resolve(&memory, &root).unwrap().object_count(), 2);
}

fn nested(memory: &mut Memory, nodes: usize) -> Ref {
    let mut root = memory.raw(b"leaf");
    for _ in 1..nodes {
        root = memory.typed(ARTIFACT, &artifact(&root));
    }
    root
}

#[test]
fn traversal_allows_64_nodes_on_a_path_and_refuses_the_65th() {
    for (nodes, succeeds) in [(64, true), (65, false)] {
        let mut memory = Memory::default();
        let root = nested(&mut memory, nodes);
        let result = Graph::resolve(&memory, &root);
        assert_eq!(result.is_ok(), succeeds);
        if let Ok(graph) = result {
            assert_eq!(graph.object_count(), nodes);
        }
    }
}

fn inventory_objects(memory: &mut Memory, payloads: usize) -> Ref {
    let rows: Vec<_> = (0..payloads).map(|_| artifact(&memory.raw(b"x"))).collect();
    let page_count = rows.len().div_ceil(256);
    let mut next = None;
    for (index, chunk) in rows.chunks(256).enumerate().rev() {
        next = Some(memory.typed(
            ARTIFACT_PAGE,
            &page(index, page_count, payloads, chunk.to_vec(), next.as_ref()),
        ));
    }
    next.unwrap()
}

#[test]
fn unique_object_bound_includes_pages_and_allows_exactly_4096() {
    for (payloads, succeeds) in [(4080, true), (4081, false)] {
        let mut memory = Memory::default();
        let root = inventory_objects(&mut memory, payloads);
        // Both inventories have16 pages, so these totals are4096 and4097.
        let result = Graph::resolve(&memory, &root);
        assert_eq!(result.is_ok(), succeeds);
        if let Ok(graph) = result {
            assert_eq!(graph.object_count(), 4096);
        }
    }
}

#[test]
fn raw_object_limit_accepts_16_mib_and_refuses_one_more_byte() {
    for (length, succeeds) in [(16_777_216, true), (16_777_217, false)] {
        let mut memory = Memory::default();
        let root = memory.insert(RAW, vec![7; length]);
        let result = Graph::resolve(&memory, &root);
        assert_eq!(result.is_ok(), succeeds);
    }
}

fn typed_bytes_at(length: usize) -> Vec<u8> {
    let mut argv = vec!["x".repeat(4096); 255];
    argv.push(String::new());
    let initial = serde_json::to_vec(&json!({"language":"rust","argv":argv}))
        .unwrap()
        .len();
    argv[255] = "x".repeat(length - initial);
    assert!(argv[255].len() <= 4096);
    let bytes = serde_json::to_vec(&json!({"language":"rust","argv":argv})).unwrap();
    assert_eq!(bytes.len(), length);
    bytes
}

#[test]
fn typed_object_limit_accepts_1_mib_and_refuses_one_more_byte() {
    for (length, succeeds) in [(1_048_576, true), (1_048_577, false)] {
        let mut memory = Memory::default();
        let root = memory.insert(FLAGS, typed_bytes_at(length));
        assert_eq!(Graph::resolve(&memory, &root).is_ok(), succeeds);
    }
}

fn aggregate(memory: &mut Memory, extra: usize) -> Ref {
    let mut refs = Vec::new();
    for byte in [1, 2, 3] {
        refs.push(memory.insert(RAW, vec![byte; 16_777_216]));
    }
    // A provisional exact reference supplies fixed-width fields to measure the
    // page overhead. It is removed; the fourth actual object has reduced bytes.
    let provisional = memory.insert(RAW, vec![4; 16_777_216]);
    refs.push(provisional.clone());
    let overhead = serde_json::to_vec(&page(0, 1, 4, refs.iter().map(artifact).collect(), None))
        .unwrap()
        .len();
    memory.bytes.remove(&key(&provisional));
    refs[3] = memory.insert(RAW, vec![4; 16_777_216 - overhead + extra]);
    let root = memory.typed(
        ARTIFACT_PAGE,
        &page(0, 1, 4, refs.iter().map(artifact).collect(), None),
    );
    let actual: u64 = refs
        .iter()
        .map(|item| u64::from(item.byte_length))
        .sum::<u64>()
        + u64::from(root.byte_length);
    assert_eq!(actual, 67_108_864 + u64::try_from(extra).unwrap());
    root
}

#[test]
fn aggregate_limit_includes_page_metadata_and_accepts_exactly_64_mib() {
    for (extra, succeeds) in [(0, true), (1, false)] {
        let mut memory = Memory::default();
        let root = aggregate(&mut memory, extra);
        let result = Graph::resolve(&memory, &root);
        assert_eq!(result.is_ok(), succeeds);
        if let Ok(graph) = result {
            assert_eq!(graph.total_bytes(), 67_108_864);
        }
    }
}

#[test]
fn retained_real_hash_back_edge_cannot_reuse_ancestor_id_with_other_metadata() {
    let mut memory = Memory::default();
    let descendant = memory.raw(b"actual retained leaf bytes");
    let root = memory.insert_as(
        descendant.artifact_id.clone(),
        ARTIFACT,
        serde_json::to_vec(&artifact(&descendant)).unwrap(),
    );
    assert_eq!(memory.bytes.len(), 2);
    assert!(Graph::resolve(&memory, &root).is_err());
    // Both refs hash real available bytes. This is an ID-conflicting back-edge,
    // not a fabricated SHA-256 fixed point or a qualified hash-cycle detector.
}

struct EndlessObjects {
    emitted: Cell<usize>,
}
struct EndlessReader<'a> {
    emitted: &'a Cell<usize>,
}
impl Read for EndlessReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        buffer.fill(0);
        self.emitted.set(self.emitted.get() + buffer.len());
        Ok(buffer.len())
    }
}
impl Objects for EndlessObjects {
    fn open(&self, _reference: &Ref) -> Result<Box<dyn Read + '_>, Error> {
        Ok(Box::new(EndlessReader {
            emitted: &self.emitted,
        }))
    }
}

#[test]
fn unending_reader_cannot_escape_the_declared_raw_object_read_bound() {
    let mut memory = Memory::default();
    let root = memory.raw(&[0]);
    let endless = EndlessObjects {
        emitted: Cell::new(0),
    };
    assert!(Graph::resolve(&endless, &root).is_err());
    assert!(endless.emitted.get() <= 16_777_217);
}

#[test]
fn page_successor_type_and_rows_query_require_actual_inventory_types() {
    let mut memory = Memory::default();
    let wrong = memory.typed(
        "hee3.receipt/1:EnvironmentPageV1",
        &page(0, 1, 0, vec![], None),
    );
    let first = memory.typed(
        FLAGS_PAGE,
        &page(0, 2, 1, vec![flags("rust")], Some(&wrong)),
    );
    assert!(Graph::resolve(&memory, &first).is_err());
    let ordinary = memory.typed(FLAGS, &flags("rust"));
    let graph = Graph::resolve(&memory, &ordinary).unwrap();
    assert!(graph.rows(&ordinary).is_err());
}
