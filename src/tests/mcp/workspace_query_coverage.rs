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
        candidates_discovered: 12,
        candidates_compiled: 4,
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
    assert_eq!(coverage["result_semantics"], "lower_bound");
    assert_eq!(coverage["omitted_possible"], true);
    assert_eq!(coverage["discovered_not_compiled_this_cycle"], 8);
}

#[test]
fn interface_reverse_coverage_points_to_csharp_constructor_type_query() {
    let mut index = WorkspaceIndex::new();
    let identity = ResolvedIdentity {
        domain: "builtin".into(),
        entity_type: "Interface".into(),
        name: "IFooService".into(),
    };
    let interface = EntityRef::new("builtin", "Interface", "IFooService")
        .with_file("IFooService.cs".to_string());
    index.add_edges(
        "IFooService.cs",
        vec![SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: interface.clone(),
            object: interface,
            layer: "builtin",
            call_evidence: None,
        }],
    );

    let hydration = HydrationReport {
        hydration_attempted: true,
        discovery_status: "partial",
        candidates_discovered: 14,
        ..HydrationReport::default()
    };
    let coverage = exact_identity_coverage(
        &index,
        &identity,
        None,
        CapabilityDirection::Reverse,
        &hydration,
    );
    assert_eq!(coverage["status"], "capability_not_established");
    assert_eq!(coverage["discovered_not_compiled_this_cycle"], 14);
    assert_eq!(coverage["alternative_query"]["type"], "reverse_edges");
    assert_eq!(coverage["alternative_query"]["domain"], "builtin");
    assert_eq!(coverage["alternative_query"]["entity_type"], "TypeRef");
    assert_eq!(coverage["alternative_query"]["name"], "IFooService");
    assert_eq!(
        coverage["alternative_query"]["relation"],
        "HasConstructorParameterType"
    );
}
