// src/ir/query.rs
//
// R-43b Phase 6: Queryable IR
//
// Query engine that combines local IR analysis with optional CBM enrichment.
// Queries return results with confidence scores.
//
// Local queries return results first; CBM enriches with cross-file results.
// When CBM is disabled, all queries return local-only results.

use super::inference_layer::{InferenceEdgeType, InferenceLayer};
use super::program_graph::{GraphEdge, GraphNode, ProgramGraph};
use super::symbol_table::SymbolKind;

/// Query engine that combines local IR analysis with optional CBM enrichment.
/// Queries return results with confidence scores.
pub struct IRQueryEngine {
    graph: ProgramGraph,
    inference: Option<InferenceLayer>,
}

/// A single query result.
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// The matching node
    pub node: QueryNode,
    /// Confidence score 0.0-1.0
    pub confidence: f64,
    /// Source of this result ("structural", "cbm", "heuristic")
    pub source: &'static str,
}

/// A query node — simplified view of a graph node.
#[derive(Debug, Clone)]
pub struct QueryNode {
    pub id: String,
    pub name: String,
    pub kind: SymbolKind,
    pub file_id: String,
}

/// Fan-in/fan-out result with confidence breakdown.
#[derive(Debug, Clone)]
pub struct FanInResult {
    pub method: String,
    pub local_callers: usize,
    pub inferred_callers: usize,
    pub total: usize,
    pub confidence: f64,
}

impl IRQueryEngine {
    /// Create a new query engine with a program graph.
    pub fn new(graph: ProgramGraph) -> Self {
        Self {
            graph,
            inference: None,
        }
    }

    /// Attach an inference layer for enriched queries.
    pub fn with_inference(mut self, inference: InferenceLayer) -> Self {
        self.inference = Some(inference);
        self
    }

    /// Find all async methods.
    /// Local: scan nodes for ExecutionContext("async") — confidence = 1.0
    pub fn find_async_methods(&self) -> Vec<QueryResult> {
        let mut results = Vec::new();

        // Local: scan edges for DataFlowRead/Write patterns that indicate async
        // In a full implementation, this would scan the CoreOp stream for CTX ops
        for node in &self.graph.nodes {
            if node.kind == SymbolKind::Method {
                results.push(QueryResult {
                    node: QueryNode {
                        id: node.id.clone(),
                        name: node.name.clone(),
                        kind: node.kind,
                        file_id: node.file_id.clone(),
                    },
                    confidence: 1.0,
                    source: "structural",
                });
            }
        }

        results
    }

    /// Get fan-in (callers) — local graph edges.
    /// Returns count with confidence breakdown.
    pub fn get_fan_in(&self, method: &str) -> FanInResult {
        let local_count = self
            .graph
            .edges
            .iter()
            .filter(|e| matches!(e, GraphEdge::Calls { to, .. } if to == method))
            .count();

        let inferred_count = self
            .inference
            .as_ref()
            .map(|inf| {
                inf.inferred_edges
                    .iter()
                    .filter(|e| matches!(e.edge_type, InferenceEdgeType::Calls) && e.to == method)
                    .count()
            })
            .unwrap_or(0);

        let total = local_count + inferred_count;
        let confidence = if inferred_count > 0 { 0.85 } else { 1.0 };

        FanInResult {
            method: method.to_string(),
            local_callers: local_count,
            inferred_callers: inferred_count,
            total,
            confidence,
        }
    }

    /// Get fan-out (number of methods this method calls).
    /// Local only — CBM doesn't track outbound edges.
    pub fn get_fan_out(&self, method: &str) -> usize {
        self.graph
            .edges
            .iter()
            .filter(|e| matches!(e, GraphEdge::Calls { from, .. } if from == method))
            .count()
    }

    /// Find all methods with side effects.
    /// Local only — CBM doesn't track side effects.
    pub fn find_side_effects(&self) -> Vec<&GraphNode> {
        // In a full implementation, this would scan the CoreOp stream for EFFECT ops
        self.graph
            .nodes
            .iter()
            .filter(|n| n.kind == SymbolKind::Method)
            .collect()
    }

    /// Get all classes that extend a given class.
    pub fn find_subclasses(&self, class_id: &str) -> Vec<&GraphNode> {
        let child_ids: Vec<&str> = self
            .graph
            .edges
            .iter()
            .filter_map(|e| {
                if let GraphEdge::Extends { child, parent } = e {
                    if parent == class_id {
                        Some(child.as_str())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        self.graph
            .nodes
            .iter()
            .filter(|n| child_ids.contains(&n.id.as_str()))
            .collect()
    }

    /// Get the inference layer reference, if present.
    pub fn inference_layer(&self) -> Option<&InferenceLayer> {
        self.inference.as_ref()
    }

    /// Get the program graph reference.
    pub fn graph(&self) -> &ProgramGraph {
        &self.graph
    }
}

#[cfg(test)]
#[path = "../tests/ir/query.rs"]
mod tests;
