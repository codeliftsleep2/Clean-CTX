// src/tests/layers/registry.rs
//
// Tests for LayerRegistry::collect_semantic_edges() (semantic plan Phase 0).

use crate::compression::Fidelity;
use crate::layers::LayerRegistry;

#[test]
fn collect_semantic_edges_defaults_to_empty() {
    let registry = LayerRegistry::default();
    // With NO type captures, no meta layer — including the always-on
    // BuiltinMetaLayer — has any declaration to project, so the collection
    // is empty under every feature combination. The dispatch + empty-capture
    // contract is what we verify here.
    let edges = registry.collect_semantic_edges("class Foo {}", &[], Fidelity::Low, None);
    assert!(edges.is_empty());
}

#[test]
fn collect_semantic_edges_empty_source_yields_empty() {
    let registry = LayerRegistry::default();
    let edges = registry.collect_semantic_edges("", &[], Fidelity::High, None);
    assert!(edges.is_empty());
}
