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

/// The Inference Layer holds all non-deterministic, derived, or estimated
/// data about the IR. This layer is NEVER serialized into the core IR wire
/// format. It is recomputed on demand and lives only in memory.
#[derive(Debug, Clone, Default)]
pub struct InferenceLayer {
    /// Inferred edges with confidence scores
    pub inferred_edges: Vec<InferenceEdge>,
    /// Per-symbol annotations (importance, dead code, blast radius)
    pub annotations: HashMap<String, Vec<InferenceAnnotation>>,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InferenceEdgeType {
    Calls,
    DataFlowRead,
    DataFlowWrite,
    Injects,
    Extends,
    Implements,
}

/// Source of an inference.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Get all edges with confidence above a threshold.
    pub fn edges_with_confidence(&self, min_confidence: f64) -> Vec<&InferenceEdge> {
        self.inferred_edges.iter()
            .filter(|e| e.confidence >= min_confidence)
            .collect()
    }

    /// Get annotations for a symbol with confidence above a threshold.
    pub fn annotations_for(
        &self,
        symbol: &str,
        min_confidence: f64,
    ) -> Vec<&InferenceAnnotation> {
        self.annotations.get(symbol)
            .map(|anns| anns.iter().filter(|a| a.confidence >= min_confidence).collect())
            .unwrap_or_default()
    }

    /// Get all annotations for a symbol (all confidence levels).
    pub fn all_annotations_for(&self, symbol: &str) -> Vec<&InferenceAnnotation> {
        self.annotations.get(symbol)
            .map(|anns| anns.iter().collect())
            .unwrap_or_default()
    }

    /// Check if there are any annotations with a given key.
    pub fn has_annotation_key(&self, key: &str) -> bool {
        self.annotations.values().any(|anns| anns.iter().any(|a| a.key == key))
    }
}

#[cfg(test)]
#[path = "../tests/ir/inference_layer.rs"]
mod tests;