use crate::mcp::acquisition::{
    AcquisitionRequirement, CleanProofRejection, CoverageUnit, InventoryEntry, InventoryManifest,
    SourceKind, clean_type_ref_proof_claim, has_certified_type_ref_name_bearing_predicate,
    type_ref_candidate_owners,
};
use std::sync::Arc;

fn entry(owner: &str, source: &str) -> InventoryEntry {
    InventoryEntry {
        owner: owner.into(),
        source_kind: SourceKind::CSharp,
        source_hash: crate::cache::content_hash(source.as_bytes()),
        source: Arc::new(source.into()),
    }
}

#[test]
fn type_ref_candidate_selection_uses_the_captured_inventory_snapshot() {
    let manifest = InventoryManifest::new(
        vec![
            entry("C:/repo/Plain.cs", "class Plain { ContextA value; }"),
            entry(
                "C:/repo/Qualified.cs",
                "class Qualified { Namespace.ContextA<string?>[] value; }",
            ),
            entry("C:/repo/Other.cs", "class Other { ContextB value; }"),
        ],
        true,
        true,
    );

    assert_eq!(
        type_ref_candidate_owners(&manifest, "ContextA"),
        vec!["C:/repo/Plain.cs", "C:/repo/Qualified.cs"]
    );
}

#[test]
fn type_ref_candidate_selection_preserves_unicode_and_authoritative_empty_sets() {
    let manifest = InventoryManifest::new(
        vec![entry(
            "C:/repo/Unicode.cs",
            "class Consumer { CaféContext context; }",
        )],
        true,
        true,
    );

    assert_eq!(
        type_ref_candidate_owners(&manifest, "CaféContext"),
        vec!["C:/repo/Unicode.cs"]
    );
    assert!(type_ref_candidate_owners(&manifest, "MissingContext").is_empty());
    assert!(type_ref_candidate_owners(&manifest, "").is_empty());
}

#[test]
fn only_the_tracked_csharp_producer_predicate_is_certified() {
    assert!(has_certified_type_ref_name_bearing_predicate(
        SourceKind::CSharp
    ));
    for uncertified in [SourceKind::TypeScript, SourceKind::Rust, SourceKind::Java] {
        assert!(!has_certified_type_ref_name_bearing_predicate(uncertified));
    }
}

#[cfg(feature = "csharp")]
#[test]
fn csharp_type_ref_production_edges_are_name_bearing_in_the_source_snapshot() {
    use crate::compression::Fidelity;
    use crate::layers::meta::semantic::SemanticRelation;

    let source = r#"
using Alias = Example.ContextA;

public class ContextA<T> {}
public interface IContextA {}

public sealed class Consumer : IContextA
{
    public Consumer(ContextA<string?> context, Alias alias) {}
}
"#;
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let (_, edges, _, _) = crate::mcp::tool_helpers::compile_source_ir_candidate(
        "C:/repo/Consumer.cs",
        source,
        Fidelity::High,
        &state,
    )
    .expect("C# production compilation");
    let type_refs: Vec<_> = edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.relation,
                SemanticRelation::HasConstructorParameterType | SemanticRelation::HasBaseType
            ) && edge.object.entity_type == "TypeRef"
        })
        .collect();

    assert!(
        !type_refs.is_empty(),
        "fixture must exercise TypeRef production"
    );
    for edge in type_refs {
        assert!(
            source.contains(&edge.object.name),
            "emitted TypeRef target must occur in its source snapshot: {:?}",
            edge.object.name
        );
    }
}

fn csharp_requirement() -> (AcquisitionRequirement, CoverageUnit) {
    let unit = CoverageUnit::builtin_type_ref("C:/repo", SourceKind::CSharp, "generation-1");
    (
        AcquisitionRequirement::incoming_type_ref("ContextA", "scope-1", [unit.clone()]),
        unit,
    )
}

#[test]
fn complete_csharp_inventory_constructs_a_provisional_clean_claim() {
    let (requirement, unit) = csharp_requirement();
    let manifest = InventoryManifest::new(
        vec![
            entry("C:/repo/Known.cs", "class Known { ContextA value; }"),
            entry("C:/repo/Other.cs", "class Other { ContextB value; }"),
        ],
        true,
        true,
    );

    let claim = clean_type_ref_proof_claim(&requirement, &unit, &manifest, "admission-1", 1, 1)
        .expect("certified complete C# inventory");
    assert_eq!(claim.inventory_generation, manifest.digest);
    assert_eq!(claim.candidate_owners, vec!["C:/repo/Known.cs"]);
    assert!(claim.exhaustive);
    assert!(!claim.truncated);
}

#[test]
fn clean_claim_rejects_incomplete_uncertified_or_mismatched_inventory() {
    let (requirement, unit) = csharp_requirement();
    let incomplete = InventoryManifest::new(
        vec![entry("C:/repo/Known.cs", "class Known { ContextA value; }")],
        true,
        false,
    );
    assert_eq!(
        clean_type_ref_proof_claim(&requirement, &unit, &incomplete, "admission-1", 1, 1),
        Err(CleanProofRejection::IncompleteInventory)
    );

    let typescript_unit =
        CoverageUnit::builtin_type_ref("C:/repo", SourceKind::TypeScript, "generation-1");
    assert_eq!(
        clean_type_ref_proof_claim(
            &requirement,
            &typescript_unit,
            &InventoryManifest::new(Vec::new(), true, true),
            "admission-1",
            1,
            1,
        ),
        Err(CleanProofRejection::UncertifiedPredicate)
    );

    let outside = InventoryManifest::new(
        vec![entry(
            "C:/other/Consumer.cs",
            "class Consumer { ContextA value; }",
        )],
        true,
        true,
    );
    assert_eq!(
        clean_type_ref_proof_claim(&requirement, &unit, &outside, "admission-1", 1, 1),
        Err(CleanProofRejection::OwnerOutsideRoot)
    );
}
