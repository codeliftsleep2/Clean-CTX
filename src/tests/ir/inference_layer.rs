// src/tests/ir/inference_layer.rs
//
// Tests for R-43b Phase 3: Inference Layer

use crate::ir::inference_layer::{
    InferenceAnnotation, InferenceEdge, InferenceEdgeType, InferenceLayer, InferenceSource,
};

#[test]
fn test_inference_layer_new() {
    let layer = InferenceLayer::new();
    assert!(layer.inferred_edges.is_empty());
    assert!(layer.annotations.is_empty());
}

#[test]
fn test_add_edge() {
    let mut layer = InferenceLayer::new();
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "M1".to_string(),
        to: "M2".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    });
    assert_eq!(layer.inferred_edges.len(), 1);
}

#[test]
fn test_add_annotation() {
    let mut layer = InferenceLayer::new();
    layer.add_annotation(
        "M1".to_string(),
        InferenceAnnotation {
            key: "importance".to_string(),
            value: "0.9".to_string(),
            confidence: 0.75,
            source: InferenceSource::Cbm,
        },
    );
    let anns = layer.all_annotations_for("M1");
    assert_eq!(anns.len(), 1);
    assert_eq!(anns[0].key, "importance");
}

#[test]
fn test_edges_with_confidence() {
    let mut layer = InferenceLayer::new();
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "M1".to_string(),
        to: "M2".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    });
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Extends,
        from: "C1".to_string(),
        to: "C2".to_string(),
        confidence: 1.0,
        source: InferenceSource::Structural,
    });

    let high_conf = layer.edges_with_confidence(0.8);
    assert_eq!(high_conf.len(), 1);

    let all = layer.edges_with_confidence(0.0);
    assert_eq!(all.len(), 2);
}

#[test]
fn test_annotations_for_with_threshold() {
    let mut layer = InferenceLayer::new();
    layer.add_annotation(
        "M1".to_string(),
        InferenceAnnotation {
            key: "importance".to_string(),
            value: "0.9".to_string(),
            confidence: 0.75,
            source: InferenceSource::Cbm,
        },
    );
    layer.add_annotation(
        "M1".to_string(),
        InferenceAnnotation {
            key: "dead_code".to_string(),
            value: "unused".to_string(),
            confidence: 0.5,
            source: InferenceSource::Heuristic,
        },
    );

    let high = layer.annotations_for("M1", 0.7);
    assert_eq!(high.len(), 1);
    assert_eq!(high[0].key, "importance");
}

#[test]
fn test_has_annotation_key() {
    let mut layer = InferenceLayer::new();
    layer.add_annotation(
        "M1".to_string(),
        InferenceAnnotation {
            key: "dead_code".to_string(),
            value: "unused".to_string(),
            confidence: 0.5,
            source: InferenceSource::Heuristic,
        },
    );
    assert!(layer.has_annotation_key("dead_code"));
    assert!(!layer.has_annotation_key("importance"));
}

#[test]
fn test_empty_layer() {
    let layer = InferenceLayer::new();
    assert!(layer.edges_with_confidence(0.0).is_empty());
    assert!(layer.annotations_for("M1", 0.0).is_empty());
    assert!(layer.all_annotations_for("M1").is_empty());
    assert!(!layer.has_annotation_key("anything"));
}