// src/tests/workspace/index.rs
//
// Tests for WorkspaceIndex core (Phase 4a).
//
// These tests prove the architectural contracts of the workspace index:
// entity registration, edge deduplication, file provenance, forward/reverse
// indexing, deterministic results, and entity ambiguity.

// The `#[path]`-loaded module is compiled in both lib and test targets.
// In the lib target the `#[cfg(test)]` gate prevents this from causing
// dead-code warnings; clippy across all targets requires this allow.
#![allow(dead_code)]

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;

/// Helper: build a simple Inject edge.
fn inject_edge(component: &str, service: &str, file: Option<&str>) -> SemanticEdge {
    let mut subj = EntityRef::new("angular", "Component", component);
    let mut obj = EntityRef::new("angular", "Service", service);
    if let Some(f) = file {
        subj.file = Some(f.to_string());
        obj.file = Some(f.to_string());
    }
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: subj,
        object: obj,
        layer: "angular",
    }
}

/// Helper: build a RouteMapsTo edge.
fn route_edge(path: &str, component: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::RouteMapsTo,
        subject: EntityRef::new("angular", "Route", path),
        object: EntityRef::new("angular", "Component", component),
        layer: "angular",
    }
}

// ── Empty Index ──────────────────────────────────────────────────────

#[test]
fn empty_index_constructs() {
    let idx = WorkspaceIndex::new();
    assert!(idx.is_empty());
    assert_eq!(idx.edge_count(), 0);
    assert_eq!(idx.entity_identity_count(), 0);
    assert_eq!(idx.entity_occurrence_count(), 0);
    assert_eq!(idx.file_count(), 0);
}

#[test]
fn empty_index_queries_return_empty() {
    let idx = WorkspaceIndex::new();
    assert!(
        idx.entities_by_identity("angular", "Component", "Foo")
            .is_empty()
    );
    assert!(
        idx.forward_edges_by_identity("angular", "Component", "Foo")
            .is_empty()
    );
    assert!(
        idx.reverse_edges_by_identity("angular", "Component", "Foo")
            .is_empty()
    );
    assert!(idx.entities_in_file("foo.ts").is_empty());
}

// ── Insertion and Retrieval ──────────────────────────────────────────

#[test]
fn insert_single_edge_registers_entities() {
    let mut idx = WorkspaceIndex::new();
    let edge = inject_edge("UserComponent", "UserService", Some("app.ts"));
    idx.add_edges("app.ts", vec![edge]);

    assert_eq!(idx.edge_count(), 1);
    assert_eq!(idx.entity_identity_count(), 2);
    assert_eq!(idx.entity_occurrence_count(), 2);
    assert_eq!(idx.file_count(), 1);
}

#[test]
fn forward_edges_retrieved_by_identity() {
    let mut idx = WorkspaceIndex::new();
    let edge = inject_edge("UserComponent", "UserService", Some("app.ts"));
    idx.add_edges("app.ts", vec![edge]);

    let outgoing = idx.forward_edges_by_identity("angular", "Component", "UserComponent");
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].relation, SemanticRelation::Injects);

    let svc_out = idx.forward_edges_by_identity("angular", "Service", "UserService");
    assert!(svc_out.is_empty());
}

#[test]
fn reverse_edges_retrieved_by_identity() {
    let mut idx = WorkspaceIndex::new();
    let edge = inject_edge("UserComponent", "UserService", Some("app.ts"));
    idx.add_edges("app.ts", vec![edge]);

    let incoming = idx.reverse_edges_by_identity("angular", "Service", "UserService");
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].relation, SemanticRelation::Injects);

    let cmp_in = idx.reverse_edges_by_identity("angular", "Component", "UserComponent");
    assert!(cmp_in.is_empty());
}

#[test]
fn entities_in_file_returns_correct_entities() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("app.ts", vec![inject_edge("A", "B", Some("app.ts"))]);
    idx.add_edges("other.ts", vec![route_edge("home", "HomeComponent")]);

    let app_entities = idx.entities_in_file("app.ts");
    assert_eq!(app_entities.len(), 2);

    let other_entities = idx.entities_in_file("other.ts");
    assert_eq!(other_entities.len(), 2);
}
fn same_edge_inserted_twice_is_indexed_once() {
    let mut idx = WorkspaceIndex::new();
    let edge = inject_edge("UserComponent", "UserService", Some("app.ts"));

    idx.add_edges("app.ts", vec![edge.clone()]);
    assert_eq!(idx.edge_count(), 1);
    assert_eq!(idx.total_edges_inserted(), 1);

    // Insert the same edge again (duplicate content + duplicate identity).
    idx.add_edges("app.ts", vec![edge.clone()]);
    assert_eq!(
        idx.edge_count(),
        1,
        "dedup must prevent second identical edge"
    );
    assert_eq!(
        idx.total_edges_inserted(),
        2,
        "total_edges_inserted counts raw insertions"
    );
}

#[test]
fn edge_dedup_ignores_file_provenance() {
    let mut idx = WorkspaceIndex::new();

    let mut subj_a = EntityRef::new("angular", "Component", "UserComponent");
    subj_a.file = Some("a.ts".to_string());
    let mut obj_a = EntityRef::new("angular", "Service", "UserService");
    obj_a.file = Some("a.ts".to_string());
    let edge_a = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: subj_a,
        object: obj_a,
        layer: "angular",
    };

    let mut subj_b = EntityRef::new("angular", "Component", "UserComponent");
    subj_b.file = Some("b.ts".to_string());
    let mut obj_b = EntityRef::new("angular", "Service", "UserService");
    obj_b.file = Some("b.ts".to_string());
    let edge_b = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: subj_b,
        object: obj_b,
        layer: "angular",
    };

    idx.add_edges("a.ts", vec![edge_a]);
    idx.add_edges("b.ts", vec![edge_b]);

    // One deduped edge because identity ignores file.
    assert_eq!(idx.edge_count(), 1);
    // Entity occurrences retained from BOTH files (architectural review fix).
    // Before the fix, entity registration was inside the dedup gate and
    // only file A's entities were registered. Now entity registration
    // happens before edge dedup, so both files' occurrences are tracked.
    assert_eq!(
        idx.entity_occurrence_count(),
        4,
        "both files' entity occurrences must be retained"
    );
    assert_eq!(
        idx.entities_in_file("b.ts").len(),
        2,
        "entities from file B must be discoverable via entities_in_file"
    );
}

// ── Entity Ambiguity ─────────────────────────────────────────────────

#[test]
fn same_entity_identity_in_multiple_files_retains_all() {
    let mut idx = WorkspaceIndex::new();

    idx.add_edges(
        "a.ts",
        vec![inject_edge("UserComponent", "UserService", Some("a.ts"))],
    );
    idx.add_edges(
        "b.ts",
        vec![inject_edge("AdminComponent", "UserService", Some("b.ts"))],
    );

    // UserService entity identity appears in both files
    let entities = idx.entities_by_identity("angular", "Service", "UserService");
    assert_eq!(
        entities.len(),
        2,
        "both occurrences of UserService must be retained"
    );

    let files: Vec<Option<&String>> = entities.iter().map(|e| e.file.as_ref()).collect();
    assert!(files.contains(&Some(&"a.ts".to_string())));
    assert!(files.contains(&Some(&"b.ts".to_string())));
}

// ── File Provenance ──────────────────────────────────────────────────

#[test]
fn file_provenance_attached_when_missing() {
    let mut idx = WorkspaceIndex::new();

    // Edge without file attached (as produced by Phase 1 extraction).
    let edge = inject_edge("UserComponent", "UserService", None);
    idx.add_edges("app.ts", vec![edge]);

    let entities = idx.entities_by_identity("angular", "Component", "UserComponent");
    assert!(!entities.is_empty());
    for entity in &entities {
        assert_eq!(
            entity.file.as_deref(),
            Some("app.ts"),
            "file provenance should be attached"
        );
    }
}

#[test]
fn multiple_files_increase_file_count() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![inject_edge("A", "S", Some("a.ts"))]);
    idx.add_edges("b.ts", vec![inject_edge("B", "S", Some("b.ts"))]);
    idx.add_edges("c.ts", vec![route_edge("/", "Root")]);

    assert_eq!(idx.file_count(), 3);
}

// ── Determinism & Counters ───────────────────────────────────────────

#[test]
fn deterministic_queries_given_same_input() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![
            inject_edge("A", "S", Some("a.ts")),
            route_edge("/home", "HomeComponent"),
            route_edge("/users", "UsersComponent"),
        ],
    );

    let first = idx.forward_edges_by_identity("angular", "Route", "/home");
    let second = idx.forward_edges_by_identity("angular", "Route", "/home");
    assert_eq!(first.len(), second.len());
}

#[test]
fn counters_reflect_insertion_and_dedup() {
    let mut idx = WorkspaceIndex::new();

    // 3 distinct edges
    idx.add_edges(
        "a.ts",
        vec![
            inject_edge("A", "S", Some("a.ts")),
            route_edge("/", "Root"),
            route_edge("/home", "HomeComponent"),
        ],
    );
    assert_eq!(idx.total_edges_inserted(), 3);
    assert_eq!(idx.edge_count(), 3);

    // 2 duplicates
    idx.add_edges(
        "b.ts",
        vec![
            inject_edge("A", "S", Some("b.ts")),
            route_edge("/home", "HomeComponent"),
        ],
    );
    assert_eq!(idx.total_edges_inserted(), 5);
    assert_eq!(idx.edge_count(), 3, "dedup must keep count at 3");

    // 1 new edge
    idx.add_edges("b.ts", vec![route_edge("/admin", "Admin")]);
    assert_eq!(idx.total_edges_inserted(), 6);
    assert_eq!(idx.edge_count(), 4);
}
// ── Phase 4b: Name-based Lookup ───────────────────────────────────────

#[test]
fn find_entities_by_name_empty_when_not_found() {
    let idx = WorkspaceIndex::new();
    assert!(idx.find_entities_by_name("NonExistent").is_empty());
}

#[test]
fn find_entities_by_name_returns_across_domains_and_types() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "app.ts",
        vec![inject_edge("UserComponent", "UserService", Some("app.ts"))],
    );
    idx.add_edges("app.ts", vec![route_edge("/users", "UsersComponent")]);

    let found = idx.find_entities_by_name("UserComponent");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].domain, "angular");
    assert_eq!(found[0].entity_type, "Component");
}

#[test]
fn find_entities_by_name_retains_ambiguity() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![inject_edge("UserComponent", "UserService", Some("a.ts"))],
    );
    idx.add_edges(
        "b.ts",
        vec![inject_edge("AdminComponent", "UserService", Some("b.ts"))],
    );

    // UserService has same name but comes from two different entity identities
    // (both are ("angular", "Service", "UserService") so same identity, but
    // two occurrences at different files).
    let found = idx.find_entities_by_name("UserService");
    assert_eq!(found.len(), 2, "both occurrences must be found");
}

// ── Phase 4b: resolve_inject_type ────────────────────────────────────

#[test]
fn resolve_inject_type_empty_when_not_found() {
    let idx = WorkspaceIndex::new();
    assert!(idx.resolve_inject_type("NonExistent").is_empty());
}

#[test]
fn resolve_inject_type_returns_injection_targets() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "app.ts",
        vec![inject_edge("UserComponent", "UserService", Some("app.ts"))],
    );

    let targets = idx.resolve_inject_type("UserService");
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].name, "UserService");
    assert_eq!(targets[0].entity_type, "Service");
    assert_eq!(targets[0].file.as_deref(), Some("app.ts"));
}

#[test]
fn resolve_inject_type_ignores_non_injected_names() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "app.ts",
        vec![
            inject_edge("UserComponent", "UserService", Some("app.ts")),
            route_edge("/home", "HomeComponent"),
        ],
    );

    // "HomeComponent" appears as an object of a RouteMapsTo edge, NOT Injects.
    let targets = idx.resolve_inject_type("HomeComponent");
    assert!(targets.is_empty(), "non-injected entities must be excluded");
}

#[test]
fn resolve_inject_type_returns_multiple_occurrences() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![inject_edge("ComponentA", "SharedService", Some("a.ts"))],
    );
    idx.add_edges(
        "b.ts",
        vec![inject_edge("ComponentB", "SharedService", Some("b.ts"))],
    );

    let targets = idx.resolve_inject_type("SharedService");
    assert_eq!(targets.len(), 2, "both injection reference occurrences");
}
#[test]
fn resolve_inject_type_accepts_spring_autowired() {
    let mut idx = WorkspaceIndex::new();
    let autowired_edge = SemanticEdge {
        relation: SemanticRelation::Autowired,
        subject: EntityRef::new("spring", "Controller", "UserController"),
        object: EntityRef {
            domain: "spring",
            entity_type: "Service",
            name: "UserService".to_string(),
            file: Some("UserController.java".to_string()),
        },
        layer: "spring",
    };
    idx.add_edges("UserController.java", vec![autowired_edge]);

    let targets = idx.resolve_inject_type("UserService");
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].domain, "spring");
    assert_eq!(targets[0].file.as_deref(), Some("UserController.java"));
}

#[test]
fn resolve_inject_type_angular_component_has_no_reverse_inject() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "app.ts",
        vec![inject_edge("UserComponent", "UserService", Some("app.ts"))],
    );

    // "UserComponent" is the SUBJECT of Injects, not the object.
    let targets = idx.resolve_inject_type("UserComponent");
    assert!(
        targets.is_empty(),
        "subjects of injection edges are not injection targets"
    );
}

// ── Phase 4b: resolve_selector ───────────────────────────────────────

#[test]
fn resolve_selector_empty_when_not_found() {
    let idx = WorkspaceIndex::new();
    assert!(idx.resolve_selector("app-not-found").is_empty());
}

#[test]
fn resolve_selector_returns_component_with_matching_selector() {
    let mut idx = WorkspaceIndex::new();

    let edge = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "UserCardComponent"),
        object: EntityRef::new("angular", "Component", "[app-user-card]"),
        layer: "angular",
    };
    idx.add_edges("user-card.ts", vec![edge]);

    let results = idx.resolve_selector("app-user-card");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "UserCardComponent");
    assert_eq!(results[0].entity_type, "Component");
}

#[test]
fn resolve_selector_unknown_selector_returns_empty() {
    let mut idx = WorkspaceIndex::new();

    let edge = SemanticEdge {
        relation: SemanticRelation::RouteMapsTo,
        subject: EntityRef::new("angular", "Route", "/home"),
        object: EntityRef::new("angular", "Component", "HomeComponent"),
        layer: "angular",
    };
    idx.add_edges("routes.ts", vec![edge]);

    let results = idx.resolve_selector("home-selector");
    assert!(results.is_empty(), "RouteMapsTo must not match HasSelector");
}

#[test]
fn resolve_selector_file_provenance_preserved() {
    let mut idx = WorkspaceIndex::new();

    let edge = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef {
            domain: "angular",
            entity_type: "Component",
            name: "HeaderComponent".to_string(),
            file: Some("header.component.ts".to_string()),
        },
        object: EntityRef {
            domain: "angular",
            entity_type: "Component",
            name: "[app-header]".to_string(),
            file: None,
        },
        layer: "angular",
    };
    idx.add_edges("header.component.ts", vec![edge]);

    let results = idx.resolve_selector("app-header");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].file.as_deref(), Some("header.component.ts"));
}

// ── Phase 4b: Phase 4a regression ───────────────────────────────────

#[test]
fn phase_4a_empty_index_constructs_unchanged() {
    let idx = WorkspaceIndex::new();
    assert!(idx.is_empty());
    assert_eq!(idx.edge_count(), 0);
    assert_eq!(idx.entity_identity_count(), 0);
    assert_eq!(idx.entity_occurrence_count(), 0);
    assert_eq!(idx.file_count(), 0);
}

#[test]
fn phase_4a_identity_queries_unchanged() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "app.ts",
        vec![inject_edge("UserComponent", "UserService", Some("app.ts"))],
    );

    let by_id = idx.entities_by_identity("angular", "Component", "UserComponent");
    assert_eq!(by_id.len(), 1);

    let forward = idx.forward_edges_by_identity("angular", "Component", "UserComponent");
    assert_eq!(forward.len(), 1);

    let reverse = idx.reverse_edges_by_identity("angular", "Service", "UserService");
    assert_eq!(reverse.len(), 1);
}
