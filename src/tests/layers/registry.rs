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
/// Phase 30-A integration: a Spring @Repository class implementing an
/// interface produces BOTH the language fact (builtin Implements, from the
/// Java-attributed builtin path) and the DI fact (spring Repository Binds to
/// its own class token) through the normal cross-layer dispatch — and no
/// Binds edge is ever fabricated for the implemented interface.
/// Requires the spring_boot feature (the Spring layer is feature-gated).
#[cfg(feature = "spring_boot")]
#[test]
fn collect_semantic_edges_java_implements_and_spring_binds_independent() {
    let registry = LayerRegistry::default();
    let source = r#"
package com.example;

import org.springframework.stereotype.Repository;

@Repository
public class SqlUserRepository implements UserRepository {}
"#;
    // The capture span is the class declaration including its annotation —
    // NOT the whole file (mirrors the C-22 capture shape the pipeline feeds
    // to meta layers; package/import lines belong to the source, which is
    // what `is_java_source` attributes).
    let capture = "@Repository\npublic class SqlUserRepository implements UserRepository {}";
    let captures = vec![("class.root".to_string(), capture.to_string())];
    let edges = registry.collect_semantic_edges(source, &captures, Fidelity::High, None);

    let ifaces: Vec<&crate::layers::meta::semantic::SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == crate::layers::meta::semantic::SemanticRelation::Implements)
        .collect();
    assert_eq!(ifaces.len(), 1, "builtin projects the Java implements fact");
    assert_eq!(ifaces[0].subject.name, "SqlUserRepository");
    assert_eq!(ifaces[0].object.name, "UserRepository");

    let binds: Vec<&crate::layers::meta::semantic::SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == crate::layers::meta::semantic::SemanticRelation::Binds)
        .collect();
    assert!(
        binds.iter().any(|e| e.object.name == "SqlUserRepository"),
        "Spring binds the repository's own class token"
    );
    assert!(
        !binds.iter().any(|e| e.object.name == "UserRepository"),
        "implementing an interface must not create a Binds edge to it"
    );
}
