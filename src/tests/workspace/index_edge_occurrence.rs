// src/tests/workspace/index_edge_occurrence.rs
//
// Occurrence-aware edge identity regressions (RED-E1..RED-E5).
//
// These tests pin the core invariant of the generic WorkspaceIndex edge
// layer:
//
//   An edge asserted by one source occurrence must never suppress an
//   equivalent-looking edge asserted by a different source occurrence.
//
// The defect they guard against (confirmed live in a multi-project Angular
// workspace) was NOT language-specific: `EdgeKey` excluded the asserting
// source occurrence, so
//
//   project-a: LoadingComponent --Injects--> Router
//   project-b: LoadingComponent --Injects--> Router
//
// collapsed into ONE indexed edge, `file_edges[project-b]` never received
// the key, and removing/recompiling one file dropped the other file's
// evidence. The fix lives in `workspace::index` (edge occurrence identity),
// never in a semantic layer. Removal/recompilation, compile-order, and
// cross-domain regressions live in `index_edge_lifecycle`.
//
// Pre-fix (RED) observations, recorded so the regressions stay meaningful:
//   RED-E1: edge_count() == 1 (one shared key for two files)
//   RED-E3: only 3 of 6 occurrences survived; overlap was lost
//   RED-E4: forward_edges returned only file A's evidence
//   RED-E5: reverse_edges counted one consumer instead of two
//
// The `#[path]`-loaded module is compiled in both lib and test targets. In
// the lib target the `#[cfg(test)]` gate prevents this from causing
// dead-code warnings; clippy across all targets requires this allow.
#![allow(dead_code)]

use super::index_support::*;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;

/// A `HasSelector` edge: component exposes a CSS selector.
fn selector_edge(component: &str, selector: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", component),
        object: EntityRef::new("angular", "Component", selector),
        layer: "angular",
    }
}

// ── RED-E1 — same subject name, relation and object, different files ──

#[test]
fn red_e1_same_triple_from_two_files_is_two_occurrences() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        FILE_A,
        vec![inject_edge("LoadingComponent", "Router", None)],
    );
    idx.add_edges(
        FILE_B,
        vec![inject_edge("LoadingComponent", "Router", None)],
    );

    assert_eq!(
        idx.edge_count(),
        2,
        "one semantic triple asserted from two source occurrences is two records"
    );
    assert_eq!(
        idx.forward_edges_by_identity("angular", "Component", "LoadingComponent")
            .len(),
        2,
        "both files' outgoing evidence must survive"
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Service", "Router")
            .len(),
        2,
        "both files' incoming evidence must survive"
    );
    assert_eq!(idx.total_edges_inserted(), 2);
}

// ── RED-E2 — same subject name, unique objects (behaviour to protect) ──

#[test]
fn red_e2_unique_selector_objects_survive_for_both_files() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        FILE_A,
        vec![selector_edge("LoadingComponent", "app-a-loading")],
    );
    idx.add_edges(
        FILE_B,
        vec![selector_edge("LoadingComponent", "app-b-loading")],
    );

    let forward = idx.forward_edges_by_identity("angular", "Component", "LoadingComponent");
    assert_eq!(forward.len(), 2);
    let mut selectors: Vec<String> = forward
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::HasSelector)
        .map(|edge| edge.object.name.clone())
        .collect();
    selectors.sort();
    assert_eq!(
        selectors,
        vec!["app-a-loading".to_string(), "app-b-loading".to_string()]
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Component", "app-a-loading")
            .len(),
        1
    );
    assert_eq!(
        idx.reverse_edges_by_identity("angular", "Component", "app-b-loading")
            .len(),
        1
    );
}

// ── RED-E3 — overlapping + unique edges form a complete union ─────────

#[test]
fn red_e3_overlapping_and_unique_injections_are_a_complete_union() {
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

    assert_eq!(
        idx.edge_count(),
        6,
        "overlapping triples are distinct evidence, not duplicates"
    );
    assert_eq!(
        evidence(idx.forward_edges_by_identity("angular", "Component", "LoadingComponent")),
        vec![
            format!("{FILE_A}|LoadingComponent->Router"),
            format!("{FILE_A}|LoadingComponent->SharedService"),
            format!("{FILE_A}|LoadingComponent->UserStateService"),
            format!("{FILE_B}|LoadingComponent->Router"),
            format!("{FILE_B}|LoadingComponent->SettingsStateService"),
            format!("{FILE_B}|LoadingComponent->SharedService"),
        ],
        "every asserted occurrence must be present with its own provenance"
    );
}
// ── RED-E4 — forward_edges returns both occurrences' evidence ─────────

#[test]
fn red_e4_forward_edges_exposes_every_occurrence_with_provenance() {
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
        vec![inject_edge("LoadingComponent", "Router", None)],
    );

    let forward = idx.forward_edges_by_identity("angular", "Component", "LoadingComponent");
    assert_eq!(forward.len(), 3, "A:2 + B:1 overlapping occurrences");
    assert_eq!(
        evidence(forward),
        vec![
            format!("{FILE_A}|LoadingComponent->Router"),
            format!("{FILE_A}|LoadingComponent->SharedService"),
            format!("{FILE_B}|LoadingComponent->Router"),
        ],
        "no occurrence may be silently dropped at query time"
    );
}

// ── RED-E5 — reverse_edges preserves duplicate-name consumers ─────────

#[test]
fn red_e5_reverse_edges_preserves_duplicate_name_consumers() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        FILE_A,
        vec![inject_edge("LoadingComponent", "SharedService", None)],
    );
    idx.add_edges(
        FILE_B,
        vec![inject_edge("LoadingComponent", "SharedService", None)],
    );

    let incoming = idx.reverse_edges_by_identity("angular", "Service", "SharedService");
    assert_eq!(
        incoming.len(),
        2,
        "both real consumers must be counted, differentiated by occurrence"
    );
    assert_eq!(
        evidence(incoming),
        vec![
            format!("{FILE_A}|LoadingComponent->SharedService"),
            format!("{FILE_B}|LoadingComponent->SharedService"),
        ]
    );
    // Semantic identity stays Model C: one identity per name, one occurrence
    // per file.
    assert_eq!(
        idx.entity_identity_count(),
        2,
        "LoadingComponent + SharedService identities only"
    );
    assert_eq!(
        idx.entity_occurrence_count(),
        4,
        "two occurrences per identity (one per file)"
    );
}
