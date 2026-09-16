// src/tests/workspace/index_edge_lifecycle.rs
//
// Edge-occurrence lifecycle regressions (RED-E6..RED-E11).
//
// The identity/dedup/query half of the occurrence-aware edge contract lives
// in `index_edge_occurrence`; this file pins the lifecycle half: removing one
// source occurrence must never damage another, recompilation must stay
// file-local, the final graph must be compile-order independent, and the same
// rule must hold for non-Angular domains.
//
// Pre-fix (RED) observations, recorded so the regressions stay meaningful:
//   RED-E6: remove_file(A) also destroyed B's evidence (shared edge_set key)
//   RED-E7: recompiling A dropped B's overlapping edges
//   RED-E8/E9: surviving evidence depended on compile order (first wins)
//   RED-E11: the same collapse occurred in the dotnet-shaped edge set
//
// The `#[path]`-loaded module is compiled in both lib and test targets. In
// the lib target the `#[cfg(test)]` gate prevents this from causing
// dead-code warnings; clippy across all targets requires this allow.
#![allow(dead_code)]

use super::index_support::*;
use crate::layers::meta::semantic::SemanticEdge;
use crate::workspace::index::WorkspaceIndex;

// ── RED-E6 — removing one file preserves the other ────────────────────

#[test]
fn red_e6_removing_one_file_preserves_the_other_occurrence() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        FILE_A,
        vec![
            inject_edge("LoadingComponent", "Router", None),
            inject_edge("LoadingComponent", "SharedService", None),
            inject_edge("LoadingComponent", "UserStateService", None),
        ],
    );
    idx.add_edges(
        FILE_B,
        vec![
            inject_edge("LoadingComponent", "Router", None),
            inject_edge("LoadingComponent", "SharedService", None),
        ],
    );

    idx.remove_file(FILE_A);

    assert_eq!(idx.edge_count(), 2, "only A's edge occurrences are gone");
    assert_eq!(
        evidence(idx.forward_edges_by_identity("angular", "Component", "LoadingComponent")),
        vec![
            format!("{FILE_B}|LoadingComponent->Router"),
            format!("{FILE_B}|LoadingComponent->SharedService"),
        ],
        "B's overlapping evidence must survive A's removal"
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "Router")
            .len(),
        1,
        "B remains the only consumer of Router"
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "SharedService")
            .len(),
        1
    );
    assert!(idx.entities_in_file(FILE_A).is_empty());
    assert_eq!(idx.entities_in_file(FILE_B).len(), 3);
    assert!(
        idx.entities_by_identity("angular", "Service", "UserStateService")
            .is_empty(),
        "an entity occurring only in the removed file must not survive"
    );
    assert_eq!(
        idx.entities_by_identity("angular", "Component", "LoadingComponent")
            .len(),
        1,
        "B's occurrence of the shared identity must survive"
    );
}

// ── RED-E7 — recompiling one file preserves the other ─────────────────

#[test]
fn red_e7_recompiling_one_file_preserves_the_other_files_edges() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        FILE_A,
        vec![
            inject_edge("LoadingComponent", "Router", None),
            inject_edge("LoadingComponent", "SharedService", None),
        ],
    );
    idx.add_edges(
        FILE_B,
        vec![
            inject_edge("LoadingComponent", "Router", None),
            inject_edge("LoadingComponent", "SettingsStateService", None),
        ],
    );

    // Production recompile lifecycle: remove_file -> add_edges, same file.
    idx.remove_file(FILE_A);
    idx.add_edges(
        FILE_A,
        vec![inject_edge("LoadingComponent", "SharedService", None)],
    );

    assert_eq!(idx.edge_count(), 3);
    assert_eq!(
        evidence(idx.forward_edges_by_identity("angular", "Component", "LoadingComponent")),
        vec![
            format!("{FILE_A}|LoadingComponent->SharedService"),
            format!("{FILE_B}|LoadingComponent->Router"),
            format!("{FILE_B}|LoadingComponent->SettingsStateService"),
        ],
        "no cross-file collateral removal on recompilation"
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "Router")
            .len(),
        1,
        "A no longer injects Router; B still does"
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "SettingsStateService")
            .len(),
        1
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "SharedService")
            .len(),
        1
    );
}

// ── RED-E8 — three-project collision ──────────────────────────────────

#[test]
fn red_e8_three_project_collision_loses_nothing() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        FILE_A,
        vec![
            inject_edge("LoadingComponent", "Router", None),
            inject_edge("LoadingComponent", "SharedService", None),
            inject_edge("LoadingComponent", "UserStateService", None),
        ],
    );
    idx.add_edges(
        FILE_B,
        vec![
            inject_edge("LoadingComponent", "Router", None),
            inject_edge("LoadingComponent", "SharedService", None),
            inject_edge("LoadingComponent", "SettingsStateService", None),
        ],
    );
    idx.add_edges(
        FILE_C,
        vec![
            inject_edge("LoadingComponent", "Router", None),
            inject_edge("LoadingComponent", "ThemeService", None),
        ],
    );

    assert_eq!(idx.edge_count(), 8, "3 + 3 + 2 occurrences");
    assert_eq!(
        idx.forward_edges_by_identity("angular", "Component", "LoadingComponent")
            .len(),
        8
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "Router")
            .len(),
        3,
        "all three projects consume Router"
    );

    let mut asserting_files: Vec<String> = idx
        .forward_edges_by_identity("angular", "Component", "LoadingComponent")
        .iter()
        .filter_map(|edge| edge.subject.file.clone())
        .collect();
    asserting_files.sort();
    asserting_files.dedup();
    assert_eq!(
        asserting_files.len(),
        3,
        "no project may be lost, regardless of insertion order"
    );

    // Model C is untouched: one semantic identity per name, one occurrence
    // per file.
    assert_eq!(idx.entity_identity_count(), 6);
    assert_eq!(idx.entity_occurrence_count(), 11);
}

// ── RED-E9 — compile order independence ───────────────────────────────

#[test]
fn red_e9_compile_order_does_not_change_the_graph() {
    let entries: Vec<(String, Vec<SemanticEdge>)> = vec![
        (
            FILE_A.to_string(),
            vec![
                inject_edge("LoadingComponent", "Router", None),
                inject_edge("LoadingComponent", "SharedService", None),
            ],
        ),
        (
            FILE_B.to_string(),
            vec![
                inject_edge("LoadingComponent", "Router", None),
                inject_edge("LoadingComponent", "SettingsStateService", None),
            ],
        ),
        (
            FILE_C.to_string(),
            vec![
                inject_edge("LoadingComponent", "Router", None),
                inject_edge("LoadingComponent", "UserStateService", None),
            ],
        ),
    ];
    let mut reversed = entries.clone();
    reversed.reverse();

    let forward_order = index_in_order(&entries);
    let reverse_order = index_in_order(&reversed);

    assert_eq!(forward_order.edge_count(), 6);
    assert_eq!(
        forward_order.edge_count(),
        reverse_order.edge_count(),
        "first-one-wins / last-one-wins is forbidden"
    );
    assert_eq!(
        forward_order.entity_occurrence_count(),
        reverse_order.entity_occurrence_count()
    );
    assert_eq!(
        evidence(forward_order.forward_edges_by_identity(
            "angular",
            "Component",
            "LoadingComponent"
        )),
        evidence(reverse_order.forward_edges_by_identity(
            "angular",
            "Component",
            "LoadingComponent"
        )),
        "the final edge graph must be order-independent"
    );
    assert_eq!(
        evidence(forward_order.reverse_edges_by_identity("angular", "Service", "Router")),
        evidence(reverse_order.reverse_edges_by_identity("angular", "Service", "Router"))
    );
    assert_eq!(
        reverse_order
            .reverse_edges_by_identity("angular", "Service", "Router")
            .len(),
        3
    );
}

// ── RED-E10 — unrelated names retain current behaviour exactly ────────

#[test]
fn red_e10_unrelated_entities_are_unaffected() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![inject_edge("AlphaComponent", "AlphaService", None)],
    );
    idx.add_edges(
        "b.ts",
        vec![inject_edge("BetaComponent", "BetaService", None)],
    );
    idx.add_edges("c.ts", vec![route_edge("/alpha", "AlphaHome")]);

    assert_eq!(idx.edge_count(), 3);
    assert_eq!(
        idx.forward_edges_by_identity("angular", "Component", "AlphaComponent")
            .len(),
        1
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "AlphaService")
            .len(),
        1
    );
    assert_eq!(
        idx.entities_by_identity("angular", "Service", "BetaService")
            .len(),
        1
    );
    assert_eq!(idx.file_count(), 3);

    // Re-extracting the same occurrence must still dedup.
    idx.add_edges(
        "a.ts",
        vec![inject_edge("AlphaComponent", "AlphaService", None)],
    );
    assert_eq!(idx.total_edges_inserted(), 4);
    assert_eq!(idx.edge_count(), 3, "same-occurrence duplicates must dedup");
}

// ── RED-E11 — cross-domain: the defect was never Angular-specific ──────

#[test]
fn red_e11_cross_domain_same_name_controllers_are_distinct_occurrences() {
    const SERVICE_A: &str = "services-a/orders.controller.cs";
    const SERVICE_B: &str = "services-b/orders.controller.cs";

    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        SERVICE_A,
        vec![
            controller_action_edge("OrdersController", "GetAll"),
            has_route_edge("OrdersController", "api/orders"),
        ],
    );
    idx.add_edges(
        SERVICE_B,
        vec![
            controller_action_edge("OrdersController", "GetAll"),
            has_route_edge("OrdersController", "api/orders"),
        ],
    );

    assert_eq!(
        idx.edge_count(),
        4,
        "the dotnet-shaped edge set hits the same graph-level rule"
    );
    assert_eq!(
        evidence(idx.forward_edges_by_identity("dotnet", "Controller", "OrdersController")),
        vec![
            format!("{SERVICE_A}|OrdersController->GetAll"),
            format!("{SERVICE_A}|OrdersController->api/orders"),
            format!("{SERVICE_B}|OrdersController->GetAll"),
            format!("{SERVICE_B}|OrdersController->api/orders"),
        ]
    );
    assert_eq!(
        idx.reverse_edges_by_identity("dotnet", "Action", "GetAll")
            .len(),
        2,
        "both controllers are real consumers"
    );

    idx.remove_file(SERVICE_A);
    assert_eq!(
        idx.edge_count(),
        2,
        "removal remains file-local in every domain"
    );
    assert_eq!(
        evidence(idx.forward_edges_by_identity("dotnet", "Controller", "OrdersController")),
        vec![
            format!("{SERVICE_B}|OrdersController->GetAll"),
            format!("{SERVICE_B}|OrdersController->api/orders"),
        ],
        "the surviving controller keeps all of its evidence"
    );
}
