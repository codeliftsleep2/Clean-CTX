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

// ── CBM Enrichment (R-43b Phase 3) ─────────────────────────────────

#[test]
fn test_enrich_from_cbm_populates_edges_and_annotations() {
    let mut bridge = new_mock_with_edges(
        vec![
            ("CallerA".to_string(), "CalleeB".to_string()),
            ("CallerC".to_string(), "CalleeD".to_string()),
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
    layer
        .enrich_from_cbm(Some(&mut bridge))
        .expect("mock-backed enrichment must succeed");

    // F10: only CALLS edges exist now (DATAFLOW removed with CBM 0.8.1).
    assert_eq!(layer.inferred_edges.len(), 2);

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
    // Not applicable (no bridge) is Ok, not an error — invariant C2.
    layer
        .enrich_from_cbm(None)
        .expect("None bridge is not a failure");
    assert!(layer.inferred_edges.is_empty());
    assert!(layer.annotations.is_empty());
}

#[test]
fn test_enrich_from_cbm_empty_bridge_is_noop() {
    let mut bridge = new_mock_with_edges(vec![], HashMap::new(), vec![]);
    let mut layer = InferenceLayer::new();
    layer
        .enrich_from_cbm(Some(&mut bridge))
        .expect("mock-backed enrichment must succeed");
    assert!(layer.inferred_edges.is_empty());
    assert!(layer.annotations.is_empty());
}

/// F11 PROPAGATION: when the CBM client fails (mock has no subprocess and
/// no cached entries), `enrich_from_cbm` must return Err — it must NEVER
/// convert the failure into an empty-but-successful enrichment.
#[test]
fn test_enrich_from_cbm_propagates_cbm_failure() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;

    // new_mock_empty(): status Available, client None, cache empty → every
    // intel query hits the missing client and returns Err.
    let mut bridge = new_mock_empty();
    assert!(
        bridge.is_available(),
        "fixture sanity: mock reports Available"
    );

    let mut layer = InferenceLayer::new();
    let result = layer.enrich_from_cbm(Some(&mut bridge));
    assert!(
        result.is_err(),
        "F11 REGRESSION: enrich_from_cbm swallowed a CBM failure into empty data"
    );
    assert!(
        layer.inferred_edges.is_empty() && layer.annotations.is_empty(),
        "a failed enrichment must not leave partial data behind"
    );
}

#[test]
fn test_enrich_from_cbm_does_not_duplicate_existing_edges() {
    let mut bridge = new_mock_with_edges(
        vec![("A".to_string(), "B".to_string())],
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

    layer
        .enrich_from_cbm(Some(&mut bridge))
        .expect("mock-backed enrichment must succeed");

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

// ── Semantic Edges (Meta-Layer Facts) ─────────────────────────────────

#[test]
fn test_semantic_edges_default_empty() {
    let layer = InferenceLayer::new();
    assert!(layer.semantic_edges.is_empty());
    assert!(layer.semantic_edges().is_empty());
}

#[test]
fn test_add_semantic_edge() {
    let mut layer = InferenceLayer::new();
    layer.add_semantic_edge(SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Component", "UserComponent"),
        object: EntityRef::new("angular", "Service", "UserService"),
        layer: "angular",
        call_evidence: None,
    });
    assert_eq!(layer.semantic_edges.len(), 1);
    assert_eq!(layer.semantic_edges().len(), 1);
}

#[test]
fn test_semantic_edges_for_matches_identity_ignores_file() {
    let mut layer = InferenceLayer::new();
    layer.add_semantic_edge(SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Component", "UserComponent")
            .with_file("a.ts".to_string()),
        object: EntityRef::new("angular", "Service", "UserService"),
        layer: "angular",
        call_evidence: None,
    });

    // Query built with a DIFFERENT file id but the same identity -> match.
    let query =
        EntityRef::new("angular", "Component", "UserComponent").with_file("b.ts".to_string());
    assert_eq!(layer.semantic_edges_for(&query).len(), 1);

    // Different name -> no match.
    let other = EntityRef::new("angular", "Component", "AdminComponent");
    assert!(layer.semantic_edges_for(&other).is_empty());
}

#[test]
fn test_all_edges_for_returns_both_kinds() {
    let mut layer = InferenceLayer::new();
    layer.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "UserComponent".to_string(),
        to: "UserService".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    });
    layer.add_semantic_edge(SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Component", "UserComponent"),
        object: EntityRef::new("angular", "Service", "UserService"),
        layer: "angular",
        call_evidence: None,
    });

    let entity = EntityRef::new("angular", "Component", "UserComponent");
    let bundle = layer.all_edges_for(&entity);
    assert_eq!(bundle.inferred.len(), 1);
    assert_eq!(bundle.semantic.len(), 1);
}
// ── Phase 6: CBM Coexistence ──────────────────────────────────────────
//
// Verify that CBM-derived inferred edges and meta-layer semantic edges
// coexist correctly through all_edges_for(). Uses the production path:
//   new_mock_with_edges() → enrich_from_cbm() for inferred edges
//   add_semantic_edge()   for semantic edges
//   all_edges_for()       for combined retrieval

#[test]
fn test_cbm_and_semantic_edges_coexist_in_all_edges_for() {
    // Arrange: create a mock CBM bridge with call edges.
    let mut bridge = new_mock_with_edges(
        vec![("UserController".to_string(), "UserService".to_string())],
        HashMap::new(),
        vec![],
    );

    let mut layer = InferenceLayer::new();

    // Act 1: enrich from CBM — populates inferred_edges via production path.
    layer
        .enrich_from_cbm(Some(&mut bridge))
        .expect("mock-backed enrichment must succeed");

    // Act 2: add a semantic edge from Spring meta-layer extraction.
    layer.add_semantic_edge(SemanticEdge {
        relation: SemanticRelation::EndpointMapsTo,
        subject: EntityRef::new("spring", "Controller", "UserController"),
        object: EntityRef::new("spring", "Endpoint", "GET /api/users"),
        layer: "spring",
        call_evidence: None,
    });

    // Assert: enrichment produced exactly one inferred edge.
    assert_eq!(
        layer.inferred_edges.len(),
        1,
        "enrich_from_cbm should load 1 call edge"
    );

    // Assert: layer holds exactly one semantic edge.
    assert_eq!(layer.semantic_edges.len(), 1, "should have 1 semantic edge");

    // Act 3: query all_edges_for an entity whose name matches the CBM
    // edge (from="UserController") AND whose identity matches the semantic
    // edge subject (domain="spring", entity_type="Controller", name="UserController").
    let entity = EntityRef::new("spring", "Controller", "UserController");
    let bundle = layer.all_edges_for(&entity);

    // Assert: both edge categories are present — neither suppresses the other.
    assert_eq!(
        bundle.inferred.len(),
        1,
        "CBM inferred edge must be present in all_edges_for"
    );
    assert_eq!(
        bundle.semantic.len(),
        1,
        "semantic edge must be present in all_edges_for"
    );

    // Verify inferred edge content
    let inferred = bundle.inferred[0];
    assert_eq!(inferred.from, "UserController");
    assert_eq!(inferred.to, "UserService");
    assert_eq!(inferred.edge_type, InferenceEdgeType::Calls);
    assert_eq!(inferred.confidence, 0.75);
    assert_eq!(inferred.source, InferenceSource::Cbm);

    // Verify semantic edge content
    let semantic = bundle.semantic[0];
    assert_eq!(semantic.relation, SemanticRelation::EndpointMapsTo);
    assert_eq!(semantic.subject.name, "UserController");
    assert_eq!(semantic.object.name, "GET /api/users");
    assert_eq!(semantic.layer, "spring");
}
