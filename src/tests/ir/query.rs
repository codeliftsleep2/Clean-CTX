// src/tests/ir/query.rs
//
// Tests for R-43b Phase 6: Queryable IR

use crate::ir::inference_layer::{
    InferenceEdge, InferenceEdgeType, InferenceLayer, InferenceSource,
};
use crate::ir::opcodes::CoreOp;
use crate::ir::program_graph::{GraphBuilder, ProgramGraph};
use crate::ir::query::IRQueryEngine;

fn sample_graph() -> ProgramGraph {
    let instructions = vec![
        CoreOp::DefClass("C1".into(), "ServiceA".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "doA".into()),
        CoreOp::DefClass("C2".into(), "ServiceB".into()),
        CoreOp::DefMethod("C2".into(), "M2".into(), "doB".into()),
        CoreOp::Extends("C1".into(), "BaseService".into()),
    ];
    GraphBuilder::build_from_instructions(&instructions)
}

#[test]
fn test_query_engine_new() {
    let graph = sample_graph();
    let engine = IRQueryEngine::new(graph);
    assert!(engine.inference_layer().is_none());
}

#[test]
fn test_query_engine_with_inference() {
    let graph = sample_graph();
    let inference = InferenceLayer::new();
    let engine = IRQueryEngine::new(graph).with_inference(inference);
    assert!(engine.inference_layer().is_some());
}

#[test]
fn test_fan_in_empty() {
    let graph = sample_graph();
    let engine = IRQueryEngine::new(graph);
    let result = engine.get_fan_in("M1");
    assert_eq!(result.local_callers, 0);
    assert_eq!(result.total, 0);
    assert_eq!(result.confidence, 1.0);
}

#[test]
fn test_fan_in_with_inferred() {
    let graph = sample_graph();
    let mut inference = InferenceLayer::new();
    inference.add_edge(InferenceEdge {
        edge_type: InferenceEdgeType::Calls,
        from: "M1".to_string(),
        to: "M2".to_string(),
        confidence: 0.75,
        source: InferenceSource::Cbm,
    });
    let engine = IRQueryEngine::new(graph).with_inference(inference);
    let result = engine.get_fan_in("M2");
    assert_eq!(result.local_callers, 0);
    assert_eq!(result.inferred_callers, 1);
    assert_eq!(result.total, 1);
    assert!(result.confidence < 1.0);
}

#[test]
fn test_fan_out_empty() {
    let graph = sample_graph();
    let engine = IRQueryEngine::new(graph);
    assert_eq!(engine.get_fan_out("M1"), 0);
}

#[test]
fn test_find_async_methods() {
    let graph = sample_graph();
    let engine = IRQueryEngine::new(graph);
    let results = engine.find_async_methods();
    // All methods are returned with confidence = 1.0
    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.confidence == 1.0));
}

#[test]
fn test_find_subclasses() {
    let graph = sample_graph();
    let engine = IRQueryEngine::new(graph);
    let subclasses = engine.find_subclasses("BaseService");
    assert_eq!(subclasses.len(), 1);
    assert_eq!(subclasses[0].id, "C1");
}

#[test]
fn test_find_side_effects() {
    let graph = sample_graph();
    let engine = IRQueryEngine::new(graph);
    let methods = engine.find_side_effects();
    assert_eq!(methods.len(), 2);
}

#[test]
fn test_graph_access() {
    let graph = sample_graph();
    let engine = IRQueryEngine::new(graph);
    let g = engine.graph();
    assert_eq!(g.nodes.len(), 4);
}