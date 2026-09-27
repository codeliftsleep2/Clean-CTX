// Phase 1 RED contract for the approved dependency-cycle relation policy.
//
// This file deliberately exercises the existing boolean API first. The
// witness return type does not exist yet, and a compile failure is not valid
// RED evidence under the repository regression procedure.
#![allow(dead_code)]

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;

fn edge(
    relation: SemanticRelation,
    domain: &'static str,
    entity_type: &'static str,
    from: &str,
    to: &str,
) -> SemanticEdge {
    SemanticEdge {
        relation,
        subject: EntityRef::new(domain, entity_type, from),
        object: EntityRef::new(domain, entity_type, to),
        layer: domain,
        call_evidence: None,
    }
}

fn add_two_edge_cycle(
    index: &mut WorkspaceIndex,
    first_relation: SemanticRelation,
    second_relation: SemanticRelation,
) {
    index.add_edges(
        "cycle-a.ts",
        vec![edge(
            first_relation,
            "angular",
            "Service",
            "AlphaService",
            "BetaService",
        )],
    );
    index.add_edges(
        "cycle-b.ts",
        vec![edge(
            second_relation,
            "angular",
            "Service",
            "BetaService",
            "AlphaService",
        )],
    );
}

#[test]
fn dependency_cycle_accepts_injects() {
    let mut index = WorkspaceIndex::new();
    add_two_edge_cycle(
        &mut index,
        SemanticRelation::Injects,
        SemanticRelation::Injects,
    );

    assert!(
        index.has_cycle(),
        "Injects is an approved dependency relation"
    );
}

#[test]
fn dependency_cycle_accepts_module_imports() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "module-a.ts",
        vec![edge(
            SemanticRelation::ImportsModule,
            "angular",
            "Module",
            "AlphaModule",
            "BetaModule",
        )],
    );
    index.add_edges(
        "module-b.ts",
        vec![edge(
            SemanticRelation::ImportsModule,
            "angular",
            "Module",
            "BetaModule",
            "AlphaModule",
        )],
    );

    assert!(
        index.has_cycle(),
        "ImportsModule is an approved dependency relation"
    );
}

#[test]
fn dependency_cycle_rejects_autowired_until_its_identity_is_reliable() {
    let mut index = WorkspaceIndex::new();
    add_two_edge_cycle(
        &mut index,
        SemanticRelation::Autowired,
        SemanticRelation::Autowired,
    );

    assert!(
        !index.has_cycle(),
        "Autowired is excluded until its producer emits reliable target identity"
    );
}

#[test]
fn dependency_cycle_rejects_structural_metadata_loops() {
    let mut index = WorkspaceIndex::new();
    add_two_edge_cycle(
        &mut index,
        SemanticRelation::RouteMapsTo,
        SemanticRelation::DeclaresInModule,
    );

    assert!(
        !index.has_cycle(),
        "routing and containment metadata are not dependency-cycle edges"
    );
}

#[test]
fn dependency_cycle_rejects_a_mixed_path_with_one_excluded_edge() {
    let mut index = WorkspaceIndex::new();
    add_two_edge_cycle(
        &mut index,
        SemanticRelation::Injects,
        SemanticRelation::Tests,
    );

    assert!(
        !index.has_cycle(),
        "every edge in a dependency-cycle witness must be policy-approved"
    );
}

#[test]
fn dependency_cycle_witness_represents_a_self_loop_as_one_step() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "self.ts",
        vec![edge(
            SemanticRelation::Injects,
            "angular",
            "Service",
            "SelfService",
            "SelfService",
        )],
    );

    let witness = index
        .dependency_cycle_witness()
        .expect("an approved self-loop must produce a witness");
    assert_eq!(witness.len(), 1);
    assert_eq!(witness[0].subject, witness[0].object);
    assert_eq!(witness[0].asserting_file, "self.ts");
}

#[test]
fn dependency_cycle_witness_is_deterministic_when_two_cycles_exist() {
    let mut index = WorkspaceIndex::new();
    for (file, from, to) in [
        ("z-one.ts", "ZuluService", "YankeeService"),
        ("z-two.ts", "YankeeService", "ZuluService"),
        ("a-one.ts", "AlphaService", "BetaService"),
        ("a-two.ts", "BetaService", "AlphaService"),
    ] {
        index.add_edges(
            file,
            vec![edge(
                SemanticRelation::Injects,
                "angular",
                "Service",
                from,
                to,
            )],
        );
    }

    let first = index
        .dependency_cycle_witness()
        .expect("one cycle must be selected");
    assert_eq!(first[0].subject.name, "AlphaService");
    for _ in 0..8 {
        assert_eq!(index.dependency_cycle_witness().as_ref(), Some(&first));
    }
}

#[test]
fn dependency_cycle_witness_is_empty_for_an_acyclic_graph() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "acyclic.ts",
        vec![edge(
            SemanticRelation::Injects,
            "angular",
            "Service",
            "AlphaService",
            "BetaService",
        )],
    );

    assert!(index.dependency_cycle_witness().is_none());
}

#[test]
fn scoped_dependency_cycle_cannot_combine_two_workspaces_evidence() {
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        "C:/repo-a/alpha.ts",
        vec![edge(
            SemanticRelation::Injects,
            "angular",
            "Service",
            "AlphaService",
            "BetaService",
        )],
    );
    index.add_edges(
        "C:/repo-b/beta.ts",
        vec![edge(
            SemanticRelation::Injects,
            "angular",
            "Service",
            "BetaService",
            "AlphaService",
        )],
    );
    let scope_a = WorkspaceScope::new(Some("C:/repo-a"), &[]).expect("scope");
    let scope_b = WorkspaceScope::new(Some("C:/repo-b"), &[]).expect("scope");

    assert!(index.dependency_cycle_witness().is_some());
    assert!(index.dependency_cycle_witness_in_scope(&scope_a).is_none());
    assert!(index.dependency_cycle_witness_in_scope(&scope_b).is_none());
}
