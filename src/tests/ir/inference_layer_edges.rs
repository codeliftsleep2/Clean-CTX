// Sibling test module for a `#[path]`-loaded test file that exceeded the
// 615-line active-file size ceiling.
//
// Declared from the parent test file with `#[path = "<file>.rs"] mod <name>;`
// -- the same nested-`#[path]` idiom already used by `src/tests/cbm/e2e.rs` --
// so this module is a DESCENDANT of that test module and `use super::*`
// inherits its entire scope (imports, helpers, fixtures). Nothing needed to be
// widened or re-imported.
//
// Pure relocation: the tests below are byte-for-byte the previously inlined
// implementations.

use super::*;

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
