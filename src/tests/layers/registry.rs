// src/tests/layers/registry.rs
//
// Tests for LayerRegistry::collect_semantic_edges() (semantic plan Phase 0).

use crate::compression::Fidelity;
use crate::layers::LayerRegistry;

struct PathIndependentLayer;

impl crate::layers::meta::MetaLayer for PathIndependentLayer {
    fn name(&self) -> &'static str {
        "path_independent_test"
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
    ) -> Option<crate::layers::meta::MetaLayerOutput> {
        Some(crate::layers::meta::MetaLayerOutput {
            layer_name: self.name(),
            rendered: "legacy-output".to_string(),
            ..Default::default()
        })
    }
}

#[test]
fn path_aware_hook_default_forwards_legacy_layer_behavior() {
    use crate::layers::meta::MetaLayer;

    let layer = PathIndependentLayer;
    let legacy = layer.enrich("source", &[], Fidelity::Low, None).unwrap();
    let path_aware = layer
        .enrich_with_path(
            "source",
            std::path::Path::new("actual/path.ts"),
            &[],
            Fidelity::Low,
            None,
        )
        .unwrap();
    assert_eq!(legacy.rendered, path_aware.rendered);
    assert!(
        layer
            .extract_semantic_edges_paired_with_path(
                "source",
                std::path::Path::new("actual/path.ts"),
                &[],
                Fidelity::Low,
                None,
            )
            .is_empty()
    );
}

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

#[cfg(feature = "angular")]
#[test]
fn path_context_does_not_change_path_independent_angular_output() {
    use std::path::Path;

    let source = "import { signal } from '@angular/core'; const count = signal(0);";
    let registry = LayerRegistry::default();
    let legacy = registry.run_meta_layers_pipeline(source, &[], Fidelity::Medium, None);
    let path_aware = registry.run_meta_layers_pipeline_with_path(
        source,
        Path::new("src/state.ts"),
        &[],
        Fidelity::Medium,
        None,
    );
    assert_eq!(legacy.len(), path_aware.len());
    assert_eq!(legacy[0].rendered, path_aware[0].rendered);
}

#[cfg(feature = "angular")]
#[test]
fn canonical_path_reaches_angular_testing_semantics() {
    use std::path::Path;

    let registry = LayerRegistry::default();
    let edges = registry.collect_semantic_edges_with_path(
        "describe('account', () => {});",
        Path::new("src/account.component.spec.ts"),
        &[],
        Fidelity::Low,
        None,
    );
    let edge = edges
        .iter()
        .find(|edge| edge.relation == crate::layers::meta::semantic::SemanticRelation::Tests)
        .expect("filename-based Angular Tests edge");
    assert_eq!(edge.subject.name, "account.component.spec.ts");
    assert_eq!(edge.object.name, "AccountComponent");
}
