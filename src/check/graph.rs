//! Bounded exact-byte receipt reference resolution. No paths or network URLs
//! from a receipt are followed; the trusted object owner supplies read handles.

use crate::contracts::receipt::{self, Record, Ref, Validate};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Read;

const MAX_OBJECTS: usize = 4096;
const MAX_GRAPH_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RAW_BYTES: u32 = 16 * 1024 * 1024;
const MAX_DEPTH: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Missing,
    Io,
    Bound,
    Identity,
    Encoding,
    Cycle,
    Inventory,
}

/// Open an immutable object by its registered identity. Implementations must
/// use owned descriptors; descriptive candidate paths are never lookup inputs.
pub trait Objects {
    /// # Errors
    /// Refuses unavailable/corrupt object custody before returning a read handle.
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, Error>;
}

#[derive(Debug)]
pub struct Node {
    reference: Ref,
    bytes: Vec<u8>,
    record: Option<Record>,
    value: Option<Value>,
}
impl Node {
    #[must_use]
    pub const fn reference(&self) -> &Ref {
        &self.reference
    }
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub const fn record(&self) -> Option<&Record> {
        self.record.as_ref()
    }
}

/// A complete, bounded graph whose bytes, schemas and page chains were checked.
/// This proves internal consistency only; collector custody and actual oracle
/// truth still require independent observations.
#[derive(Debug)]
pub struct Graph {
    nodes: BTreeMap<String, Node>,
    bytes: u64,
}
impl Graph {
    /// Resolve the entire reference closure, or refuse without a partial success.
    /// # Errors
    /// Rejects missing/changed/oversized bytes, ID aliases, unknown schemas,
    /// cycles, invalid typed values and incomplete or contradictory inventories.
    pub fn resolve(owner: &impl Objects, root: &Ref) -> Result<Self, Error> {
        let mut graph = Self {
            nodes: BTreeMap::new(),
            bytes: 0,
        };
        graph.visit(owner, root, &mut BTreeSet::new(), 0)?;
        graph.validate_pages()?;
        if is_page(root.schema_id.as_str()) {
            graph.rows(root)?;
        }
        Ok(graph)
    }

    /// Borrow only when the entire reference metadata matches the checked node.
    /// # Errors
    /// Refuses an absent ID or any substituted reference field.
    pub fn get(&self, reference: &Ref) -> Result<&Node, Error> {
        let node = self
            .nodes
            .get(reference.artifact_id.as_str())
            .ok_or(Error::Missing)?;
        if &node.reference != reference {
            return Err(Error::Identity);
        }
        Ok(node)
    }

    #[must_use]
    pub fn object_count(&self) -> usize {
        self.nodes.len()
    }
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.bytes
    }

    fn visit(
        &mut self,
        owner: &impl Objects,
        reference: &Ref,
        active: &mut BTreeSet<String>,
        depth: usize,
    ) -> Result<(), Error> {
        reference.validate().map_err(|_| Error::Encoding)?;
        let id = reference.artifact_id.as_str();
        if active.contains(id) {
            return Err(Error::Cycle);
        }
        if let Some(prior) = self.nodes.get(id) {
            return if &prior.reference == reference {
                Ok(())
            } else {
                Err(Error::Identity)
            };
        }
        if depth >= MAX_DEPTH || self.nodes.len() >= MAX_OBJECTS {
            return Err(Error::Bound);
        }
        self.load_new(owner, reference)?;
        let mut links = Vec::new();
        if let Some(value) = &self.get(reference)?.value {
            references(value, false, &mut links)?;
        }
        active.insert(id.to_owned());
        for (link, continuation) in links {
            self.visit(owner, &link, active, depth + 1)?;
            if !continuation && is_page(link.schema_id.as_str()) {
                let page = self.get(&link)?.value.as_ref().ok_or(Error::Inventory)?;
                if count(page, "page_index")? != 0 {
                    return Err(Error::Inventory);
                }
            }
        }
        active.remove(id);
        Ok(())
    }

    fn load_new(&mut self, owner: &impl Objects, reference: &Ref) -> Result<(), Error> {
        reference.validate().map_err(|_| Error::Encoding)?;
        if self.nodes.len() >= MAX_OBJECTS {
            return Err(Error::Bound);
        }
        if self.nodes.contains_key(reference.artifact_id.as_str()) {
            return Err(Error::Identity);
        }
        let raw = reference.schema_id.as_str() == "hee3.raw/1";
        let limit = if raw {
            MAX_RAW_BYTES
        } else {
            u32::try_from(receipt::MAX_BYTES).map_err(|_| Error::Bound)?
        };
        if reference.byte_length > limit {
            return Err(Error::Bound);
        }
        let total = self
            .bytes
            .checked_add(u64::from(reference.byte_length))
            .ok_or(Error::Bound)?;
        if total > MAX_GRAPH_BYTES {
            return Err(Error::Bound);
        }
        let mut bytes = Vec::new();
        owner
            .open(reference)?
            .take(u64::from(reference.byte_length) + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::Io)?;
        if bytes.len() != reference.byte_length as usize
            || digest(&bytes) != reference.sha256.as_str()
        {
            return Err(Error::Identity);
        }
        let (record, value) = if raw {
            (None, None)
        } else {
            let record = receipt::decode_record(reference.schema_id.as_str(), &bytes)
                .map_err(|_| Error::Encoding)?;
            // The closed DTO decoder above authorizes the exact fixed wire shape.
            // A Value here is only a traversal view, never an untyped protocol input.
            let value: Value = serde_json::from_slice(&bytes).map_err(|_| Error::Encoding)?;
            (Some(record), Some(value))
        };
        self.bytes = total;
        self.nodes.insert(
            reference.artifact_id.as_str().to_owned(),
            Node {
                reference: reference.clone(),
                bytes,
                record,
                value,
            },
        );
        Ok(())
    }

    fn validate_pages(&self) -> Result<(), Error> {
        for node in self.nodes.values() {
            if !is_page(node.reference.schema_id.as_str()) {
                continue;
            }
            let page = node.value.as_ref().ok_or(Error::Inventory)?;
            if count(page, "page_index")? == 0 {
                self.rows(&node.reference)?;
            }
        }
        Ok(())
    }

    /// Complete ordered rows from one first page. The row values have already
    /// passed their fixed DTO validators; callers must still check observation truth.
    /// # Errors
    /// Refuses nonfirst pages, missing links, inconsistent counts, duplicate row
    /// identities and unsorted/repeated subject paths.
    pub fn rows(&self, first: &Ref) -> Result<Vec<&Value>, Error> {
        if !is_page(first.schema_id.as_str()) {
            return Err(Error::Inventory);
        }
        let initial = self.get(first)?.value.as_ref().ok_or(Error::Inventory)?;
        if count(initial, "page_index")? != 0 {
            return Err(Error::Inventory);
        }
        let pages = count(initial, "page_count")?;
        let total = count(initial, "total_rows")?;
        if pages as usize > self.nodes.len() {
            return Err(Error::Inventory);
        }
        let mut current = first.clone();
        let mut seen_pages = BTreeSet::new();
        let mut seen_rows = BTreeSet::new();
        let mut rows = Vec::new();
        let mut last_path: Option<&str> = None;
        for index in 0..pages {
            if !seen_pages.insert(current.artifact_id.as_str().to_owned()) {
                return Err(Error::Cycle);
            }
            let page = self.get(&current)?.value.as_ref().ok_or(Error::Inventory)?;
            if current.schema_id != first.schema_id
                || count(page, "page_index")? != index
                || count(page, "page_count")? != pages
                || count(page, "total_rows")? != total
            {
                return Err(Error::Inventory);
            }
            for row in page
                .get("rows")
                .and_then(Value::as_array)
                .ok_or(Error::Inventory)?
            {
                let key = row_key(first.schema_id.as_str(), row)?;
                if !seen_rows.insert(key) {
                    return Err(Error::Inventory);
                }
                if first.schema_id.as_str() == "hee3.receipt/1:SubjectFilePageV1" {
                    let path = row
                        .get("path")
                        .and_then(Value::as_str)
                        .ok_or(Error::Inventory)?;
                    if last_path.is_some_and(|last| last.as_bytes() >= path.as_bytes()) {
                        return Err(Error::Inventory);
                    }
                    last_path = Some(path);
                }
                rows.push(row);
            }
            if index + 1 < pages {
                current = serde_json::from_value(
                    page.get("next")
                        .and_then(|v| v.get("value"))
                        .cloned()
                        .ok_or(Error::Inventory)?,
                )
                .map_err(|_| Error::Inventory)?;
            }
        }
        if rows.len() != total as usize {
            return Err(Error::Inventory);
        }
        Ok(rows)
    }
}

/// A retention observation with complete NEW metadata and only the historical
/// root's exact bytes/shape. It deliberately is not a complete historical Graph.
#[derive(Debug)]
pub struct AvailabilityAppend {
    graph: Graph,
    root: Ref,
    envelope: receipt::AvailabilityReceiptV1,
    historical: receipt::ReceiptV1,
}

impl AvailabilityAppend {
    /// Resolve a retention-loss append without requiring lost historical payloads.
    ///
    /// # Errors
    /// Refuses unavailable/changed prior root bytes, malformed records, collisions,
    /// invalid new descriptor-page chains or nonempty missing inventory marked complete.
    pub fn resolve(
        owner: &impl Objects,
        root: &receipt::TypedRef<receipt::AvailabilityReceiptV1>,
    ) -> Result<Self, Error> {
        let mut graph = Graph {
            nodes: BTreeMap::new(),
            bytes: 0,
        };
        graph.load_new(owner, root.as_ref())?;
        let envelope: receipt::AvailabilityReceiptV1 =
            receipt::decode(graph.get(root.as_ref())?.bytes()).map_err(|_| Error::Encoding)?;
        graph.load_new(owner, envelope.receipt.as_ref())?;
        let historical: receipt::ReceiptV1 =
            receipt::decode(graph.get(envelope.receipt.as_ref())?.bytes())
                .map_err(|_| Error::Encoding)?;
        graph.visit(
            owner,
            envelope.availability.missing_objects.as_ref(),
            &mut BTreeSet::new(),
            1,
        )?;
        graph.validate_pages()?;
        if envelope.availability.state == receipt::AvailabilityV1State::Complete
            && !graph
                .rows(envelope.availability.missing_objects.as_ref())?
                .is_empty()
        {
            return Err(Error::Inventory);
        }
        Ok(Self {
            graph,
            root: root.as_ref().clone(),
            envelope,
            historical,
        })
    }

    #[must_use]
    pub const fn envelope(&self) -> &receipt::AvailabilityReceiptV1 {
        &self.envelope
    }

    /// The historical root passed structure/hash checks; its descendants may be lost.
    #[must_use]
    pub const fn historical(&self) -> &receipt::ReceiptV1 {
        &self.historical
    }

    /// # Errors
    /// Refuses any internal reference identity mismatch.
    pub fn envelope_bytes(&self) -> Result<&[u8], Error> {
        Ok(self.graph.get(&self.root)?.bytes())
    }

    /// # Errors
    /// Refuses any internal reference identity mismatch.
    pub fn historical_bytes(&self) -> Result<&[u8], Error> {
        Ok(self.graph.get(self.envelope.receipt.as_ref())?.bytes())
    }

    /// Complete validated missing-object descriptors; their target bytes are not resolved.
    /// # Errors
    /// Refuses a contradictory inventory or malformed descriptor.
    pub fn missing_objects(&self) -> Result<Vec<receipt::MissingObjectV1>, Error> {
        self.graph
            .rows(self.envelope.availability.missing_objects.as_ref())?
            .into_iter()
            .map(|row| serde_json::from_value(row.clone()).map_err(|_| Error::Encoding))
            .collect()
    }

    #[must_use]
    pub fn metadata_object_count(&self) -> usize {
        self.graph.object_count()
    }

    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.graph.total_bytes()
    }
}

fn count(value: &Value, name: &str) -> Result<u32, Error> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|n| n.try_into().ok())
        .ok_or(Error::Inventory)
}

fn is_page(schema: &str) -> bool {
    // Every supported page name belongs to the already validated fixed vocabulary.
    schema.ends_with("PageV1")
}

fn references(value: &Value, continuation: bool, out: &mut Vec<(Ref, bool)>) -> Result<(), Error> {
    match value {
        Value::Object(fields)
            if fields.len() == 5
                && fields.contains_key("schema_id")
                && fields.contains_key("artifact_id")
                && fields.contains_key("sha256")
                && fields.contains_key("byte_length")
                && fields.contains_key("media_type") =>
        {
            let reference: Ref =
                serde_json::from_value(value.clone()).map_err(|_| Error::Encoding)?;
            reference.validate().map_err(|_| Error::Encoding)?;
            out.push((reference, continuation));
        }
        Value::Object(fields) => {
            for (key, child) in fields {
                references(child, continuation || key == "next", out)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                references(child, continuation, out)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn row_key(schema: &str, row: &Value) -> Result<String, Error> {
    let field = match schema {
        "hee3.receipt/1:CasePageV1" => "case_id",
        "hee3.receipt/1:SubjectFilePageV1" => "path",
        "hee3.receipt/1:LockPageV1" => "lock_id",
        "hee3.receipt/1:DependencyPageV1" => "dependency_id",
        "hee3.receipt/1:ToolPageV1" | "hee3.receipt/1:DiagnosticPageV1" => "tool_id",
        "hee3.receipt/1:LanguageFlagsPageV1" => "language",
        "hee3.receipt/1:StandardPageV1" => "standard_id",
        "hee3.receipt/1:EnvironmentPageV1" => "name",
        "hee3.receipt/1:GrantPageV1" => "grant_id",
        "hee3.receipt/1:EffectPageV1" => "effect_id",
        "hee3.receipt/1:ResourcePageV1" => "metric",
        "hee3.receipt/1:ObligationPageV1" => "obligation_id",
        "hee3.receipt/1:CampaignPageV1" => "campaign_id",
        "hee3.receipt/1:MutantPageV1" => "mutant_id",
        "hee3.receipt/1:AssumptionPageV1" => "assumption_id",
        "hee3.receipt/1:FindingPageV1" => "finding_id",
        "hee3.receipt/1:MissingObjectPageV1" => "artifact_id",
        "hee3.receipt/1:ArtifactPageV1" => {
            return row
                .get("object")
                .and_then(|v| v.get("artifact_id"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or(Error::Inventory);
        }
        _ => return Err(Error::Inventory),
    };
    row.get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or(Error::Inventory)
}

fn digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut value = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 15)]));
    }
    value
}
