//! RC04 append controls authored from the public contract, not resolver source.
//! Historical fixture identities are fictional; these tests claim no authenticity.
use habitat_engine::check::collector::{Publisher, Sink, SinkError};
use habitat_engine::check::graph::{AvailabilityAppend, Error, Graph, Objects};
use habitat_engine::contracts::receipt::{
    AvailabilityReceiptV1, Id, MissingObjectPageV1, Name, ReceiptRecord, ReceiptV1, Ref, Sha,
    TypedRef, decode,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{Cursor, Read};

fn id(n: u64) -> Id {
    Id::new(format!("00000000-0000-4000-8000-{n:012x}")).unwrap()
}
fn raw_ref<T: ReceiptRecord>(n: u64, bytes: &[u8]) -> TypedRef<T> {
    let mut digest = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(digest, "{byte:02x}").unwrap();
    }
    TypedRef::new(Ref {
        artifact_id: id(n),
        sha256: Sha::new(digest).unwrap(),
        byte_length: u32::try_from(bytes.len()).unwrap(),
        media_type: Name::new("application/json").unwrap(),
        schema_id: Name::new(T::SCHEMA_ID).unwrap(),
    })
    .unwrap()
}
#[derive(Default)]
struct Memory {
    bytes: BTreeMap<String, Vec<u8>>,
    hashes: BTreeMap<String, Vec<u8>>,
    opens: RefCell<Vec<String>>,
    by_hash: bool,
    failed_id: Option<String>,
    next_id: u64,
    published: Vec<Ref>,
    corrupt_writes: bool,
}
impl Memory {
    fn put<T: ReceiptRecord>(&mut self, n: u64, value: &Value) -> TypedRef<T> {
        let bytes = serde_json::to_vec(value).unwrap();
        let reference = raw_ref::<T>(n, &bytes);
        self.bytes.insert(
            reference.as_ref().artifact_id.as_str().to_owned(),
            bytes.clone(),
        );
        self.hashes
            .insert(reference.as_ref().sha256.as_str().to_owned(), bytes);
        reference
    }
}
impl Objects for Memory {
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, Error> {
        let key = reference.artifact_id.as_str();
        self.opens.borrow_mut().push(key.to_owned());
        if self.failed_id.as_deref() == Some(key) {
            return Err(Error::Io);
        }
        let bytes = if self.by_hash {
            self.hashes.get(reference.sha256.as_str())
        } else {
            self.bytes.get(key)
        }
        .ok_or(Error::Missing)?;
        Ok(Box::new(Cursor::new(bytes.as_slice())))
    }
}
impl Sink for Memory {
    fn fresh_id(&mut self) -> Result<Id, SinkError> {
        self.next_id += 1;
        Ok(id(100_000 + self.next_id))
    }
    fn contains_id(&self, id: &Id) -> Result<bool, SinkError> {
        Ok(self.bytes.contains_key(id.as_str()))
    }
    fn publish(&mut self, reference: &Ref, bytes: &[u8]) -> Result<Ref, SinkError> {
        let mut actual = bytes.to_vec();
        if self.corrupt_writes {
            actual.push(b' ');
        }
        self.bytes
            .insert(reference.artifact_id.as_str().to_owned(), actual);
        self.published.push(reference.clone());
        Ok(reference.clone())
    }
}
fn history() -> Value {
    let source: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/receipts/inventory-examples.json"
    )))
    .unwrap();
    let value = source["definitions"]["ReceiptV1"].clone();
    let _: ReceiptV1 = decode(&serde_json::to_vec(&value).unwrap()).unwrap();
    value
}
fn descriptor(n: u64) -> Value {
    json!({"artifact_id":id(n).as_str(),"expected_sha256":format!("sha256:{}","1".repeat(64)),
        "expected_byte_length":0,"expected_schema_id":"hee3.raw/1","reason":"lost"})
}
fn pages(count: u32) -> Vec<Value> {
    (0..count).map(|i|json!({"page_index":i,"page_count":count,"row_count":1,"total_rows":count,
        "rows":[descriptor(10_000+u64::from(i))],"next":{"value":null,"unavailable_reason":"end_of_inventory"}})).collect()
}
struct Fixture {
    owner: Memory,
    envelope: Value,
    root: TypedRef<AvailabilityReceiptV1>,
    historical: TypedRef<ReceiptV1>,
    pages: Vec<TypedRef<MissingObjectPageV1>>,
}
impl Fixture {
    fn new(values: Vec<Value>) -> Self {
        Self::with_ids(values, false)
    }
    fn with_ids(mut values: Vec<Value>, alias: bool) -> Self {
        let mut owner = Memory::default();
        let historical = owner.put::<ReceiptV1>(2, &history());
        let mut next: Option<TypedRef<MissingObjectPageV1>> = None;
        let mut references = Vec::new();
        for (index, value) in values.iter_mut().enumerate().rev() {
            if let Some(reference) = &next {
                value["next"] = json!({"value":reference,"unavailable_reason":null});
            }
            let n = if alias {
                100
            } else {
                100 + u64::try_from(index).unwrap()
            };
            let reference = owner.put::<MissingObjectPageV1>(n, value);
            references.push(reference.clone());
            next = Some(reference);
        }
        references.reverse();
        owner.by_hash = alias;
        let envelope = json!({"protocol":"hee3.receipt.availability","version":1,"receipt":historical,
            "availability":{"observed_unix_ms":"1","state":"incomplete","missing_objects":next.unwrap(),"retention_policy":"retain-v1"}});
        let root = owner.put::<AvailabilityReceiptV1>(1, &envelope);
        Self {
            owner,
            envelope,
            root,
            historical,
            pages: references,
        }
    }
    fn empty() -> Self {
        let mut f = Self::new(vec![
            json!({"page_index":0,"page_count":1,"row_count":0,"total_rows":0,
            "rows":[],"next":{"value":null,"unavailable_reason":"end_of_inventory"}}),
        ]);
        f.envelope["availability"]["state"] = json!("complete");
        f.refresh();
        f
    }
    fn refresh(&mut self) {
        self.root = self.owner.put::<AvailabilityReceiptV1>(1, &self.envelope);
    }
    fn replace_history(&mut self, value: &Value) {
        self.historical = self.owner.put::<ReceiptV1>(2, value);
        self.envelope["receipt"] = serde_json::to_value(&self.historical).unwrap();
        self.refresh();
    }
    fn resolve(&self) -> Result<AvailabilityAppend, Error> {
        AvailabilityAppend::resolve(&self.owner, &self.root)
    }
}
#[test]
fn missing_historical_payloads_preserve_exact_manifests_but_never_relax_graph() {
    let f = Fixture::new(pages(1));
    let a = f.resolve().unwrap();
    assert_eq!(a.envelope_bytes().unwrap(), f.owner.bytes[id(1).as_str()]);
    assert_eq!(a.historical_bytes().unwrap(), f.owner.bytes[id(2).as_str()]);
    assert_eq!(a.envelope().receipt, f.historical);
    assert_eq!(a.historical().identity.module_id.as_str(), "check");
    assert_eq!(a.missing_objects().unwrap().len(), 1);
    assert_eq!(a.metadata_object_count(), 3);
    assert_eq!(
        a.total_bytes(),
        f.owner
            .bytes
            .values()
            .map(|v| u64::try_from(v.len()).unwrap())
            .sum::<u64>()
    );
    let opens = f.owner.opens.borrow().clone();
    assert!(opens.iter().all(|id| f.owner.bytes.contains_key(id)));
    assert!(Graph::resolve(&f.owner, f.historical.as_ref()).is_err());
}
#[test]
fn complete_empty_new_inventory_does_not_require_old_missing_inventory() {
    let f = Fixture::empty();
    let a = f.resolve().unwrap();
    assert!(a.missing_objects().unwrap().is_empty());
    let old = a
        .historical()
        .availability
        .missing_objects
        .as_ref()
        .artifact_id
        .as_str();
    assert!(!f.owner.opens.borrow().iter().any(|id| id == old));
}
#[test]
fn complete_with_missing_descriptors_is_rejected() {
    let mut f = Fixture::new(pages(1));
    f.envelope["availability"]["state"] = json!("complete");
    f.refresh();
    assert!(f.resolve().is_err());
}
#[test]
fn new_page_chain_is_complete_and_preserves_row_order() {
    let f = Fixture::new(pages(2));
    let a = f.resolve().unwrap();
    let rows = a.missing_objects().unwrap();
    assert_eq!(
        rows.iter()
            .map(|r| r.artifact_id.clone())
            .collect::<Vec<_>>(),
        vec![id(10_000), id(10_001)]
    );
    assert_eq!(a.metadata_object_count(), 4);
}
#[test]
fn missing_new_next_page_is_not_historical_retention_loss() {
    let mut f = Fixture::new(pages(2));
    f.owner
        .bytes
        .remove(f.pages[1].as_ref().artifact_id.as_str());
    assert!(f.resolve().is_err());
}
#[test]
fn historical_hash_is_verified_against_returned_bytes() {
    let mut f = Fixture::new(pages(1));
    let bytes = f.owner.bytes.get_mut(id(2).as_str()).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let changed = text.replacen("\"generation\":\"1\"", "\"generation\":\"2\"", 1);
    assert_ne!(changed.as_bytes(), bytes.as_slice());
    *bytes = changed.into_bytes();
    assert!(f.resolve().is_err());
}
#[test]
fn historical_declared_length_is_exact() {
    let mut f = Fixture::new(pages(1));
    let len = f.historical.as_ref().byte_length;
    f.envelope["receipt"]["byte_length"] = json!(len + 1);
    f.refresh();
    assert!(f.resolve().is_err());
}
#[test]
fn historical_closed_required_shape_is_not_skipped() {
    let mut f = Fixture::new(pages(1));
    let mut old = history();
    old["unexpected"] = json!(true);
    f.replace_history(&old);
    assert!(f.resolve().is_err());
    old.as_object_mut().unwrap().remove("unexpected");
    old.as_object_mut().unwrap().remove("identity");
    f.replace_history(&old);
    assert!(f.resolve().is_err());
}
#[test]
fn historical_local_pass_guard_is_still_enforced() {
    let mut f = Fixture::new(pages(1));
    let mut old = history();
    old["observations"]["cleanup"] = json!("pending");
    f.replace_history(&old);
    assert!(f.resolve().is_err());
}
#[test]
fn envelope_hash_is_verified_without_normalization() {
    let mut f = Fixture::new(pages(1));
    let mut reference = f.root.clone().into_inner();
    reference.sha256 = Sha::new(format!("sha256:{}", "0".repeat(64))).unwrap();
    f.root = TypedRef::new(reference).unwrap();
    assert!(f.resolve().is_err());
}
#[test]
fn new_page_hash_is_checked_even_when_locally_valid() {
    let mut f = Fixture::new(pages(1));
    let bytes = f.owner.bytes.get_mut(id(100).as_str()).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let changed = text.replacen("lost", "gone", 1);
    assert_ne!(changed.as_bytes(), bytes.as_slice());
    *bytes = changed.into_bytes();
    assert!(f.resolve().is_err());
}
#[test]
fn new_inventory_target_schema_is_closed() {
    let mut f = Fixture::new(pages(1));
    f.envelope["availability"]["missing_objects"]["schema_id"] = json!("hee3.receipt/1:CasePageV1");
    f.refresh();
    assert!(f.resolve().is_err());
}
#[test]
fn new_page_index_cannot_skip_a_position() {
    let mut p = pages(2);
    p[0]["page_count"] = json!(3);
    p[1]["page_count"] = json!(3);
    p[1]["page_index"] = json!(2);
    assert!(Fixture::new(p).resolve().is_err());
}
#[test]
fn new_page_totals_must_agree() {
    let mut p = pages(2);
    p[1]["total_rows"] = json!(3);
    assert!(Fixture::new(p).resolve().is_err());
}
#[test]
fn duplicate_missing_descriptor_identity_across_pages_refuses() {
    let mut p = pages(2);
    p[1]["rows"][0]["artifact_id"] = p[0]["rows"][0]["artifact_id"].clone();
    assert!(Fixture::new(p).resolve().is_err());
}
#[test]
fn metadata_uuid_alias_refuses_even_when_each_hash_resolves() {
    let f = Fixture::with_ids(pages(2), true);
    assert!(f.resolve().is_err());
}
#[test]
fn historical_typed_byte_limit_applies_before_full_resolution() {
    let mut f = Fixture::new(pages(1));
    let mut old = history();
    old["invocation"]["argv"] = json!(vec!["x".repeat(4096); 256]);
    assert!(serde_json::to_vec(&old).unwrap().len() > 1024 * 1024);
    f.replace_history(&old);
    assert!(f.resolve().is_err());
}
#[test]
fn new_chain_depth_boundary_is_63_accepted_64_rejected() {
    let benign = Fixture::new(pages(63));
    assert_eq!(
        benign.resolve().unwrap().missing_objects().unwrap().len(),
        63
    );
    assert!(Fixture::new(pages(64)).resolve().is_err());
}
#[test]
fn new_metadata_owner_read_error_refuses() {
    let mut f = Fixture::new(pages(1));
    f.owner.failed_id = Some(id(100).as_str().to_owned());
    assert!(f.resolve().is_err());
}
#[test]
fn publication_retains_exact_append_without_old_payloads() {
    let mut f = Fixture::new(pages(1));
    let envelope: AvailabilityReceiptV1 =
        decode(&serde_json::to_vec(&f.envelope).unwrap()).unwrap();
    let result = Publisher::new(&mut f.owner)
        .finalize_availability(&envelope)
        .unwrap();
    assert_eq!(
        result.bytes,
        f.owner.bytes[result.reference.as_ref().artifact_id.as_str()]
    );
    let a = AvailabilityAppend::resolve(&f.owner, &result.reference).unwrap();
    assert_eq!(a.envelope(), &envelope);
    assert_eq!(f.owner.published.len(), 1);
    assert!(Graph::resolve(&f.owner, f.historical.as_ref()).is_err());
}
#[test]
fn invalid_append_never_publishes_root_and_keeps_attempt() {
    let mut f = Fixture::new(pages(1));
    f.envelope["availability"]["state"] = json!("complete");
    let envelope: AvailabilityReceiptV1 =
        decode(&serde_json::to_vec(&f.envelope).unwrap()).unwrap();
    {
        let mut publisher = Publisher::new(&mut f.owner);
        assert!(publisher.finalize_availability(&envelope).is_err());
        assert_eq!(publisher.attempted_refs().len(), 1);
    }
    assert!(f.owner.published.is_empty());
}
#[test]
fn changed_sink_bytes_refuse_without_claiming_rollback() {
    let mut f = Fixture::new(pages(1));
    f.owner.corrupt_writes = true;
    let envelope: AvailabilityReceiptV1 =
        decode(&serde_json::to_vec(&f.envelope).unwrap()).unwrap();
    {
        let mut publisher = Publisher::new(&mut f.owner);
        assert!(publisher.finalize_availability(&envelope).is_err());
        assert_eq!(publisher.attempted_refs().len(), 1);
    }
    assert_eq!(f.owner.published.len(), 1);
    assert!(
        f.owner
            .bytes
            .contains_key(f.owner.published[0].artifact_id.as_str())
    );
}
