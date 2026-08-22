// src/tests/ir/inference_layer.rs
//
// Tests for R-43b Phase 3: Inference Layer

use std::collections::HashMap;

use crate::cbm::bridge::test_helpers::new_mock_with_edges;
use crate::cbm::bridge::{DeadCodeEntry, SymbolImportance};
use crate::ir::inference_layer::{
    InferenceAnnotation, InferenceEdge, InferenceEdgeType, InferenceLayer, InferenceSource,
};

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

// ── All Edge Types ──────────────────────────────────────────────────

#[test]
fn test_all_edge_types() {
    let mut layer = InferenceLayer::new();
    let edge_types = vec![
        (InferenceEdgeType::Calls, "Calls"),
        (InferenceEdgeType::DataFlowRead, "DataFlowRead"),
        (InferenceEdgeType::DataFlowWrite, "DataFlowWrite"),
        (InferenceEdgeType::Injects, "Injects"),
        (InferenceEdgeType::Extends, "Extends"),
        (InferenceEdgeType::Implements, "Implements"),
    ];

    for (i, (edge_type, _name)) in edge_types.iter().enumerate() {
        layer.add_edge(InferenceEdge {
            edge_type: edge_type.clone(),
            from: format!("S{}", i),
            to: format!("T{}", i),
            confidence: 0.5,
            source: InferenceSource::Heuristic,
        });
    }

    assert_eq!(layer.inferred_edges.len(), 6);

    // Verify each edge type is present
    let types_present: Vec<InferenceEdgeType> = layer
        .inferred_edges
        .iter()
        .map(|e| e.edge_type.clone())
        .collect();
    for (edge_type, _) in &edge_types {
        assert!(
            types_present.contains(edge_type),
            "Missing edge type: {:?}",
            edge_type
        );
    }
}

// ── All Inference Sources ───────────────────────────────────────────

#[test]
fn test_all_inference_sources() {
    let mut layer = InferenceLayer::new();
    let sources = vec![
        (InferenceSource::Structural, "Structural", 1.0),
        (InferenceSource::Cbm, "Cbm", 0.75),
        (InferenceSource::Heuristic, "Heuristic", 0.5),
        (InferenceSource::AiGenerated, "AiGenerated", 0.25),
    ];

    for (i, (source, name, expected_conf)) in sources.iter().enumerate() {
        layer.add_edge(InferenceEdge {
            edge_type: InferenceEdgeType::Calls,
            from: format!("S{}", i),
            to: format!("T{}", i),
            confidence: *expected_conf,
            source: source.clone(),
        });
        layer.add_annotation(
            format!("S{}", i),
            InferenceAnnotation {
                key: format!("key_{}", name),
                value: "test".to_string(),
                confidence: *expected_conf,
                source: source.clone(),
            },
        );
    }

    assert_eq!(layer.inferred_edges.len(), 4);

    // Verify each source is present in edges
    let sources_present: Vec<InferenceSource> = layer
        .inferred_edges
        .iter()
        .map(|e| e.source.clone())
        .collect();
    for (source, _, _) in &sources {
        assert!(
            sources_present.contains(source),
            "Missing source: {:?}",
            source
        );
    }

    // Verify each source is present in annotations
    for (_source, name, _) in &sources {
        let key = format!("key_{}", name);
        assert!(
            layer.has_annotation_key(&key),
            "Missing annotation key: {}",
            key
        );
    }
}

// ── Confidence Boundary Tests ───────────────────────────────────────

#[test]
fn test_confidence_boundary_zero() {
    let mut layer = InferenceLayer::new();
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "A".to_string(),
        to: "B".to_string(),
        confidence: 0.0,
        source: InferenceSource::AiGenerated,
    });

    // Edge with 0.0 confidence should be included at threshold 0.0
    let edges = layer.edges_with_confidence(0.0);
    assert_eq!(edges.len(), 1);

    // Edge with 0.0 confidence should be excluded at threshold > 0.0
    let edges = layer.edges_with_confidence(0.0001);
    assert_eq!(edges.len(), 0);
}

#[test]
fn test_confidence_boundary_one() {
    let mut layer = InferenceLayer::new();
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "A".to_string(),
        to: "B".to_string(),
        confidence: 1.0,
        source: InferenceSource::Structural,
    });

    // Edge with 1.0 confidence should be included at threshold 1.0
    let edges = layer.edges_with_confidence(1.0);
    assert_eq!(edges.len(), 1);
}

#[test]
fn test_confidence_boundary_annotation() {
    let mut layer = InferenceLayer::new();
    layer.add_annotation(
        "M1".to_string(),
        InferenceAnnotation {
            key: "test".to_string(),
            value: "value".to_string(),
            confidence: 0.5,
            source: InferenceSource::Heuristic,
        },
    );

    // At exact threshold
    let anns = layer.annotations_for("M1", 0.5);
    assert_eq!(anns.len(), 1);

    // Just above threshold
    let anns = layer.annotations_for("M1", 0.5001);
    assert_eq!(anns.len(), 0);
}

// ── Multiple Symbols ────────────────────────────────────────────────

#[test]
fn test_multiple_symbols() {
    let mut layer = InferenceLayer::new();
    let symbols = vec!["A", "B", "C", "D", "E"];

    for (i, sym) in symbols.iter().enumerate() {
        layer.add_annotation(
            sym.to_string(),
            InferenceAnnotation {
                key: "importance".to_string(),
                value: format!("{}", i as f64 / 10.0),
                confidence: 0.5 + (i as f64 * 0.1),
                source: InferenceSource::Heuristic,
            },
        );
    }

    // Each symbol should have its annotation
    for sym in &symbols {
        let anns = layer.all_annotations_for(sym);
        assert_eq!(anns.len(), 1, "Symbol {} should have 1 annotation", sym);
    }

    // Unknown symbol should return empty
    assert!(layer.all_annotations_for("Unknown").is_empty());
}

#[test]
fn test_multiple_annotations_per_symbol() {
    let mut layer = InferenceLayer::new();
    let keys = vec!["importance", "dead_code", "blast_radius", "complexity"];

    for key in &keys {
        layer.add_annotation(
            "M1".to_string(),
            InferenceAnnotation {
                key: key.to_string(),
                value: "test".to_string(),
                confidence: 0.5,
                source: InferenceSource::Heuristic,
            },
        );
    }

    let anns = layer.all_annotations_for("M1");
    assert_eq!(anns.len(), 4);

    let keys_present: Vec<&str> = anns.iter().map(|a| a.key.as_str()).collect();
    for key in &keys {
        assert!(
            keys_present.contains(key),
            "Missing annotation key: {}",
            key
        );
    }
}

// ── Edge Cases ──────────────────────────────────────────────────────

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

// ── CBM Enrichment (R-43b Phase 3) ─────────────────────────────────

#[test]
fn test_enrich_from_cbm_populates_edges_and_annotations() {
    let mut bridge = new_mock_with_edges(
        vec![
            ("CallerA".to_string(), "CalleeB".to_string()),
            ("CallerC".to_string(), "CalleeD".to_string()),
        ],
        vec![
            (
                "MethodX".to_string(),
                "TargetY".to_string(),
                "reads".to_string(),
            ),
            (
                "MethodZ".to_string(),
                "TargetW".to_string(),
                "writes".to_string(),
            ),
        ],
        {
            let mut m = HashMap::new();
            m.insert(
                "Sym1".to_string(),
                SymbolImportance {
                    symbol: "Sym1".to_string(),
                    score: 0.9,
                    file: "a.ts".to_string(),
                },
            );
            m
        },
        vec![DeadCodeEntry {
            symbol: "DeadSym".to_string(),
            file: "b.ts".to_string(),
            reason: "unused".to_string(),
        }],
    );

    let mut layer = InferenceLayer::new();
    layer.enrich_from_cbm(Some(&mut bridge));

    // 2 call edges + 2 dataflow edges
    assert_eq!(layer.inferred_edges.len(), 4);

    // All CBM-derived edges have confidence 0.75 and source Cbm
    for edge in &layer.inferred_edges {
        assert_eq!(edge.confidence, 0.75);
        assert_eq!(edge.source, InferenceSource::Cbm);
    }

    // Verify call edges
    let calls: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.edge_type == InferenceEdgeType::Calls)
        .collect();
    assert_eq!(calls.len(), 2);
    assert!(
        calls
            .iter()
            .any(|e| e.from == "CallerA" && e.to == "CalleeB")
    );
    assert!(
        calls
            .iter()
            .any(|e| e.from == "CallerC" && e.to == "CalleeD")
    );

    // Verify dataflow edges (reads → DataFlowRead, writes → DataFlowWrite)
    let reads: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.edge_type == InferenceEdgeType::DataFlowRead)
        .collect();
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].from, "MethodX");
    assert_eq!(reads[0].to, "TargetY");

    let writes: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.edge_type == InferenceEdgeType::DataFlowWrite)
        .collect();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].from, "MethodZ");
    assert_eq!(writes[0].to, "TargetW");

    // Verify importance annotation
    let imp = layer.all_annotations_for("Sym1");
    assert_eq!(imp.len(), 1);
    assert_eq!(imp[0].key, "importance");
    assert_eq!(imp[0].value, "0.9");
    assert_eq!(imp[0].confidence, 0.75);
    assert_eq!(imp[0].source, InferenceSource::Cbm);

    // Verify dead code annotation
    let dead = layer.all_annotations_for("DeadSym");
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].key, "dead_code");
    assert_eq!(dead[0].value, "unused");
    assert_eq!(dead[0].confidence, 0.75);
    assert_eq!(dead[0].source, InferenceSource::Cbm);
}

#[test]
fn test_enrich_from_cbm_none_bridge_is_noop() {
    let mut layer = InferenceLayer::new();
    layer.enrich_from_cbm(None);
    assert!(layer.inferred_edges.is_empty());
    assert!(layer.annotations.is_empty());
}

#[test]
fn test_enrich_from_cbm_empty_bridge_is_noop() {
    let mut bridge = new_mock_with_edges(vec![], vec![], HashMap::new(), vec![]);
    let mut layer = InferenceLayer::new();
    layer.enrich_from_cbm(Some(&mut bridge));
    assert!(layer.inferred_edges.is_empty());
    assert!(layer.annotations.is_empty());
}

#[test]
fn test_enrich_from_cbm_does_not_duplicate_existing_edges() {
    let mut bridge = new_mock_with_edges(
        vec![("A".to_string(), "B".to_string())],
        vec![],
        HashMap::new(),
        vec![],
    );

    let mut layer = InferenceLayer::new();
    // Pre-add a structural edge (confidence 1.0) — should be preserved
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "A".to_string(),
        to: "B".to_string(),
        confidence: 1.0,
        source: InferenceSource::Structural,
    });

    layer.enrich_from_cbm(Some(&mut bridge));

    // 1 structural + 1 CBM edge
    assert_eq!(layer.inferred_edges.len(), 2);
    let structural: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.source == InferenceSource::Structural)
        .collect();
    let cbm: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.source == InferenceSource::Cbm)
        .collect();
    assert_eq!(structural.len(), 1);
    assert_eq!(cbm.len(), 1);
}

#[test]
fn test_enrich_from_cbm_direction_normalization() {
    // CBM may report direction as "WRITES", "DATAFLOW_WRITE", "write_to",
    // "READS", "DATAFLOW_READ", or even "DATAFLOW" (no direction info).
    // Anything containing "write" (case-insensitive) is a write; everything
    // else defaults to read.
    let mut bridge = new_mock_with_edges(
        vec![],
        vec![
            ("M1".to_string(), "T1".to_string(), "WRITES".to_string()),
            (
                "M2".to_string(),
                "T2".to_string(),
                "DATAFLOW_WRITE".to_string(),
            ),
            ("M3".to_string(), "T3".to_string(), "write_to".to_string()),
            ("M4".to_string(), "T4".to_string(), "READS".to_string()),
            ("M5".to_string(), "T5".to_string(), "DATAFLOW".to_string()),
        ],
        HashMap::new(),
        vec![],
    );

    let mut layer = InferenceLayer::new();
    layer.enrich_from_cbm(Some(&mut bridge));

    assert_eq!(layer.inferred_edges.len(), 5);

    let writes: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.edge_type == InferenceEdgeType::DataFlowWrite)
        .collect();
    let reads: Vec<&InferenceEdge> = layer
        .inferred_edges
        .iter()
        .filter(|e| e.edge_type == InferenceEdgeType::DataFlowRead)
        .collect();

    // "WRITES", "DATAFLOW_WRITE", "write_to" → 3 writes
    assert_eq!(writes.len(), 3, "expected 3 write edges");
    // "READS", "DATAFLOW" → 2 reads (DATAFLOW has no "write" substring)
    assert_eq!(reads.len(), 2, "expected 2 read edges");

    // Verify the write edges are from M1, M2, M3
    let write_methods: Vec<&str> = writes.iter().map(|e| e.from.as_str()).collect();
    assert!(write_methods.contains(&"M1"));
    assert!(write_methods.contains(&"M2"));
    assert!(write_methods.contains(&"M3"));
}
