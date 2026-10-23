use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;

#[test]
fn issue_162_entity_registration_keeps_referenced_targets_in_their_publisher_file() {
    let mut index = WorkspaceIndex::new();
    let leaf = EntityRef::new("angular", "Directive", "Leaf").with_file("leaf.ts".to_string());
    let module = EntityRef::new("angular", "Module", "AppModule").with_file("app.ts".to_string());
    index.add_edges(
        "app.ts",
        vec![SemanticEdge {
            subject: module,
            object: leaf.clone(),
            relation: SemanticRelation::DeclaresInModule,
            layer: "angular",
            call_evidence: None,
        }],
    );
    let declaration = SemanticEdge {
        subject: leaf.clone(),
        object: leaf,
        relation: SemanticRelation::Defines,
        layer: "angular",
        call_evidence: None,
    };
    index.add_edges("leaf.ts", vec![declaration.clone()]);
    assert_eq!(
        index.entities_in_file("leaf.ts").len(),
        1,
        "a prior target occurrence must not suppress its own publication"
    );
    index.remove_file("leaf.ts");
    index.add_edges("leaf.ts", vec![declaration]);
    assert_eq!(index.entities_in_file("leaf.ts").len(), 1);
    index.remove_file("app.ts");
    assert_eq!(
        index.entities_in_file("leaf.ts").len(),
        1,
        "retiring the referring owner must preserve the target's own registration"
    );
    assert_eq!(
        index
            .entities_by_identity("angular", "Directive", "Leaf")
            .len(),
        1,
        "publisher bookkeeping must not multiply entity occurrences"
    );
}
