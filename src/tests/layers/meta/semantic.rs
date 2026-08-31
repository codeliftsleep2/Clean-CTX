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
