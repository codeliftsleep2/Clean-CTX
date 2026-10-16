use super::{BaseReference, Catalog, TypeShape, reference_candidates::ReferenceCandidateIndex};

fn reference(
    file: &str,
    owner: &str,
    namespace: &str,
    nesting: &[&str],
    written: &str,
) -> BaseReference {
    BaseReference {
        file: file.to_string(),
        owner: owner.to_string(),
        namespace: vec![namespace.to_string()],
        nesting: nesting.iter().map(|part| (*part).to_string()).collect(),
        written: written.to_string(),
        shape: TypeShape {
            qualifier: Vec::new(),
            name: written.to_string(),
            arity: 0,
        },
        imports: Vec::new(),
    }
}

#[test]
fn same_file_owner_bucket_preserves_all_candidates_and_original_order() {
    let references = vec![
        reference("same.cs", "Owner", "First", &[], "FirstBase"),
        reference("other.cs", "Owner", "Other", &[], "OtherBase"),
        reference("same.cs", "Owner", "Second", &["Nested"], "SecondBase"),
        reference("same.cs", "Different", "First", &[], "DifferentBase"),
    ];
    let catalog = Catalog {
        reference_candidates: ReferenceCandidateIndex::build(&references),
        references,
        ..Catalog::default()
    };

    let candidates: Vec<_> = catalog.references_for_owner("same.cs", "Owner").collect();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].written, "FirstBase");
    assert_eq!(candidates[0].namespace.join("."), "First");
    assert!(candidates[0].nesting.is_empty());
    assert_eq!(candidates[1].written, "SecondBase");
    assert_eq!(candidates[1].namespace.join("."), "Second");
    assert_eq!(candidates[1].nesting.join("."), "Nested");
    assert!(
        catalog
            .references_for_owner("missing.cs", "Owner")
            .next()
            .is_none()
    );
    assert!(
        catalog
            .references_for_owner("same.cs", "Missing")
            .next()
            .is_none()
    );
}
