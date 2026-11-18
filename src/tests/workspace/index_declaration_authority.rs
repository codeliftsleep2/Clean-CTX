use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;

fn declaration(name: &str) -> SemanticEdge {
    let entity = EntityRef::new("example", "Thing", name);
    SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: entity.clone(),
        object: entity,
        layer: "test",
        call_evidence: None,
    }
}

fn use_edge(name: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Selects,
        subject: EntityRef::new("example", "Owner", "owner"),
        object: EntityRef::new("example", "Thing", name),
        layer: "test",
        call_evidence: None,
    }
}

#[test]
fn declaration_authority_is_separate_from_entity_occurrence() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("declaration.ts", vec![declaration("shared")]);
    index.add_edges("use.ts", vec![use_edge("used_only")]);

    assert_eq!(index.entities_in_file("declaration.ts").len(), 1);
    assert_eq!(index.entities_in_file("use.ts").len(), 2);
    assert_eq!(
        index.declarations_in_file("declaration.ts").len(),
        1,
        "self-Defines must survive normalization as declaration authority"
    );
    assert!(index.declarations_in_file("use.ts").is_empty());
    assert!(
        index
            .declarations_by_identity("example", "Thing", "used_only")
            .is_empty(),
        "relationship endpoints must not manufacture declaration authority"
    );
    assert_eq!(
        index.edge_count(),
        1,
        "self-Defines must remain absent from the graph"
    );
}

#[test]
fn declaration_authority_tracks_file_lifecycle_independently_of_use() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("declaration.ts", vec![declaration("shared")]);
    index.add_edges("use.ts", vec![use_edge("shared")]);
    assert_eq!(
        index
            .declarations_by_identity("example", "Thing", "shared")
            .len(),
        1
    );

    index.remove_file("declaration.ts");
    assert_eq!(index.entities_in_file("use.ts").len(), 2);
    assert!(
        index
            .declarations_by_identity("example", "Thing", "shared")
            .is_empty(),
        "use occurrence must survive without preserving stale authority"
    );

    index.add_edges("declaration.ts", vec![declaration("shared")]);
    index.remove_file("use.ts");
    assert_eq!(index.entities_in_file("declaration.ts").len(), 1);
    assert_eq!(index.declarations_in_file("declaration.ts").len(), 1);
    assert_eq!(index.edge_count(), 0);
}

#[test]
fn declaration_authority_preserves_source_distinct_occurrences() {
    let mut index = WorkspaceIndex::new();
    index.add_edges("a.ts", vec![declaration("shared")]);
    index.add_edges("b.ts", vec![declaration("shared")]);
    assert_eq!(
        index
            .declarations_by_identity("example", "Thing", "shared")
            .len(),
        2
    );
    assert_eq!(index.declarations_in_file("a.ts").len(), 1);
    assert_eq!(index.declarations_in_file("b.ts").len(), 1);

    index.remove_file("a.ts");
    let surviving = index.declarations_by_identity("example", "Thing", "shared");
    assert_eq!(surviving.len(), 1);
    assert_eq!(surviving[0].file.as_deref(), Some("b.ts"));
}
