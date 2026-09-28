// src/tests/layers/registry.rs
//
// Tests for LayerRegistry::collect_semantic_edges() (semantic plan Phase 0).

use crate::compression::Fidelity;
use crate::layers::LayerRegistry;

struct PathIndependentLayer;

struct ContextSemanticLayer;

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

impl crate::layers::meta::MetaLayer for ContextSemanticLayer {
    fn name(&self) -> &'static str {
        "context_semantic_test"
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
        None
    }

    fn extract_semantic_edges(
        &self,
        _source: &str,
        _class_captures: &[String],
        _fidelity: Fidelity,
        _config: Option<&crate::config::CleanCtxConfig>,
    ) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
        use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

        vec![SemanticEdge {
            relation: SemanticRelation::Extends,
            subject: EntityRef::new("test", "Class", "Child"),
            object: EntityRef::new("test", "Class", "Parent"),
            layer: "test",
            call_evidence: None,
        }]
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
fn compilation_context_adapter_preserves_legacy_layer_output() {
    use crate::layers::meta::{MetaLayer, MetaLayerContext};

    let source = "source";
    let path = std::path::Path::new("actual/path.ts");
    let captures = Vec::new();
    let paired_captures = Vec::new();
    let lexical_regions = crate::meta_util::LexicalRegions::new(source);
    let context = MetaLayerContext::new(
        source,
        path,
        &captures,
        &paired_captures,
        Fidelity::Low,
        None,
        &lexical_regions,
    );
    let layer = PathIndependentLayer;

    let legacy = layer
        .enrich_with_path(source, path, &captures, Fidelity::Low, None)
        .expect("legacy layer output");
    let contextual = layer
        .enrich_context(&context)
        .expect("context adapter must preserve legacy layer output");

    assert_eq!(contextual.rendered, legacy.rendered);
}

#[test]
fn compilation_context_adapter_preserves_legacy_semantic_edges() {
    use crate::layers::meta::{MetaLayer, MetaLayerContext};

    let source = "class Child extends Parent {}";
    let path = std::path::Path::new("child.ts");
    let captures = Vec::new();
    let paired_captures = Vec::new();
    let lexical_regions = crate::meta_util::LexicalRegions::new(source);
    let context = MetaLayerContext::new(
        source,
        path,
        &captures,
        &paired_captures,
        Fidelity::Low,
        None,
        &lexical_regions,
    );
    let layer = ContextSemanticLayer;

    let legacy = layer.extract_semantic_edges_paired_with_path(
        source,
        path,
        &paired_captures,
        Fidelity::Low,
        None,
    );
    let contextual = layer.extract_semantic_edges_context(&context);

    assert_eq!(contextual.len(), legacy.len());
    assert_eq!(contextual[0].relation, legacy[0].relation);
}

#[test]
fn registry_context_dispatch_preserves_applicable_layer_output() {
    use crate::layers::meta::MetaLayerContext;

    let registry = LayerRegistry {
        languages: Vec::new(),
        meta_layers: vec![Box::new(PathIndependentLayer)],
    };
    let source = "source";
    let path = std::path::Path::new("actual/path.ts");
    let captures = Vec::new();
    let paired_captures = Vec::new();
    let lexical_regions = crate::meta_util::LexicalRegions::new(source);
    let context = MetaLayerContext::new(
        source,
        path,
        &captures,
        &paired_captures,
        Fidelity::Low,
        None,
        &lexical_regions,
    );

    let outputs = registry.run_meta_layers_context(&context);

    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].rendered, "legacy-output");
}

#[test]
fn registry_context_dispatch_preserves_applicable_semantic_edges() {
    use crate::layers::meta::MetaLayerContext;

    let registry = LayerRegistry {
        languages: Vec::new(),
        meta_layers: vec![Box::new(ContextSemanticLayer)],
    };
    let source = "class Child extends Parent {}";
    let path = std::path::Path::new("child.ts");
    let captures = Vec::new();
    let paired_captures = Vec::new();
    let lexical_regions = crate::meta_util::LexicalRegions::new(source);
    let context = MetaLayerContext::new(
        source,
        path,
        &captures,
        &paired_captures,
        Fidelity::Low,
        None,
        &lexical_regions,
    );

    let edges = registry.collect_semantic_edges_context(&context);

    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].relation,
        crate::layers::meta::semantic::SemanticRelation::Extends
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
