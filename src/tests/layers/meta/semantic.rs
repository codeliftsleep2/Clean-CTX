// src/tests/layers/meta/semantic.rs
//
// Tests for the semantic relationship model (semantic plan Phase 0):
// EntityRef identity semantics, SemanticEdge construction, and the default
// empty MetaLayer::extract_semantic_edges().

use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::layers::meta::{MetaLayer, MetaLayerOutput};

#[test]
fn entity_ref_equality_ignores_file() {
    let a = EntityRef::new("angular", "Component", "UserComponent");
    let b = EntityRef::new("angular", "Component", "UserComponent");
    let c = EntityRef::new("angular", "Component", "UserComponent")
        .with_file("other/path/file.ts".to_string());

    assert_eq!(a, b);
    // file must NOT participate in identity matching (plan U1).
    assert_eq!(a, c);
}

#[test]
fn entity_ref_identity_distinguishes_all_identity_fields() {
    let base = EntityRef::new("angular", "Component", "UserComponent");
    assert_ne!(base, EntityRef::new("ngrx", "Component", "UserComponent"));
    assert_ne!(base, EntityRef::new("angular", "Service", "UserComponent"));
    assert_ne!(
        base,
        EntityRef::new("angular", "Component", "OtherComponent")
    );
}

#[test]
fn entity_ref_hash_matches_identity() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of(e: &EntityRef) -> u64 {
        let mut hasher = DefaultHasher::new();
        e.hash(&mut hasher);
        hasher.finish()
    }

    let a = EntityRef::new("angular", "Component", "UserComponent");
    let same_identity =
        EntityRef::new("angular", "Component", "UserComponent").with_file("p.ts".to_string());
    assert_eq!(hash_of(&a), hash_of(&same_identity));

    let different = EntityRef::new("angular", "Component", "AdminComponent");
    assert_ne!(hash_of(&a), hash_of(&different));
}

#[test]
fn semantic_edge_construction() {
    let edge = SemanticEdge {
        relation: SemanticRelation::RouteMapsTo,
        subject: EntityRef::new("angular", "Route", "/users"),
        object: EntityRef::new("angular", "Component", "UserComponent"),
        layer: "angular",
    };
    assert_eq!(edge.relation, SemanticRelation::RouteMapsTo);
    assert_eq!(edge.layer, "angular");
    assert_eq!(edge.subject, EntityRef::new("angular", "Route", "/users"));
    assert_eq!(
        edge.object,
        EntityRef::new("angular", "Component", "UserComponent")
    );
}

/// Minimal meta-layer implementing the trait WITHOUT overriding
/// extract_semantic_edges() -- the default empty impl must be used.
struct DefaultEdgesLayer;

impl MetaLayer for DefaultEdgesLayer {
    fn name(&self) -> &'static str {
        "default_edges_test"
    }

    fn is_applicable(
        &self,
        _source: &str,
        _path: &std::path::Path,
        _config: Option<&crate::config::CleanCtxConfig>,
    ) -> bool {
        true
    }

    fn enrich(
        &self,
        _source: &str,
        _class_captures: &[String],
        _fidelity: Fidelity,
        _config: Option<&crate::config::CleanCtxConfig>,
    ) -> Option<MetaLayerOutput> {
        None
    }
}

#[test]
fn extract_semantic_edges_default_is_empty() {
    let layer = DefaultEdgesLayer;
    let edges = layer.extract_semantic_edges("source()", &[], Fidelity::Low, None);
    assert!(edges.is_empty());
}

// ── Substrate invariant tests (Phase 1) ─────────────────────────────────

#[test]
fn semantic_edge_supports_cross_domain() {
    // Cross-domain edges are valid (SEM-006): subject in one domain,
    // object in another, edge layer independent of both.
    let edge = SemanticEdge {
        relation: SemanticRelation::Implements,
        subject: EntityRef::new("builtin", "Class", "ApplicationDbContext"),
        object: EntityRef::new("builtin", "Interface", "IApplicationDbContext"),
        layer: "dotnet",
    };
    assert_eq!(edge.subject.domain, "builtin");
    assert_eq!(edge.object.domain, "builtin");
    // Layer provenance independent of endpoint domain (SEM-007).
    assert_eq!(edge.layer, "dotnet");
}

#[test]
fn semantic_edge_layer_independent_of_endpoint_domain() {
    // Edge layer/provenance must not be inferable from endpoint domain.
    // Same relation, same domains, different layers.
    let angular_edge = SemanticEdge {
        relation: SemanticRelation::HasStore,
        subject: EntityRef::new("angular", "Component", "ShellComponent"),
        object: EntityRef::new("ngrx", "Store", "AppState"),
        layer: "ngrx",
    };
    let dotnet_edge = SemanticEdge {
        relation: SemanticRelation::HasEntity,
        subject: EntityRef::new("dotnet", "DbContext", "AppDbContext"),
        object: EntityRef::new("dotnet", "Entity", "Customer"),
        layer: "dotnet",
    };
    assert_ne!(angular_edge.layer, dotnet_edge.layer);
    // NgRx emits cross-domain edges (angular subject → ngrx object).
    assert_eq!(angular_edge.subject.domain, "angular");
    assert_eq!(angular_edge.object.domain, "ngrx");
}

#[test]
fn generic_relations_are_constructible() {
    // The generic substrate vocabulary (Implements, Extends, Binds) must be
    // representable with framework-agnostic semantics (SEM-011/012/013).
    let implements = SemanticEdge {
        relation: SemanticRelation::Implements,
        subject: EntityRef::new("builtin", "Class", "OrderRepository"),
        object: EntityRef::new("builtin", "Interface", "IRepository"),
        layer: "dotnet",
    };
    let extends = SemanticEdge {
        relation: SemanticRelation::Extends,
        subject: EntityRef::new("builtin", "Class", "BaseController"),
        object: EntityRef::new("builtin", "Class", "Controller"),
        layer: "spring",
    };
    let binds = SemanticEdge {
        relation: SemanticRelation::Binds,
        subject: EntityRef::new("dotnet", "Implementation", "AppDbContext"),
        object: EntityRef::new("dotnet", "Token", "IApplicationDbContext"),
        layer: "dotnet",
    };
    assert_eq!(implements.relation, SemanticRelation::Implements);
    assert_eq!(extends.relation, SemanticRelation::Extends);
    assert_eq!(binds.relation, SemanticRelation::Binds);
}

#[test]
fn binds_direction_is_implementation_to_token() {
    // Binds(implementation, token): direction is implementation → token,
    // distinct from Implements (language-level) and Injects (consumer-side).
    let binds = SemanticEdge {
        relation: SemanticRelation::Binds,
        subject: EntityRef::new("dotnet", "Implementation", "AppDbContext"),
        object: EntityRef::new("dotnet", "Token", "IApplicationDbContext"),
        layer: "dotnet",
    };
    assert_eq!(binds.subject.name, "AppDbContext");
    assert_eq!(binds.object.name, "IApplicationDbContext");
}
