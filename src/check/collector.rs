//! Typed receipt assembly over trusted observations and an immutable object sink.
//! Internal consistency is not observation authenticity, custody or admission.

use super::{consistency, graph};
use crate::contracts::receipt::{
    self, Address, ArtifactPageV1, ArtifactV1, ArtifactsV1, AvailabilityReceiptV1, AvailabilityV1,
    CampaignPageV1, CampaignV1, CasePageV1, CaseV1, CaseV1Outcome, CasesV1, DiagnosticPageV1,
    DiagnosticV1, DiagnosticsV1, Id, List, Maybe, MissingObjectPageV1, MissingObjectV1,
    MutantPageV1, MutantV1, MutantV1Outcome, MutationV1, Name, ObligationPageV1, ObligationV1,
    ObservationsV1, ProducerV1, ReceiptRecord, ReceiptV1, ReceiptV1Protocol,
    ReceiptV1Serialization, Ref, ResourcePageV1, ResourceV1, ReviewReceiptV1, Text, TypedRef, U64,
    VerdictV1,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read};

const MAX_OBJECTS: usize = 4096;
const MAX_GRAPH_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SinkError {
    Unavailable,
    Publication,
}
/// Trusted app-owned registered evidence lookup and immutable publication. The
/// registry covers this collection's input/output closure, not all historical IDs.
pub trait Sink: graph::Objects {
    /// # Errors
    /// Refuses inability to allocate a `UUIDv4` unused in the registered closure.
    fn fresh_id(&mut self) -> Result<Id, SinkError>;
    /// # Errors
    /// Refuses inability to determine registered ID presence.
    fn contains_id(&self, id: &Id) -> Result<bool, SinkError>;
    /// Publish exactly the requested bytes/metadata, retaining incomplete effects.
    /// # Errors
    /// Refuses unsuccessful immutable publication; it must not claim rollback.
    fn publish(&mut self, reference: &Ref, bytes: &[u8]) -> Result<Ref, SinkError>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Sink(SinkError),
    Encoding(receipt::Error),
    Graph(graph::Error),
    Consistency(consistency::Error),
    Identity,
    Bound,
    CasePlan,
    UnsupportedRoot,
}
impl From<SinkError> for Error {
    fn from(e: SinkError) -> Self {
        Self::Sink(e)
    }
}
impl From<receipt::Error> for Error {
    fn from(e: receipt::Error) -> Self {
        Self::Encoding(e)
    }
}
impl From<graph::Error> for Error {
    fn from(e: graph::Error) -> Self {
        Self::Graph(e)
    }
}
impl From<consistency::Error> for Error {
    fn from(e: consistency::Error) -> Self {
        Self::Consistency(e)
    }
}

#[derive(Clone, Debug)]
pub struct CaseObservation {
    pub case_id: Name,
    pub executed: bool,
    pub outcome: CaseV1Outcome,
    pub producer: ProducerV1,
    pub detector_id: Name,
    pub benign_pair_id: Maybe<Id>,
    pub raw_evidence_refs: List<Ref>,
    pub reason: Text,
}
#[derive(Debug)]
pub struct Observed {
    pub observations: ObservationsV1,
    pub cases: Vec<CaseObservation>,
    pub diagnostic_baseline: bool,
    pub diagnostics: Vec<DiagnosticV1>,
    pub diagnostic_mismatch: Maybe<Text>,
    pub artifacts: Vec<ArtifactV1>,
    pub artifacts_finalized: bool,
    pub campaigns: Vec<CampaignV1>,
    pub verdict: VerdictV1,
    pub availability: AvailabilityV1,
}
#[derive(Debug)]
pub struct Finalized {
    pub reference: TypedRef<ReceiptV1>,
    pub bytes: Vec<u8>,
    pub summary: consistency::Summary,
}

/// Exact appended bytes. Publication grants no review authority or current admission.
#[derive(Debug)]
pub struct FinalizedAppend<T: ReceiptRecord> {
    pub reference: TypedRef<T>,
    pub bytes: Vec<u8>,
}
/// A bounded publication session, not an evidence registry or process collector.
pub struct Publisher<'a, S: Sink> {
    sink: &'a mut S,
    attempted: Vec<Ref>,
    allocated: BTreeSet<String>,
    bytes: u64,
}
impl<'a, S: Sink> Publisher<'a, S> {
    pub fn new(sink: &'a mut S) -> Self {
        Self {
            sink,
            attempted: Vec::new(),
            allocated: BTreeSet::new(),
            bytes: 0,
        }
    }
    /// Includes reserved roots and failed publication attempts for reconciliation.
    #[must_use]
    pub fn attempted_refs(&self) -> &[Ref] {
        &self.attempted
    }

    /// Append a retention observation after checking the prior root's exact bytes
    /// and structure and the complete new metadata. Historical payload loss is allowed.
    ///
    /// # Errors
    /// Refuses malformed or inconsistent metadata, missing prior root, identity
    /// collisions, graph bounds or failed immutable publication/readback.
    pub fn finalize_availability(
        &mut self,
        value: &AvailabilityReceiptV1,
    ) -> Result<FinalizedAppend<AvailabilityReceiptV1>, Error> {
        let bytes = receipt::encode(value)?;
        let reference = self.reserve::<AvailabilityReceiptV1>(&bytes)?;
        let view = RootView {
            sink: &*self.sink,
            reference: reference.as_ref(),
            bytes: &bytes,
        };
        graph::AvailabilityAppend::resolve(&view, &reference)?;
        self.publish_reserved(&reference, &bytes)?;
        Ok(FinalizedAppend { reference, bytes })
    }

    /// Append a separate review of the exact prior receipt. Review identities and
    /// actions remain trusted caller observations, never authority from a hash.
    ///
    /// # Errors
    /// Refuses a changed review subject, incomplete reference closure, malformed
    /// records, identity collisions, bounds or failed publication/readback.
    pub fn finalize_review(
        &mut self,
        value: &ReviewReceiptV1,
    ) -> Result<FinalizedAppend<ReviewReceiptV1>, Error> {
        let bytes = receipt::encode(value)?;
        let reference = self.reserve::<ReviewReceiptV1>(&bytes)?;
        let view = RootView {
            sink: &*self.sink,
            reference: reference.as_ref(),
            bytes: &bytes,
        };
        graph::Graph::resolve(&view, reference.as_ref())?;
        self.publish_reserved(&reference, &bytes)?;
        Ok(FinalizedAppend { reference, bytes })
    }

    /// Publish a sealed typed non-envelope record; encode once and verify exact readback.
    /// # Errors
    /// Refuses root bypass, invalid/overbound records, ID collisions, sink errors or substitutions.
    pub fn record<T: ReceiptRecord>(&mut self, record: &T) -> Result<TypedRef<T>, Error> {
        if [
            ReceiptV1::SCHEMA_ID,
            ReviewReceiptV1::SCHEMA_ID,
            AvailabilityReceiptV1::SCHEMA_ID,
        ]
        .contains(&T::SCHEMA_ID)
        {
            return Err(Error::UnsupportedRoot);
        }
        let bytes = receipt::encode(record)?;
        let reference = self.reserve::<T>(&bytes)?;
        self.publish_reserved(&reference, &bytes)?;
        Ok(reference)
    }

    fn reserve<T: ReceiptRecord>(&mut self, bytes: &[u8]) -> Result<TypedRef<T>, Error> {
        let total = self
            .bytes
            .checked_add(u64::try_from(bytes.len()).map_err(|_| Error::Bound)?)
            .ok_or(Error::Bound)?;
        if self.attempted.len() >= MAX_OBJECTS || total > MAX_GRAPH_BYTES {
            return Err(Error::Bound);
        }
        let id = self.sink.fresh_id()?;
        if self.sink.contains_id(&id)? || !self.allocated.insert(id.as_str().to_owned()) {
            return Err(Error::Identity);
        }
        let reference = receipt::reference_for::<T>(id, bytes)?;
        self.attempted.push(reference.as_ref().clone());
        self.bytes = total;
        Ok(reference)
    }
    fn publish_reserved<T: ReceiptRecord>(
        &mut self,
        reference: &TypedRef<T>,
        bytes: &[u8],
    ) -> Result<(), Error> {
        let actual = self.sink.publish(reference.as_ref(), bytes)?;
        if &actual != reference.as_ref() {
            return Err(Error::Identity);
        }
        let mut observed = Vec::new();
        self.sink
            .open(reference.as_ref())?
            .take(u64::from(reference.as_ref().byte_length) + 1)
            .read_to_end(&mut observed)
            .map_err(|_| Error::Graph(graph::Error::Io))?;
        if observed != bytes {
            return Err(Error::Identity);
        }
        Ok(())
    }

    fn pages<R: Clone, P: ReceiptRecord>(
        &mut self,
        rows: &[R],
        build: impl Fn(u32, u32, u32, List<R>, Maybe<TypedRef<P>>) -> P,
    ) -> Result<TypedRef<P>, Error> {
        let page_count = rows.len().div_ceil(receipt::MAX_ITEMS).max(1);
        if page_count > MAX_OBJECTS {
            return Err(Error::Bound);
        }
        let count = u32::try_from(page_count).map_err(|_| Error::Bound)?;
        let total = u32::try_from(rows.len()).map_err(|_| Error::Bound)?;
        let mut next = Maybe::unavailable(Text::new("end_of_inventory")?);
        for index in (0..page_count).rev() {
            let start = (index * receipt::MAX_ITEMS).min(rows.len());
            let end = (start + receipt::MAX_ITEMS).min(rows.len());
            let page = build(
                u32::try_from(index).map_err(|_| Error::Bound)?,
                count,
                total,
                List::new(rows[start..end].to_vec())?,
                next,
            );
            next = Maybe::present(self.record(&page)?);
        }
        next.value.ok_or(Error::Bound)
    }

    /// Assemble exact plan bindings and recomputed inventories, then validate the
    /// complete temporary root graph before publishing those same encoded bytes.
    /// # Errors
    /// Refuses missing/duplicate observations, bounds, contradictory metadata,
    /// invalid evidence closure or sink failure. Staged objects remain retained.
    pub fn finalize(
        &mut self,
        prepared: &consistency::Prepared,
        observed: Observed,
    ) -> Result<Finalized, Error> {
        let case_rows = bind_cases(prepared, &observed.cases)?;
        let case_pages = self.case_pages(&case_rows)?;
        let cases = {
            let case_graph = graph::Graph::resolve(self.sink, case_pages.as_ref())?;
            case_counts(prepared, &case_rows, case_pages, &case_graph)?
        };
        let diagnostic_pages = self.diagnostic_pages(&observed.diagnostics)?;
        let diagnostics = diagnostic_counts(&observed, diagnostic_pages)?;
        let artifact_pages = self.artifact_pages(&observed.artifacts)?;
        let artifacts = artifact_counts(&observed, artifact_pages)?;
        let campaign_pages = self.campaign_pages(&observed.campaigns)?;
        let mutation = {
            let campaign_graph = graph::Graph::resolve(self.sink, campaign_pages.as_ref())?;
            mutation_counts(&observed.campaigns, campaign_pages, &campaign_graph)?
        };
        let root = ReceiptV1 {
            protocol: ReceiptV1Protocol::Hee3Receipt,
            version: 1,
            serialization: ReceiptV1Serialization::JsonExactV1,
            schema_sha256: prepared.schema_sha256.clone(),
            identity: prepared.identity.clone(),
            subjects: prepared.subjects.clone(),
            invocation: prepared.invocation.clone(),
            observations: observed.observations,
            cases,
            diagnostics,
            artifacts,
            mutation,
            review: Maybe::unavailable(Text::new("collection_not_reviewed")?),
            verdict: observed.verdict,
            availability: observed.availability,
        };
        let bytes = receipt::encode(&root)?;
        let reference = self.reserve::<ReceiptV1>(&bytes)?;
        let view = RootView {
            sink: &*self.sink,
            reference: reference.as_ref(),
            bytes: &bytes,
        };
        let graph = graph::Graph::resolve(&view, reference.as_ref())?;
        let summary = consistency::validate(&graph, &reference, prepared)?;
        self.publish_reserved(&reference, &bytes)?;
        Ok(Finalized {
            reference,
            bytes,
            summary,
        })
    }
}

struct RootView<'a, S> {
    sink: &'a S,
    reference: &'a Ref,
    bytes: &'a [u8],
}
impl<S: Sink> graph::Objects for RootView<'_, S> {
    fn open(&self, reference: &Ref) -> Result<Box<dyn Read + '_>, graph::Error> {
        if reference.artifact_id == self.reference.artifact_id {
            if reference != self.reference {
                return Err(graph::Error::Identity);
            }
            return Ok(Box::new(Cursor::new(self.bytes)));
        }
        self.sink.open(reference)
    }
}

fn bind_cases(
    prepared: &consistency::Prepared,
    observations: &[CaseObservation],
) -> Result<Vec<CaseV1>, Error> {
    if prepared.cases.is_empty()
        || prepared.cases.len() != observations.len()
        || prepared.cases.len() > MAX_OBJECTS * receipt::MAX_ITEMS
    {
        return Err(Error::CasePlan);
    }
    let observed: BTreeMap<&str, &CaseObservation> = observations
        .iter()
        .map(|c| (c.case_id.as_str(), c))
        .collect();
    let mut ids = BTreeSet::new();
    if observed.len() != observations.len() {
        return Err(Error::CasePlan);
    }
    prepared
        .cases
        .iter()
        .map(|plan| {
            if !ids.insert(plan.case_id.as_str()) {
                return Err(Error::CasePlan);
            }
            let row = observed.get(plan.case_id.as_str()).ok_or(Error::CasePlan)?;
            Ok(CaseV1 {
                case_id: plan.case_id.clone(),
                primary_module_id: plan.primary_module_id.clone(),
                criterion_ids: plan.criterion_ids.clone(),
                fixture_sha256: plan.fixture_sha256.clone(),
                oracle_id: plan.oracle_id.clone(),
                expected: plan.expected.clone(),
                mandatory: plan.mandatory,
                excluded: plan.excluded,
                selected: plan.selected,
                executed: row.executed,
                outcome: row.outcome,
                producer_exit_or_signal: row.producer.clone(),
                detector_id: row.detector_id.clone(),
                benign_pair_id: row.benign_pair_id.clone(),
                raw_evidence_refs: row.raw_evidence_refs.clone(),
                reason: row.reason.clone(),
            })
        })
        .collect()
}
fn case_counts(
    prepared: &consistency::Prepared,
    rows: &[CaseV1],
    inventory: TypedRef<CasePageV1>,
    graph: &graph::Graph,
) -> Result<CasesV1, Error> {
    let mut c = [0_u32; 12];
    let mut primary_credit = 0_u32;
    for (row, plan) in rows.iter().zip(&prepared.cases) {
        c[0] += 1;
        c[1] += u32::from(row.selected);
        c[2] += u32::from(row.executed);
        let index = if row.excluded {
            10
        } else {
            match row.outcome {
                CaseV1Outcome::Passed => 3,
                CaseV1Outcome::Failed => 4,
                CaseV1Outcome::Skipped => 5,
                CaseV1Outcome::Ignored => 6,
                CaseV1Outcome::Broken => 7,
                CaseV1Outcome::Timeout => 8,
                CaseV1Outcome::Invalid => 9,
                CaseV1Outcome::Unmeasured => 11,
            }
        };
        c[index] += 1;
        let reviewed = consistency::reviewed_case(graph, row, plan)?;
        if row.executed
            && row.outcome == CaseV1Outcome::Passed
            && row.primary_module_id == prepared.identity.module_id
            && reviewed
        {
            primary_credit += 1;
        }
    }
    Ok(CasesV1 {
        inventory,
        discovered: c[0],
        selected: c[1],
        executed: c[2],
        passed: c[3],
        failed: c[4],
        skipped: c[5],
        ignored: c[6],
        broken: c[7],
        timed_out: c[8],
        invalid: c[9],
        excluded: c[10],
        unmeasured: c[11],
        primary_credit,
    })
}
fn diagnostic_counts(
    observed: &Observed,
    by_tool: TypedRef<DiagnosticPageV1>,
) -> Result<DiagnosticsV1, Error> {
    let mut warning_count = 0_u32;
    let mut error_count = 0_u32;
    for row in &observed.diagnostics {
        if row.baseline != observed.diagnostic_baseline {
            return Err(consistency::Error::Diagnostic.into());
        }
        warning_count = warning_count
            .checked_add(row.warning_count)
            .ok_or(Error::Bound)?;
        error_count = error_count
            .checked_add(row.error_count)
            .ok_or(Error::Bound)?;
    }
    Ok(DiagnosticsV1 {
        baseline: observed.diagnostic_baseline,
        warning_count,
        error_count,
        by_tool,
        stdout_truncated: observed.diagnostics.iter().any(|r| r.stdout_truncated),
        stderr_truncated: observed.diagnostics.iter().any(|r| r.stderr_truncated),
        mismatch: observed.diagnostic_mismatch.clone(),
    })
}
fn artifact_counts(
    observed: &Observed,
    inventory: TypedRef<ArtifactPageV1>,
) -> Result<ArtifactsV1, Error> {
    let total = observed.artifacts.iter().try_fold(0_u64, |total, row| {
        total
            .checked_add(u64::from(row.object.byte_length))
            .ok_or(Error::Bound)
    })?;
    Ok(ArtifactsV1 {
        inventory,
        count: u32::try_from(observed.artifacts.len()).map_err(|_| Error::Bound)?,
        total_bytes: U64::new(total.to_string())?,
        finalized: observed.artifacts_finalized,
    })
}
fn mutation_counts(
    campaigns: &[CampaignV1],
    inventory: TypedRef<CampaignPageV1>,
    graph: &graph::Graph,
) -> Result<MutationV1, Error> {
    let mut ids = BTreeSet::new();
    let mut c = [0_u32; 8];
    for campaign in campaigns {
        let rows = graph.rows(campaign.mutants.as_ref())?;
        if rows.len() != campaign.planned_mutants as usize {
            return Err(consistency::Error::Mutation.into());
        }
        for row in rows {
            // Graph already checked each exact closed typed page. This is a typed
            // traversal view, not untyped input or a fallback schema interpretation.
            let mutant: MutantV1 = serde_json::from_value(row.clone())
                .map_err(|_| Error::Encoding(receipt::Error::Json))?;
            if mutant.campaign_id != campaign.campaign_id
                || !ids.insert(mutant.mutant_id.as_str().to_owned())
            {
                return Err(consistency::Error::Mutation.into());
            }
            c[0] = c[0].checked_add(1).ok_or(Error::Bound)?;
            let index = match mutant.outcome {
                MutantV1Outcome::Caught => 1,
                MutantV1Outcome::Survived => 2,
                MutantV1Outcome::Timeout => 3,
                MutantV1Outcome::Unviable => 4,
                MutantV1Outcome::ReviewedEquivalent => 5,
                MutantV1Outcome::Excluded => 6,
                MutantV1Outcome::Unmeasured => 7,
            };
            c[index] = c[index].checked_add(1).ok_or(Error::Bound)?;
        }
    }
    Ok(MutationV1 {
        campaigns: inventory,
        campaign_count: u32::try_from(campaigns.len()).map_err(|_| Error::Bound)?,
        distinct_mutants: c[0],
        caught: c[1],
        survived: c[2],
        timed_out: c[3],
        unviable: c[4],
        equivalent: c[5],
        excluded: c[6],
        unmeasured: c[7],
    })
}

// Explicit named page constructors over the fixed sealed DTO vocabulary.
impl<S: Sink> Publisher<'_, S> {
    /// Publish the complete ordered case inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn case_pages(&mut self, rows: &[CaseV1]) -> Result<TypedRef<CasePageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            CasePageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
    /// Publish the complete ordered diagnostic inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn diagnostic_pages(
        &mut self,
        rows: &[DiagnosticV1],
    ) -> Result<TypedRef<DiagnosticPageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            DiagnosticPageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
    /// Publish the complete ordered artifact inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn artifact_pages(
        &mut self,
        rows: &[ArtifactV1],
    ) -> Result<TypedRef<ArtifactPageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            ArtifactPageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
    /// Publish the complete ordered campaign inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn campaign_pages(
        &mut self,
        rows: &[CampaignV1],
    ) -> Result<TypedRef<CampaignPageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            CampaignPageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
    /// Publish the complete ordered mutant inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn mutant_pages(&mut self, rows: &[MutantV1]) -> Result<TypedRef<MutantPageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            MutantPageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
    /// Publish the complete ordered resource inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn resource_pages(
        &mut self,
        rows: &[ResourceV1],
    ) -> Result<TypedRef<ResourcePageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            ResourcePageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
    /// Publish the complete ordered obligation inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn obligation_pages(
        &mut self,
        rows: &[ObligationV1],
    ) -> Result<TypedRef<ObligationPageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            ObligationPageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
    /// Publish the complete ordered missing-object inventory, including an explicit empty page.
    /// # Errors
    /// Refuses invalid rows, page/graph profile bounds or immutable publication failure.
    pub fn missing_object_pages(
        &mut self,
        rows: &[MissingObjectV1],
    ) -> Result<TypedRef<MissingObjectPageV1>, Error> {
        self.pages(rows, |page_index, page_count, total_rows, rows, next| {
            MissingObjectPageV1 {
                page_index,
                page_count,
                total_rows,
                row_count: u32::try_from(rows.as_slice().len()).unwrap_or(u32::MAX),
                rows,
                next,
            }
        })
    }
}
