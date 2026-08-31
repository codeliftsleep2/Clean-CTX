// src/ir/inference_layer.rs
//
// R-43b Phase 3: Inference Layer
//
// The Inference Layer holds all non-deterministic, derived, or estimated
// data about the IR. This layer is NEVER serialized into the core IR wire
// format. It is recomputed on demand and lives only in memory.
//
// ── Facts vs Inferences ──────────────────────────────────────────
// CoreOp stream = pure facts (structural, deterministic, replayable)
// InferenceLayer = derived analysis (estimated, non-deterministic, ephemeral)
//
// ── Confidence Scores ────────────────────────────────────────────
// 1.0  = structural fact (from tree-sitter parsing)
// 0.75 = CBM-derived (cross-file call edge, importance, dead code)
// 0.5  = heuristic-based (pattern matching, estimation)
// 0.25 = AI-generated (LLM reasoning, subject to hallucination)

use std::collections::HashMap;

use crate::cbm::bridge::GraphBridge;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge};

/// The Inference Layer holds all non-deterministic, derived, or estimated
/// data about the IR. This layer is NEVER serialized into the core IR wire
/// format. It is recomputed on demand and lives only in memory.
#[derive(Debug, Clone, Default)]
pub struct InferenceLayer {
    /// Inferred edges with confidence scores
    pub inferred_edges: Vec<InferenceEdge>,
    /// Meta-layer semantic edges (structural facts, implicit confidence 1.0)
    pub semantic_edges: Vec<SemanticEdge>,
    /// Per-symbol annotations (importance, dead code, blast radius)
    pub annotations: HashMap<String, Vec<InferenceAnnotation>>,
}

/// All edges (inferred + semantic) touching a single entity.
///
/// Returned by [`InferenceLayer::all_edges_for`]. The two lists are kept
/// separate because the edge kinds are intentionally distinct: inferred
/// edges are confidence-scored derivations, semantic edges are structural
/// meta-layer facts (implicit confidence 1.0).
#[derive(Debug, Clone)]
pub struct EdgeSet<'a> {
    /// CBM/structural/derived edges (explicit confidence + source).
    pub inferred: Vec<&'a InferenceEdge>,
    /// Meta-layer structural facts (implicit confidence 1.0).
    pub semantic: Vec<&'a SemanticEdge>,
}

/// An inferred edge between two symbols.
#[derive(Debug, Clone)]
pub struct InferenceEdge {
    /// Type of edge
    pub edge_type: InferenceEdgeType,
    /// Source symbol ID
    pub from: String,
    /// Target symbol ID
    pub to: String,
    /// Confidence score 0.0-1.0
    /// 1.0 = structural fact, 0.75 = CBM-derived, 0.5 = heuristic
    pub confidence: f64,
    /// Source of this inference
    pub source: InferenceSource,
}

/// Types of inferred edges.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InferenceEdgeType {
    Calls,
    DataFlowRead,
    DataFlowWrite,
    Injects,
    Extends,
    Implements,
}

/// Source of an inference.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InferenceSource {
    /// From tree-sitter parsing (confidence = 1.0)
    Structural,
    /// From CBM knowledge graph (confidence = 0.75)
    Cbm,
    /// From heuristic pattern matching (confidence = 0.5)
    Heuristic,
    /// From AI-generated reasoning (confidence = configurable)
    AiGenerated,
}

/// A per-symbol annotation with confidence.
#[derive(Debug, Clone)]
pub struct InferenceAnnotation {
    /// Annotation key ("importance", "dead_code", "blast_radius")
    pub key: String,
    /// Serialized annotation value
    pub value: String,
    /// Confidence score 0.0-1.0
    pub confidence: f64,
    /// Source of this inference
    pub source: InferenceSource,
}

impl InferenceLayer {
    /// Build an empty inference layer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an inferred edge.
    pub fn add_edge(&mut self, edge: InferenceEdge) {
        self.inferred_edges.push(edge);
    }

    /// Add an annotation for a symbol.
    pub fn add_annotation(&mut self, symbol: String, annotation: InferenceAnnotation) {
        self.annotations.entry(symbol).or_default().push(annotation);
    }

    /// Add a semantic edge (meta-layer structural fact, confidence 1.0).
    pub fn add_semantic_edge(&mut self, edge: SemanticEdge) {
        self.semantic_edges.push(edge);
    }

    /// Get all semantic edges (references into the layer's store).
    pub fn semantic_edges(&self) -> Vec<&SemanticEdge> {
        self.semantic_edges.iter().collect()
    }

    /// Get semantic edges whose subject or object equals the given entity
    /// IDENTITY. Entity identity is (domain, entity_type, name) -- `file`
    /// is excluded, so a query built with a different file id still matches
    /// (plan U1/U2).
    pub fn semantic_edges_for(&self, entity: &EntityRef) -> Vec<&SemanticEdge> {
        self.semantic_edges
            .iter()
            .filter(|e| e.subject == *entity || e.object == *entity)
            .collect()
    }

    /// All edges touching an entity: inferred (CBM/structural, matched by
    /// symbol ID) plus semantic (meta-layer facts, matched by identity).
    pub fn all_edges_for(&self, entity: &EntityRef) -> EdgeSet<'_> {
        EdgeSet {
            inferred: self
                .inferred_edges
                .iter()
                .filter(|e| e.from == entity.name || e.to == entity.name)
                .collect(),
            semantic: self.semantic_edges_for(entity),
        }
    }

    /// Get all edges with confidence above a threshold.
    pub fn edges_with_confidence(&self, min_confidence: f64) -> Vec<&InferenceEdge> {
        self.inferred_edges
            .iter()
            .filter(|e| e.confidence >= min_confidence)
            .collect()
    }

    /// Get annotations for a symbol with confidence above a threshold.
    pub fn annotations_for(&self, symbol: &str, min_confidence: f64) -> Vec<&InferenceAnnotation> {
        self.annotations
            .get(symbol)
            .map(|anns| {
                anns.iter()
                    .filter(|a| a.confidence >= min_confidence)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all annotations for a symbol (all confidence levels).
    pub fn all_annotations_for(&self, symbol: &str) -> Vec<&InferenceAnnotation> {
        self.annotations
            .get(symbol)
            .map(|anns| anns.iter().collect())
            .unwrap_or_default()
    }

    /// Check if there are any annotations with a given key.
    pub fn has_annotation_key(&self, key: &str) -> bool {
        self.annotations
            .values()
            .any(|anns| anns.iter().any(|a| a.key == key))
    }

    /// Enrich this layer with CBM knowledge-graph data.
    ///
    /// R-43b Phase 3: Consumed by `InferenceLayerPass` (Pass 6) after local
    /// graph construction. A `None` bridge or an unavailable bridge is not
    /// an error — enrichment is simply not applicable and `Ok(())` is
    /// returned (invariant C2).
    ///
    /// AUDIT FIX (F11): CBM failures now propagate as [`CbmError::Err`].
    /// The layer NEVER converts a CBM failure into empty data — callers own
    /// the failure policy. All CBM-derived edges and annotations carry
    /// `confidence = 0.75` and `source = InferenceSource::Cbm`
    /// (invariant C3). This method never writes into the core `CoreOp`
    /// instruction stream (invariant C1).
    ///
    /// Populates:
    ///   - Cross-file CALLS edges → `inferred_edges`
    ///   - Symbol importance → `annotations["importance"]`
    ///   - Dead code → `annotations["dead_code"]`
    ///
    /// DATAFLOW edges are NOT populated: the edge type does not exist in
    /// CBM 0.8.1 (see the F10 limitation note in `GraphBridge`).
    pub fn enrich_from_cbm(
        &mut self,
        bridge: Option<&mut GraphBridge>,
    ) -> Result<(), crate::cbm::client::CbmError> {
        let bridge = match bridge {
            Some(b) if b.is_available() => b,
            // Not applicable — not a failure.
            _ => return Ok(()),
        };

        // Cross-file CALLS edges (confidence = 0.75)
        for (caller, callee) in bridge.get_call_edges()? {
            self.inferred_edges.push(InferenceEdge {
                edge_type: InferenceEdgeType::Calls,
                from: caller,
                to: callee,
                confidence: 0.75,
                source: InferenceSource::Cbm,
            });
        }

        // Symbol importance (confidence = 0.75)
        for (name, info) in bridge.get_symbol_importance_mut()? {
            self.annotations
                .entry(name)
                .or_default()
                .push(InferenceAnnotation {
                    key: "importance".into(),
                    value: info.score.to_string(),
                    confidence: 0.75,
                    source: InferenceSource::Cbm,
                });
        }

        // Dead code (confidence = 0.75)
        for entry in bridge.get_dead_code()? {
            self.annotations
                .entry(entry.symbol.clone())
                .or_default()
                .push(InferenceAnnotation {
                    key: "dead_code".into(),
                    value: entry.reason.clone(),
                    confidence: 0.75,
                    source: InferenceSource::Cbm,
                });
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../tests/ir/inference_layer.rs"]
mod tests;
