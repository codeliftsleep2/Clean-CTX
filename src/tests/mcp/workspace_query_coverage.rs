use super::{CapabilityDirection, exact_identity_coverage};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::tool_handlers::hydration::HydrationReport;
use crate::mcp::tool_handlers::query::identity::ResolvedIdentity;
use crate::workspace::index::WorkspaceIndex;

fn indexed_identity() -> (WorkspaceIndex, ResolvedIdentity) {
    let identity = ResolvedIdentity {
        domain: "builtin".into(),
        entity_type: "TypeRef".into(),
        name: "IFooService".into(),
    };
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "consumer.cs",
        vec![SemanticEdge {
            relation: SemanticRelation::HasConstructorParameterType,
            subject: EntityRef::new("builtin", "Class", "Consumer")
                .with_file("consumer.cs".to_string()),
            object: EntityRef::new("builtin", "TypeRef", "IFooService")
                .with_file("consumer.cs".to_string()),
            layer: "builtin",
            call_evidence: None,
        }],
    );
    (index, identity)
}

#[test]
fn workspace_query_exact_identity_coverage_partial_hydration() {
    let (index, identity) = indexed_identity();
    let hydration = HydrationReport {
        hydration_attempted: true,
        discovery_status: "partial",
        ..HydrationReport::default()
    };
    let coverage = exact_identity_coverage(
        &index,
        &identity,
        None,
        CapabilityDirection::Reverse,
        &hydration,
    );
    assert_eq!(coverage["status"], "indexed_evidence_only");
    assert_eq!(coverage["identity_indexed"], true);
    assert_eq!(coverage["capability_established"], true);
    assert_eq!(coverage["source_complete"], false);
}
