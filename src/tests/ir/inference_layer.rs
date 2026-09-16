// src/tests/ir/inference_layer.rs
//
// Tests for R-43b Phase 3: Inference Layer

use std::collections::HashMap;

use crate::cbm::bridge::test_helpers::new_mock_with_edges;
use crate::cbm::bridge::{DeadCodeEntry, SymbolImportance};
use crate::ir::inference_layer::{
    InferenceAnnotation, InferenceEdge, InferenceEdgeType, InferenceLayer, InferenceSource,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

// ── Basic Functionality ─────────────────────────────────────────────

#[test]
fn test_inference_layer_new() {
    let layer = InferenceLayer::new();
    assert!(layer.inferred_edges.is_empty());
    assert!(layer.annotations.is_empty());
}

#[test]
fn test_inference_layer_default() {
    let layer = InferenceLayer::default();
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

#[test]
fn test_empty_symbol_name() {
    let mut layer = InferenceLayer::new();
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "".to_string(),
        to: "".to_string(),
        confidence: 1.0,
        source: InferenceSource::Structural,
    });
    assert_eq!(layer.inferred_edges.len(), 1);
    assert_eq!(layer.inferred_edges[0].from, "");
    assert_eq!(layer.inferred_edges[0].to, "");
}

#[test]
fn test_special_characters_in_symbols() {
    let mut layer = InferenceLayer::new();
    let special = "fn_foo::<T>(&self) -> Result<(), Error>";

    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: special.to_string(),
        to: "bar".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    });

    layer.add_annotation(
        special.to_string(),
        InferenceAnnotation {
            key: "complexity".to_string(),
            value: "high".to_string(),
            confidence: 0.5,
            source: InferenceSource::Heuristic,
        },
    );

    assert_eq!(layer.inferred_edges.len(), 1);
    assert_eq!(layer.inferred_edges[0].from, special);

    let anns = layer.all_annotations_for(special);
    assert_eq!(anns.len(), 1);
}

#[test]
fn test_unicode_symbols() {
    let mut layer = InferenceLayer::new();
    let unicode = "über_函数_λ";

    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::DataFlowRead,
        from: unicode.to_string(),
        to: "result".to_string(),
        confidence: 0.5,
        source: InferenceSource::Heuristic,
    });

    layer.add_annotation(
        unicode.to_string(),
        InferenceAnnotation {
            key: "name".to_string(),
            value: unicode.to_string(),
            confidence: 1.0,
            source: InferenceSource::Structural,
        },
    );

    assert_eq!(layer.inferred_edges.len(), 1);
    assert_eq!(layer.inferred_edges[0].from, unicode);

    let anns = layer.all_annotations_for(unicode);
    assert_eq!(anns.len(), 1);
    assert_eq!(anns[0].value, unicode);
}

// ── Clone and Debug Traits ──────────────────────────────────────────

#[test]
fn test_inference_edge_clone() {
    let edge = InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "A".to_string(),
        to: "B".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    };
    let cloned = edge.clone();
    assert_eq!(edge.from, cloned.from);
    assert_eq!(edge.to, cloned.to);
    assert_eq!(edge.confidence, cloned.confidence);
    assert_eq!(edge.edge_type, cloned.edge_type);
    assert_eq!(edge.source, cloned.source);
}

#[test]
fn test_inference_annotation_clone() {
    let ann = InferenceAnnotation {
        key: "importance".to_string(),
        value: "0.9".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    };
    let cloned = ann.clone();
    assert_eq!(ann.key, cloned.key);
    assert_eq!(ann.value, cloned.value);
    assert_eq!(ann.confidence, cloned.confidence);
    assert_eq!(ann.source, cloned.source);
}

#[test]
fn test_inference_layer_clone() {
    let mut layer = InferenceLayer::new();
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "A".to_string(),
        to: "B".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    });
    layer.add_annotation(
        "A".to_string(),
        InferenceAnnotation {
            key: "importance".to_string(),
            value: "0.9".to_string(),
            confidence: 0.75,
            source: InferenceSource::Cbm,
        },
    );

    let cloned = layer.clone();
    assert_eq!(cloned.inferred_edges.len(), 1);
    assert_eq!(cloned.annotations.len(), 1);
    assert_eq!(cloned.inferred_edges[0].from, "A");
}

#[test]
fn test_inference_edge_debug() {
    let edge = InferenceEdge {
        edge_type: InferenceEdgeType::DataFlowWrite,
        from: "A".to_string(),
        to: "B".to_string(),
        confidence: 0.5,
        source: InferenceSource::Heuristic,
    };
    let debug_str = format!("{:?}", edge);
    assert!(debug_str.contains("DataFlowWrite"));
    assert!(debug_str.contains("A"));
    assert!(debug_str.contains("B"));
    assert!(debug_str.contains("0.5"));
}

#[test]
fn test_inference_annotation_debug() {
    let ann = InferenceAnnotation {
        key: "dead_code".to_string(),
        value: "true".to_string(),
        confidence: 0.5,
        source: InferenceSource::Heuristic,
    };
    let debug_str = format!("{:?}", ann);
    assert!(debug_str.contains("dead_code"));
    assert!(debug_str.contains("true"));
    assert!(debug_str.contains("0.5"));
}

// ── Stress / Large Data ─────────────────────────────────────────────

#[test]
fn test_many_edges() {
    let mut layer = InferenceLayer::new();
    for i in 0..1000 {
        layer.add_edge(InferenceEdge {
            edge_type: InferenceEdgeType::Calls,
            from: format!("S{}", i),
            to: format!("T{}", i),
            confidence: if i % 2 == 0 { 0.75 } else { 0.5 },
            source: InferenceSource::Cbm,
        });
    }
    assert_eq!(layer.inferred_edges.len(), 1000);

    let high_conf = layer.edges_with_confidence(0.6);
    assert_eq!(high_conf.len(), 500);
}

#[test]
fn test_many_annotations() {
    let mut layer = InferenceLayer::new();
    for i in 0..500 {
        layer.add_annotation(
            format!("S{}", i % 50), // 50 symbols, 10 annotations each
            InferenceAnnotation {
                key: format!("key_{}", i),
                value: format!("value_{}", i),
                confidence: 0.5 + (i as f64 * 0.001).min(0.5),
                source: InferenceSource::Heuristic,
            },
        );
    }
    assert_eq!(layer.annotations.len(), 50);

    // Each symbol should have 10 annotations
    for i in 0..50 {
        let anns = layer.all_annotations_for(&format!("S{}", i));
        assert_eq!(anns.len(), 10, "Symbol S{} should have 10 annotations", i);
    }
}

// ── Interaction Tests ───────────────────────────────────────────────

#[test]
fn test_edges_and_annotations_independent() {
    let mut layer = InferenceLayer::new();

    // Add edges
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "A".to_string(),
        to: "B".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    });

    // Add annotations for different symbol
    layer.add_annotation(
        "C".to_string(),
        InferenceAnnotation {
            key: "importance".to_string(),
            value: "0.8".to_string(),
            confidence: 0.5,
            source: InferenceSource::Heuristic,
        },
    );

    // Edges should not affect annotations and vice versa
    assert_eq!(layer.inferred_edges.len(), 1);
    assert_eq!(layer.annotations.len(), 1);

    // Symbol with edge but no annotation
    assert!(layer.all_annotations_for("A").is_empty());

    // Symbol with annotation but no edge
    let edges_from_c: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.from == "C")
        .collect();
    assert!(edges_from_c.is_empty());
}

#[test]
fn test_has_annotation_key_multiple_symbols() {
    let mut layer = InferenceLayer::new();
    layer.add_annotation(
        "A".to_string(),
        InferenceAnnotation {
            key: "importance".to_string(),
            value: "0.9".to_string(),
            confidence: 0.75,
            source: InferenceSource::Cbm,
        },
    );
    layer.add_annotation(
        "B".to_string(),
        InferenceAnnotation {
            key: "dead_code".to_string(),
            value: "true".to_string(),
            confidence: 0.5,
            source: InferenceSource::Heuristic,
        },
    );

    assert!(layer.has_annotation_key("importance"));
    assert!(layer.has_annotation_key("dead_code"));
    assert!(!layer.has_annotation_key("blast_radius"));
}

// ── Edge Type Equality ──────────────────────────────────────────────

#[test]
fn test_edge_type_equality() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(InferenceEdgeType::Calls);
    set.insert(InferenceEdgeType::DataFlowRead);
    set.insert(InferenceEdgeType::DataFlowWrite);
    set.insert(InferenceEdgeType::Injects);
    set.insert(InferenceEdgeType::Extends);
    set.insert(InferenceEdgeType::Implements);

    assert_eq!(set.len(), 6);

    // Duplicate insert should not increase size
    set.insert(InferenceEdgeType::Calls);
    assert_eq!(set.len(), 6);
}

#[test]
fn test_source_equality() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(InferenceSource::Structural);
    set.insert(InferenceSource::Cbm);
    set.insert(InferenceSource::Heuristic);
    set.insert(InferenceSource::AiGenerated);

    assert_eq!(set.len(), 4);

    // Duplicate insert should not increase size
    set.insert(InferenceSource::Structural);
    assert_eq!(set.len(), 4);
}

#[path = "inference_layer_edges.rs"]
mod inference_layer_edges;

#[path = "inference_layer_cbm.rs"]
mod inference_layer_cbm;
