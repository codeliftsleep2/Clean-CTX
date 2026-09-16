// src/tests/workspace/index_graph.rs
//
// WorkspaceIndex graph tests: cycle detection, transitive dependency
// traversal, the registration-record write boundary, and the dependency
// relation boundary.
//
// The `#[path]`-loaded module is compiled in both lib and test targets. In
// the lib target the `#[cfg(test)]` gate prevents this from causing
// dead-code warnings; clippy across all targets requires this allow.
#![allow(dead_code)]

use super::index_support::*;
use crate::layers::meta::semantic::*;
use crate::workspace::index::WorkspaceIndex;

// ── Phase 4c: Graph Traversal — has_cycle ────────────────────────────

#[test]
fn has_cycle_empty_index() {
    let idx = WorkspaceIndex::new();
    assert!(!idx.has_cycle());
}

#[test]
fn has_cycle_single_entity() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![inject_edge("A", "B", Some("a.ts"))]);
    assert!(!idx.has_cycle());
}

#[test]
fn has_cycle_self_loop() {
    let mut idx = WorkspaceIndex::new();
    let self_loop = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Component", "A"),
        object: EntityRef::new("angular", "Component", "A"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![self_loop]);
    assert!(idx.has_cycle(), "self-loop must be detected as a cycle");
}

#[test]
fn has_cycle_simple_cycle() {
    let mut idx = WorkspaceIndex::new();
    // Use same domain/entity_type so entity keys chain: A → B → A.
    let edge1 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "A"),
        object: EntityRef::new("angular", "Service", "B"),
        layer: "angular",
        call_evidence: None,
    };
    let edge2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "A"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![edge1, edge2]);
    assert!(idx.has_cycle(), "A → B → A must be detected as a cycle");
}

#[test]
fn has_cycle_no_cycle() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![
            inject_edge("A", "B", Some("a.ts")),
            inject_edge("B", "C", Some("a.ts")),
        ],
    );
    assert!(!idx.has_cycle(), "A → B → C has no cycle");
}

#[test]
fn has_cycle_structural_cycle() {
    let mut idx = WorkspaceIndex::new();
    // Use RouteMapsTo and DeclaresInModule to form a cycle through
    // structural relations — all relations are traversed for cycle detection.
    let edge1 = SemanticEdge {
        relation: SemanticRelation::RouteMapsTo,
        subject: EntityRef::new("angular", "Route", "/home"),
        object: EntityRef::new("angular", "Component", "HomeComponent"),
        layer: "angular",
        call_evidence: None,
    };
    let edge2 = SemanticEdge {
        relation: SemanticRelation::DeclaresInModule,
        subject: EntityRef::new("angular", "Module", "AppModule"),
        object: EntityRef::new("angular", "Route", "/home"),
        layer: "angular",
        call_evidence: None,
    };
    let edge3 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Component", "HomeComponent"),
        object: EntityRef::new("angular", "Module", "AppModule"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![edge1, edge2, edge3]);
    assert!(
        idx.has_cycle(),
        "cycle through mixed relation types must be detected"
    );
}

#[test]
fn has_cycle_ambiguous_entity() {
    let mut idx = WorkspaceIndex::new();
    // Same entity identity in two files, single edge B → A (no cycle).
    idx.add_edges("a.ts", vec![inject_edge("B", "A", Some("a.ts"))]);
    idx.add_edges("b.ts", vec![inject_edge("B", "A", Some("b.ts"))]);
    assert!(!idx.has_cycle());
}

// ── Registration records (B1 write boundary) ─────────────────────────

#[test]
fn self_defines_edge_is_registration_record_only() {
    let mut idx = WorkspaceIndex::new();
    // Self-referential `Defines` — the shape BuiltinMetaLayer emits to
    // register an ordinary declaration. It is a registration carrier, not a
    // relationship: register once, never index as an edge.
    idx.add_edges(
        "a.ts",
        vec![SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: EntityRef::new("builtin", "Class", "UserService"),
            object: EntityRef::new("builtin", "Class", "UserService"),
            layer: "builtin",
            call_evidence: None,
        }],
    );

    assert_eq!(
        idx.entities_by_identity("builtin", "Class", "UserService")
            .len(),
        1,
        "registration record must register exactly one occurrence"
    );
    assert_eq!(
        idx.find_entities_by_name("UserService").len(),
        1,
        "name lookup must see exactly one occurrence"
    );
    assert_eq!(
        idx.edge_count(),
        0,
        "registration record must not become a graph edge"
    );
    assert!(
        idx.forward_edges_by_identity("builtin", "Class", "UserService")
            .is_empty(),
        "registration record must not appear in the forward index"
    );
    assert!(
        idx.reverse_edges_by_identity("builtin", "Class", "UserService")
            .is_empty(),
        "registration record must not appear in the reverse index"
    );
    assert_eq!(
        idx.entities_in_file("a.ts").len(),
        1,
        "file bookkeeping must contain exactly one occurrence"
    );
    assert!(
        !idx.has_cycle(),
        "registration record must not create a cycle"
    );
}

#[test]
fn self_defines_recompile_does_not_accumulate() {
    let mut idx = WorkspaceIndex::new();
    let registration = || SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: EntityRef::new("builtin", "Class", "Foo"),
        object: EntityRef::new("builtin", "Class", "Foo"),
        layer: "builtin",
        call_evidence: None,
    };

    idx.add_edges("a.ts", vec![registration()]);
    assert_eq!(idx.entities_by_identity("builtin", "Class", "Foo").len(), 1);
    assert_eq!(idx.edge_count(), 0);

    // Production recompile lifecycle: remove_file → add_edges, repeatedly.
    idx.remove_file("a.ts");
    idx.add_edges("a.ts", vec![registration()]);
    idx.remove_file("a.ts");
    idx.add_edges("a.ts", vec![registration()]);

    assert_eq!(
        idx.entities_by_identity("builtin", "Class", "Foo").len(),
        1,
        "recompilation must not accumulate registration occurrences"
    );
    assert_eq!(
        idx.edge_count(),
        0,
        "registration records must never accumulate as graph edges"
    );
    assert!(
        idx.forward_edges_by_identity("builtin", "Class", "Foo")
            .is_empty()
    );
    assert!(
        idx.reverse_edges_by_identity("builtin", "Class", "Foo")
            .is_empty()
    );
    assert_eq!(idx.entities_in_file("a.ts").len(), 1);
    assert!(!idx.has_cycle());
}

#[test]
fn framework_defines_edges_still_traversed() {
    let mut idx = WorkspaceIndex::new();
    // Non-self `Defines(A, B)` — the shape framework layers emit (e.g.
    // angular routing Guard → Guard.kind). Subject and object differ, so
    // this is a real relationship, not a registration record.
    let defines = SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: EntityRef::new("angular", "Guard", "AuthGuard"),
        object: EntityRef::new("angular", "Guard", "CanActivate"),
        layer: "angular",
        call_evidence: None,
    };
    // Closing the loop with a dependency edge proves the Defines edge is
    // traversed by graph algorithms.
    let injects = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Guard", "CanActivate"),
        object: EntityRef::new("angular", "Guard", "AuthGuard"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![defines, injects]);

    assert_eq!(
        idx.edge_count(),
        2,
        "real Defines relationships must remain graph edges"
    );
    // AuthGuard is the subject of the Defines edge and the object of the
    // Injects edge; CanActivate is the reverse. Occurrence identity is
    // (domain, entity_type, name, file): both identities participate in two
    // edges in the same file, so each is registered exactly once for it.
    // Edge participation does not multiply registrations.
    assert_eq!(
        idx.entities_by_identity("angular", "Guard", "AuthGuard")
            .len(),
        1
    );
    assert_eq!(
        idx.entities_by_identity("angular", "Guard", "CanActivate")
            .len(),
        1
    );

    let outgoing = idx.forward_edges_by_identity("angular", "Guard", "AuthGuard");
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].relation, SemanticRelation::Defines);

    let incoming = idx.reverse_edges_by_identity("angular", "Guard", "CanActivate");
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].relation, SemanticRelation::Defines);

    assert!(
        idx.has_cycle(),
        "Defines(A, B) + Injects(B, A) must be traversed as a cycle"
    );
}

#[test]
fn partial_class_two_files_two_occurrences() {
    let mut idx = WorkspaceIndex::new();
    // Same identity from two different files (e.g. C# partial classes):
    // cross-file occurrences are never deduplicated.
    let registration = || SemanticEdge {
        relation: SemanticRelation::Defines,
        subject: EntityRef::new("builtin", "Class", "Foo"),
        object: EntityRef::new("builtin", "Class", "Foo"),
        layer: "builtin",
        call_evidence: None,
    };

    idx.add_edges("a.cs", vec![registration()]);
    idx.add_edges("b.cs", vec![registration()]);

    let entities = idx.entities_by_identity("builtin", "Class", "Foo");
    assert_eq!(
        entities.len(),
        2,
        "cross-file occurrences of one identity must remain distinct"
    );
    let files: Vec<Option<&String>> = entities.iter().map(|e| e.file.as_ref()).collect();
    assert!(files.contains(&Some(&"a.cs".to_string())));
    assert!(files.contains(&Some(&"b.cs".to_string())));

    assert_eq!(
        idx.entities_in_file("a.cs").len(),
        1,
        "each file discovers exactly its own occurrence"
    );
    assert_eq!(idx.entities_in_file("b.cs").len(), 1);
    assert_eq!(
        idx.edge_count(),
        0,
        "registration records must not become graph edges"
    );
    assert!(!idx.has_cycle());
}

// ── Phase 4c: Graph Traversal — transitive_dependencies ─────────────

#[test]
fn transitive_deps_unknown_entity() {
    let idx = WorkspaceIndex::new();
    let deps = idx.transitive_dependencies("angular", "Component", "NonExistent", 1);
    assert!(deps.is_empty());
}

#[test]
fn transitive_deps_no_outgoing() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![inject_edge("A", "B", Some("a.ts"))]);
    let deps = idx.transitive_dependencies("angular", "Service", "B", 1);
    assert!(deps.is_empty(), "B has no outgoing edges");
}

#[test]
fn transitive_deps_depth_1() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![
            inject_edge("A", "B", Some("a.ts")),
            inject_edge("B", "C", Some("a.ts")),
        ],
    );
    let deps = idx.transitive_dependencies("angular", "Component", "A", 1);
    assert_eq!(deps.len(), 1, "depth=1 should return only B");
    assert_eq!(
        deps[0],
        (
            "angular".to_string(),
            "Service".to_string(),
            "B".to_string()
        )
    );
}

#[test]
fn transitive_deps_depth_2() {
    let mut idx = WorkspaceIndex::new();
    // Same entity_type so chains: A(Service) → B(Service) → C(Service)
    let e1 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "A"),
        object: EntityRef::new("angular", "Service", "B"),
        layer: "angular",
        call_evidence: None,
    };
    let e2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "C"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![e1, e2]);
    let deps = idx.transitive_dependencies("angular", "Service", "A", 2);
    assert_eq!(deps.len(), 2, "depth=2 should return B and C");
}

#[test]
fn transitive_deps_unlimited() {
    let mut idx = WorkspaceIndex::new();
    let e1 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "A"),
        object: EntityRef::new("angular", "Service", "B"),
        layer: "angular",
        call_evidence: None,
    };
    let e2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "C"),
        layer: "angular",
        call_evidence: None,
    };
    let e3 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "C"),
        object: EntityRef::new("angular", "Service", "D"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![e1, e2, e3]);
    let deps = idx.transitive_dependencies("angular", "Service", "A", 0);
    assert_eq!(deps.len(), 3, "unlimited depth should return B, C, D");
}
#[test]
fn transitive_deps_with_cycle() {
    let mut idx = WorkspaceIndex::new();
    // A(Service) → B(Service) → C(Service) → A(Service)
    let e1 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "A"),
        object: EntityRef::new("angular", "Service", "B"),
        layer: "angular",
        call_evidence: None,
    };
    let e2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "C"),
        layer: "angular",
        call_evidence: None,
    };
    let e3 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "C"),
        object: EntityRef::new("angular", "Service", "A"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![e1, e2, e3]);
    let deps = idx.transitive_dependencies("angular", "Service", "A", 0);
    assert!(deps.contains(&(
        "angular".to_string(),
        "Service".to_string(),
        "B".to_string()
    )));
    assert!(deps.contains(&(
        "angular".to_string(),
        "Service".to_string(),
        "C".to_string()
    )));
    assert_eq!(deps.len(), 2, "no infinite loop, start excluded");
}

#[test]
fn transitive_deps_ignores_structural() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![
            inject_edge("A", "B", Some("a.ts")),
            route_edge("/home", "HomeComponent"),
        ],
    );
    let deps = idx.transitive_dependencies("angular", "Component", "A", 1);
    assert_eq!(deps.len(), 1, "only Injects relation should be traversed");
    assert_eq!(
        deps[0],
        (
            "angular".to_string(),
            "Service".to_string(),
            "B".to_string()
        )
    );
}

#[test]
fn transitive_deps_disconnected() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![
            inject_edge("A", "B", Some("a.ts")),
            inject_edge("C", "D", Some("a.ts")),
        ],
    );
    let deps = idx.transitive_dependencies("angular", "Component", "A", 0);
    assert_eq!(deps.len(), 1, "should only return B, not C or D");
    assert_eq!(
        deps[0],
        (
            "angular".to_string(),
            "Service".to_string(),
            "B".to_string()
        )
    );
}

#[test]
fn transitive_deps_duplicate_edge() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![inject_edge("A", "B", Some("a.ts"))]);
    idx.add_edges("b.ts", vec![inject_edge("A", "B", Some("b.ts"))]);
    let deps = idx.transitive_dependencies("angular", "Component", "A", 1);
    assert_eq!(deps.len(), 1, "duplicate edges must not duplicate results");
    assert_eq!(
        deps[0],
        (
            "angular".to_string(),
            "Service".to_string(),
            "B".to_string()
        )
    );
}

#[test]
fn transitive_deps_deterministic() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![
            inject_edge("A", "B", Some("a.ts")),
            inject_edge("B", "C", Some("a.ts")),
            inject_edge("A", "D", Some("a.ts")),
        ],
    );
    let first = idx.transitive_dependencies("angular", "Component", "A", 0);
    let second = idx.transitive_dependencies("angular", "Component", "A", 0);
    assert_eq!(first, second, "same query must produce same result");
}

// -- Phase 4c: Dependency relation boundary (helpers in index_support) --

// ── Phase 4c: Dependency relation boundary ──────────────────────────

#[test]
fn transitive_deps_approved_dependency_traversed_injects() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![inject_edge("A", "B", Some("a.ts"))]);
    let deps = idx.transitive_dependencies("angular", "Component", "A", 1);
    assert_eq!(deps.len(), 1, "Injects must be traversed");
}

#[test]
fn transitive_deps_approved_dependency_traversed_imports_module() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.ts",
        vec![imports_module_edge("AppModule", "SharedModule")],
    );
    let deps = idx.transitive_dependencies("angular", "Module", "AppModule", 1);
    assert_eq!(deps.len(), 1, "ImportsModule must be traversed");
}

#[test]
fn transitive_deps_approved_dependency_traversed_handles_action() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![handles_action_edge("LoadUsers$", "loadUsers")]);
    let deps = idx.transitive_dependencies("ngrx", "Effect", "LoadUsers$", 1);
    assert_eq!(deps.len(), 1, "HandlesAction must be traversed");
}

#[test]
fn transitive_deps_approved_dependency_traversed_config_props() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![config_props_edge("AppConfig", "app")]);
    let deps = idx.transitive_dependencies("spring", "Configuration", "AppConfig", 1);
    assert_eq!(deps.len(), 1, "ConfigurationProperties must be traversed");
}

#[test]
fn transitive_deps_structural_not_traversed_has_selector() {
    let mut idx = WorkspaceIndex::new();
    let sel_edge = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "UserCard"),
        object: EntityRef::new("angular", "Component", "app-user-card"),
        layer: "angular",
        call_evidence: None,
    };
    idx.add_edges("a.ts", vec![sel_edge]);
    let deps = idx.transitive_dependencies("angular", "Component", "UserCard", 1);
    assert!(deps.is_empty(), "HasSelector must NOT be traversed");
}

#[test]
fn transitive_deps_structural_not_traversed_route_maps_to() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![route_edge("/home", "HomeComponent")]);
    let deps = idx.transitive_dependencies("angular", "Route", "/home", 1);
    assert!(deps.is_empty(), "RouteMapsTo must NOT be traversed");
}
