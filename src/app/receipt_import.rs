//! Transfer an exact collector receipt closure into the coordinator's object owner.
//! This checks representation and custody, never observation truth or task acceptance.

use crate::app::evidence::Evidence;
use crate::check::{
    collector::{Sink, SinkError},
    consistency::{self, Prepared, Summary},
    graph::{self, Graph, Objects},
};
use crate::contracts::receipt::{ReceiptV1, TypedRef};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Graph(graph::Error),
    Consistency(consistency::Error),
    Publication(SinkError),
    Conflict,
}

/// Copy only the validated root's reachable objects, preserving all five Ref fields.
/// Existing exact references are read back during final validation; conflicting or
/// pending IDs refuse before publication. The root is published after its children.
/// Source bytes come from the bounded resolved graph.
/// The caller retains `destination`, including partial/pending publications, on error.
///
/// The source must belong to the trusted collector, and `prepared` must be the
/// coordinator's pre-execution value. Consistent untrusted metadata is not evidence
/// that any process ran. No ledger transition or verdict reinterpretation occurs.
///
/// # Errors
/// Refuses incomplete/corrupt graphs, changed preparation, conflicting identity,
/// publication/readback failure, or either owner's expired deadline.
pub fn receipt(
    destination: &mut Evidence<'_>,
    source: &Evidence<'_>,
    prepared: &Prepared,
    root: &TypedRef<ReceiptV1>,
) -> Result<Summary, Error> {
    let graph = Graph::resolve(source, root.as_ref()).map_err(Error::Graph)?;
    consistency::validate(&graph, root, prepared).map_err(Error::Consistency)?;
    let mut closure = Vec::with_capacity(graph.object_count());
    for (reference, _) in source.registered().values() {
        match graph.get(reference) {
            Ok(node) => {
                if destination
                    .registered()
                    .get(reference.artifact_id.as_str())
                    .is_some_and(|(existing, _)| existing != reference)
                    || destination
                        .pending_publications()
                        .iter()
                        .any(|(pending, _)| pending.artifact_id == reference.artifact_id)
                {
                    return Err(Error::Conflict);
                }
                if destination
                    .registered()
                    .contains_key(reference.artifact_id.as_str())
                {
                    // Evidence::open verifies all stored bytes and custody before
                    // returning its owned in-memory reader. Do this before writes.
                    drop(destination.open(reference).map_err(Error::Graph)?);
                }
                closure.push(node);
            }
            Err(graph::Error::Missing) => {}
            Err(error) => return Err(Error::Graph(error)),
        }
    }
    if closure.len() != graph.object_count() {
        return Err(Error::Graph(graph::Error::Missing));
    }
    closure.sort_by_key(|node| node.reference() == root.as_ref());
    for node in closure {
        if !destination
            .registered()
            .contains_key(node.reference().artifact_id.as_str())
        {
            destination
                .publish(node.reference(), node.bytes())
                .map_err(Error::Publication)?;
        }
    }
    let imported = Graph::resolve(destination, root.as_ref()).map_err(Error::Graph)?;
    consistency::validate(&imported, root, prepared).map_err(Error::Consistency)
}
