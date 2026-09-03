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
        object: EntityRef::new("angular", "Component", "app-user-card"),
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
            name: "app-header".to_string(),
            file: None,
        },
        layer: "angular",
    };
    idx.add_edges("header.component.ts", vec![edge]);

    let results = idx.resolve_selector("app-header");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].file.as_deref(), Some("header.component.ts"));
}

// ── HasSelector literal-value regression ─────────────────────────────
// resolve_selector() MUST match against the literal selector string stored
// by the HasSelector edge, with no bracket encoding. The three distinct
// selector forms MUST resolve independently.

#[test]
fn resolve_selector_literal_forms_are_distinct() {
    let mut idx = WorkspaceIndex::new();

    // Three components, each with a different selector form.
    let element_edge = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "ElementComp"),
        object: EntityRef::new("angular", "Component", "app-widget"),
        layer: "angular",
    };
    let attribute_edge = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "AttributeComp"),
        object: EntityRef::new("angular", "Component", "[app-widget]"),
        layer: "angular",
    };
    let class_edge = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "ClassComp"),
        object: EntityRef::new("angular", "Component", ".app-widget"),
        layer: "angular",
    };
    idx.add_edges(
        "selectors.ts",
        vec![element_edge, attribute_edge, class_edge],
    );

    // Element selector resolves with bare name.
    let element_results = idx.resolve_selector("app-widget");
    assert_eq!(element_results.len(), 1);
    assert_eq!(element_results[0].name, "ElementComp");

    // Attribute selector resolves with bracketed form.
    let attribute_results = idx.resolve_selector("[app-widget]");
    assert_eq!(attribute_results.len(), 1);
    assert_eq!(attribute_results[0].name, "AttributeComp");

    // Class selector resolves with dotted form.
    let class_results = idx.resolve_selector(".app-widget");
    assert_eq!(class_results.len(), 1);
    assert_eq!(class_results[0].name, "ClassComp");

    // Cross-form lookups MUST NOT match.
    assert!(
        idx.resolve_selector("app-widget").len() == 1
            && idx.resolve_selector("app-widget")[0].name == "ElementComp",
        "bare 'app-widget' must resolve only the element selector"
    );
    assert!(
        idx.resolve_selector("[app-widget]").len() == 1
            && idx.resolve_selector("[app-widget]")[0].name == "AttributeComp",
        "'[app-widget]' must resolve only the attribute selector"
    );

    // Unknown selector returns empty.
    assert!(idx.resolve_selector("nonexistent").is_empty());
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
    };
    let edge2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "A"),
        layer: "angular",
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
    };
    let edge2 = SemanticEdge {
        relation: SemanticRelation::DeclaresInModule,
        subject: EntityRef::new("angular", "Module", "AppModule"),
        object: EntityRef::new("angular", "Route", "/home"),
        layer: "angular",
    };
    let edge3 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Component", "HomeComponent"),
        object: EntityRef::new("angular", "Module", "AppModule"),
        layer: "angular",
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
    };
    // Closing the loop with a dependency edge proves the Defines edge is
    // traversed by graph algorithms.
    let injects = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Guard", "CanActivate"),
        object: EntityRef::new("angular", "Guard", "AuthGuard"),
        layer: "angular",
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
// ── Occurrence Identity (C1): idempotent entity registration ──────────
//
// Occurrence identity is (domain, entity_type, name, file). These tests
// prove that edge participation does not multiply registrations, that
// cross-file ambiguity is preserved, and that the occurrence-based query
// surfaces report each occurrence exactly once.

/// Helper: build a ControllerAction edge (dotnet controller shape).
fn controller_action_edge(controller: &str, action: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::ControllerAction,
        subject: EntityRef::new("dotnet", "Controller", controller),
        object: EntityRef::new("dotnet", "Action", action),
        layer: "dotnet",
    }
}

/// Helper: build a HasRoute edge (dotnet controller shape).
fn has_route_edge(controller: &str, route: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::HasRoute,
        subject: EntityRef::new("dotnet", "Controller", controller),
        object: EntityRef::new("dotnet", "Route", route),
        layer: "dotnet",
    }
}

/// A: Multi-edge single entity — multiple legitimate edges mentioning the
/// same subject must produce exactly one entity occurrence.
#[test]
fn multi_edge_subject_registers_one_occurrence() {
    let mut idx = WorkspaceIndex::new();
    // Controller shape: 1 HasRoute + 3 ControllerAction — the subject
    // participates in 4 legitimate edges in one file.
    let mut edges = vec![has_route_edge("OrdersController", "api/orders")];
    for action in ["GetAll", "GetById", "Create"] {
        edges.push(controller_action_edge("OrdersController", action));
    }
    idx.add_edges("orders.cs", edges);

    assert_eq!(idx.edge_count(), 4, "every legitimate edge is indexed");
    assert_eq!(
        idx.entities_by_identity("dotnet", "Controller", "OrdersController")
            .len(),
        1,
        "the subject participated in 4 edges but must be registered once for the file"
    );
    assert_eq!(
        idx.entity_occurrence_count(),
        5,
        "semantic content: 1 controller + 1 route + 3 actions"
    );
}

/// B: Cross-file preservation — the same entity identity occurring in two
/// files (each mentioning it in multiple edges) must produce two
/// occurrences, one per file.
#[test]
fn cross_file_occurrences_preserved_for_multi_edge_subjects() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "a.cs",
        vec![
            has_route_edge("OrdersController", "api/orders"),
            controller_action_edge("OrdersController", "GetAll"),
            controller_action_edge("OrdersController", "GetById"),
        ],
    );
    idx.add_edges(
        "b.cs",
        vec![
            has_route_edge("OrdersController", "api/orders"),
            controller_action_edge("OrdersController", "GetAll"),
        ],
    );

    let entities = idx.entities_by_identity("dotnet", "Controller", "OrdersController");
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
        4,
        "file A discovers exactly its own occurrences: controller + route + 2 actions"
    );
    assert_eq!(
        idx.entities_in_file("b.cs").len(),
        3,
        "file B discovers exactly its own occurrences: controller + route + 1 action"
    );
}

/// C: Recompile/re-ingest idempotency — repeated ingestion of the same file
/// must not accumulate duplicate entity occurrences, both with and without
/// an intervening `remove_file`.
#[test]
fn reingest_does_not_accumulate_occurrences() {
    let mut idx = WorkspaceIndex::new();
    let edges = || {
        vec![
            has_route_edge("OrdersController", "api/orders"),
            controller_action_edge("OrdersController", "GetAll"),
            controller_action_edge("OrdersController", "GetById"),
        ]
    };

    // Misuse pattern: repeated ingestion without an intervening remove_file.
    idx.add_edges("orders.cs", edges());
    let after_first = idx.entity_occurrence_count();
    idx.add_edges("orders.cs", edges());
    assert_eq!(
        idx.entity_occurrence_count(),
        after_first,
        "re-ingesting the same file must not accumulate occurrences"
    );
    assert_eq!(
        idx.entities_in_file("orders.cs").len(),
        after_first,
        "entities_in_file must not report duplicates after re-ingestion"
    );

    // Production recompile lifecycle: remove_file → add_edges.
    idx.remove_file("orders.cs");
    idx.add_edges("orders.cs", edges());
    assert_eq!(
        idx.entity_occurrence_count(),
        after_first,
        "recompilation must not accumulate occurrences"
    );
    assert_eq!(idx.edge_count(), 3, "the legitimate edge set is unchanged");
}

/// D: `entities_in_file()` uniqueness — a file whose entity is referenced by
/// multiple edges must return that entity exactly once.
#[test]
fn entities_in_file_reports_each_occurrence_once() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges(
        "orders.cs",
        vec![
            has_route_edge("OrdersController", "api/orders"),
            controller_action_edge("OrdersController", "GetAll"),
            controller_action_edge("OrdersController", "GetById"),
        ],
    );

    let file_entities = idx.entities_in_file("orders.cs");
    assert_eq!(
        file_entities.len(),
        4,
        "controller + route + 2 actions — the controller is mentioned by 3 edges but returned once"
    );
    let distinct: std::collections::BTreeSet<(&str, &str, &str)> = file_entities
        .iter()
        .map(|e| (e.domain, e.entity_type, e.name.as_str()))
        .collect();
    assert_eq!(
        distinct.len(),
        file_entities.len(),
        "every returned occurrence must be a distinct identity"
    );
}

/// E: Resolution-surface uniqueness — occurrence-based resolution must not
/// multiply an occurrence merely because it participates in multiple edges.
#[test]
fn resolve_inject_type_unique_despite_multiple_injectors() {
    let mut idx = WorkspaceIndex::new();
    // Three components in one file inject the same service.
    idx.add_edges(
        "app.ts",
        vec![
            inject_edge("CompA", "UserService", Some("app.ts")),
            inject_edge("CompB", "UserService", Some("app.ts")),
            inject_edge("CompC", "UserService", Some("app.ts")),
        ],
    );

    let targets = idx.resolve_inject_type("UserService");
    assert_eq!(
        targets.len(),
        1,
        "a target injected by 3 components must resolve to one occurrence, not 3"
    );
    assert_eq!(targets[0].file.as_deref(), Some("app.ts"));
}

#[test]
fn resolve_selector_unique_despite_subject_multi_edge() {
    let mut idx = WorkspaceIndex::new();
    // One component: 1 HasSelector + 2 Injects — the subject participates in
    // 3 edges. Selector resolution must not multiply by the subject's other
    // edges.
    let selector = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "UserCard"),
        object: EntityRef::new("angular", "Component", "app-user-card"),
        layer: "angular",
    };
    idx.add_edges(
        "user-card.ts",
        vec![
            selector,
            inject_edge("UserCard", "SvcOne", Some("user-card.ts")),
            inject_edge("UserCard", "SvcTwo", Some("user-card.ts")),
        ],
    );

    let resolved = idx.resolve_selector("app-user-card");
    assert_eq!(
        resolved.len(),
        1,
        "selector resolution must return the component once regardless of its other edges"
    );
    assert_eq!(resolved[0].name, "UserCard");
    assert_eq!(resolved[0].file.as_deref(), Some("user-card.ts"));
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
    };
    let e2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "C"),
        layer: "angular",
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
    };
    let e2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "C"),
        layer: "angular",
    };
    let e3 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "C"),
        object: EntityRef::new("angular", "Service", "D"),
        layer: "angular",
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
    };
    let e2 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "B"),
        object: EntityRef::new("angular", "Service", "C"),
        layer: "angular",
    };
    let e3 = SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", "C"),
        object: EntityRef::new("angular", "Service", "A"),
        layer: "angular",
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

// ── Phase 4c: Dependency relation boundary ──────────────────────────
/// Helper: create an ImportsModule edge (dependency relation).
fn imports_module_edge(from: &str, to: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::ImportsModule,
        subject: EntityRef::new("angular", "Module", from),
        object: EntityRef::new("angular", "Module", to),
        layer: "angular",
    }
}

/// Helper: create a HandlesAction edge (dependency relation).
fn handles_action_edge(effect: &str, action: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::HandlesAction,
        subject: EntityRef::new("ngrx", "Effect", effect),
        object: EntityRef::new("ngrx", "Action", action),
        layer: "ngrx",
    }
}

/// Helper: create a ConfigurationProperties edge (dependency relation).
fn config_props_edge(config: &str, prefix: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::ConfigurationProperties,
        subject: EntityRef::new("spring", "Configuration", config),
        object: EntityRef::new("spring", "Properties", prefix),
        layer: "spring",
    }
}

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

// ── Phase 4c: Phase 4b Regression ────────────────────────────────────

#[test]
fn phase_4b_find_entities_by_name_unchanged() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![inject_edge("A", "B", Some("a.ts"))]);
    assert_eq!(idx.find_entities_by_name("A").len(), 1);
    assert!(idx.find_entities_by_name("NonExistent").is_empty());
}

#[test]
fn phase_4b_resolve_inject_type_unchanged() {
    let mut idx = WorkspaceIndex::new();
    idx.add_edges("a.ts", vec![inject_edge("A", "B", Some("a.ts"))]);
    assert_eq!(idx.resolve_inject_type("B").len(), 1);
    assert!(idx.resolve_inject_type("A").is_empty());
}

#[test]
fn phase_4b_resolve_selector_unchanged() {
    let mut idx = WorkspaceIndex::new();
    let sel_edge = SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "C"),
        object: EntityRef::new("angular", "Component", "app-c"),
        layer: "angular",
    };
    idx.add_edges("a.ts", vec![sel_edge]);
    assert_eq!(idx.resolve_selector("app-c").len(), 1);
    assert!(idx.resolve_selector("nonexistent").is_empty());
}
