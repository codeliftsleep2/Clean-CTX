// src/tests/workspace/index_queries.rs
//
// WorkspaceIndex lookup tests: name-based lookup, inject-target resolution,
// selector resolution, occurrence identity, and the Phase 4a / Phase 4b
// query regressions.
//
// The `#[path]`-loaded module is compiled in both lib and test targets. In
// the lib target the `#[cfg(test)]` gate prevents this from causing
// dead-code warnings; clippy across all targets requires this allow.
#![allow(dead_code)]

use super::index_support::*;
use crate::layers::meta::semantic::*;
use crate::workspace::index::WorkspaceIndex;

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

// ── Occurrence Identity (C1): idempotent entity registration ──────────
//
// Occurrence identity is (domain, entity_type, name, file). These tests
// prove that edge participation does not multiply registrations, that
// cross-file ambiguity is preserved, and that the occurrence-based query
// surfaces report each occurrence exactly once.

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

#[test]
fn resolve_selector_unique_despite_cross_file_duplicate_selector() {
    let mut idx = WorkspaceIndex::new();
    // Two files assert the identical HasSelector triple: one entity identity,
    // one occurrence per file. Occurrence-aware edge storage keeps both edge
    // occurrences, so the resolver must answer with each entity occurrence
    // exactly once instead of once per asserting file.
    let selector = |file: &str| SemanticEdge {
        relation: SemanticRelation::HasSelector,
        subject: EntityRef::new("angular", "Component", "LoadingComponent")
            .with_file(file.to_string()),
        object: EntityRef::new("angular", "Component", "app-loading").with_file(file.to_string()),
        layer: "angular",
    };
    idx.add_edges(
        "project-a/loading.component.ts",
        vec![selector("project-a/loading.component.ts")],
    );
    idx.add_edges(
        "project-b/loading.component.ts",
        vec![selector("project-b/loading.component.ts")],
    );

    let resolved = idx.resolve_selector("app-loading");
    assert_eq!(
        resolved.len(),
        2,
        "both file occurrences must be returned, each exactly once"
    );
    let mut files: Vec<String> = resolved.iter().filter_map(|e| e.file.clone()).collect();
    files.sort();
    assert_eq!(
        files,
        vec![
            "project-a/loading.component.ts".to_string(),
            "project-b/loading.component.ts".to_string()
        ]
    );
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
