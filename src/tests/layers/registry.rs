// src/tests/layers/registry.rs
//
// Tests for LayerRegistry::collect_semantic_edges() (semantic plan Phase 0).

use crate::compression::Fidelity;
use crate::layers::LayerRegistry;

#[test]
fn collect_semantic_edges_defaults_to_empty() {
    let registry = LayerRegistry::default();
    // Phase 0: no meta-layer overrides extract_semantic_edges yet, so the
    // trait's default empty impl applies under every feature combination.
    // The dispatch + default-impl contract is what we verify here; Phase 1
    // makes Angular return real edges.
    let edges = registry.collect_semantic_edges("class Foo {}", &[], Fidelity::Low, None);
    assert!(edges.is_empty());
}

#[test]
fn collect_semantic_edges_empty_source_yields_empty() {
    let registry = LayerRegistry::default();
    let edges = registry.collect_semantic_edges("", &[], Fidelity::High, None);
    assert!(edges.is_empty());
}
