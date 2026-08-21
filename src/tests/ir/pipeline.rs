// src/tests/ir/pipeline.rs
//
// Tests for R-43b: Explicit Pass Pipeline

use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;
use crate::ir::pipeline::{
    CoreIRPass, ExecutionSemanticsPass, IRPass, InferenceLayerPass, LanguageLayerPass,
    MetaLayerPass, PassContext, PassPipeline, ProgramGraphPass, ValidationPass,
};

#[test]
fn test_pipeline_new() {
    let pipeline = PassPipeline::new();
    assert_eq!(pipeline.pass_count(), 0);
}

#[test]
fn test_pipeline_add_pass() {
    let mut pipeline = PassPipeline::new();
    pipeline.add_pass(Box::new(CoreIRPass::new()));
    assert_eq!(pipeline.pass_count(), 1);
}

#[test]
fn test_pipeline_run_empty() {
    let pipeline = PassPipeline::new();
    let mut ctx = PassContext::new("source".to_string(), "file.ts".to_string(), Fidelity::Low);
    let result = pipeline.run(&mut ctx);
    assert!(result.is_ok());
}

#[test]
fn test_pipeline_run_with_passes() {
    let mut pipeline = PassPipeline::new();
    pipeline.add_pass(Box::new(CoreIRPass::new()));
    pipeline.add_pass(Box::new(LanguageLayerPass::new()));
    pipeline.add_pass(Box::new(MetaLayerPass::new()));
    pipeline.add_pass(Box::new(ExecutionSemanticsPass::new()));
    pipeline.add_pass(Box::new(ProgramGraphPass::new()));
    pipeline.add_pass(Box::new(InferenceLayerPass::new()));
    pipeline.add_pass(Box::new(ValidationPass::new()));

    let mut ctx = PassContext::new("source".to_string(), "file.ts".to_string(), Fidelity::Low);
    let result = pipeline.run(&mut ctx);
    assert!(result.is_ok());
    assert!(ctx.program_graph.is_some());
    assert!(ctx.inference_layer.is_some());
}

#[test]
fn test_core_ir_pass_empty_source() {
    let pass = CoreIRPass::new();
    let mut ctx = PassContext::new(String::new(), "file.ts".to_string(), Fidelity::Low);
    let result = pass.run(&mut ctx);
    assert!(result.is_err());
}

#[test]
fn test_pass_names() {
    assert_eq!(CoreIRPass::new().name(), "core_ir");
    assert_eq!(LanguageLayerPass::new().name(), "language_layer");
    assert_eq!(MetaLayerPass::new().name(), "meta_layer");
    assert_eq!(ExecutionSemanticsPass::new().name(), "execution_semantics");
    assert_eq!(ProgramGraphPass::new().name(), "program_graph");
    assert_eq!(InferenceLayerPass::new().name(), "inference_layer");
    assert_eq!(ValidationPass::new().name(), "validation");
}

#[test]
fn test_program_graph_pass_builds_graph() {
    let mut ctx = PassContext::new("source".to_string(), "file.ts".to_string(), Fidelity::Low);
    ctx.instructions
        .push(CoreOp::DefClass("C1".into(), "TestClass".into()));
    ctx.instructions.push(CoreOp::DefMethod(
        "C1".into(),
        "M1".into(),
        "testMethod".into(),
    ));

    let pass = ProgramGraphPass::new();
    let result = pass.run(&mut ctx);
    assert!(result.is_ok());
    assert!(ctx.program_graph.is_some());

    let graph = ctx.program_graph.unwrap();
    assert_eq!(graph.nodes.len(), 2);
}

#[test]
fn test_inference_layer_pass_builds_layer() {
    let mut ctx = PassContext::new("source".to_string(), "file.ts".to_string(), Fidelity::Low);

    let pass = InferenceLayerPass::new();
    let result = pass.run(&mut ctx);
    assert!(result.is_ok());
    assert!(ctx.inference_layer.is_some());
}

// ── R-43b Phase 3: CBM enrichment through the pass ──────────────────

#[test]
fn test_inference_layer_pass_with_cbm_enriches_layer() {
    use crate::cbm::bridge::SymbolImportance;
    use crate::cbm::bridge::test_helpers::new_mock_with_edges;
    use std::collections::HashMap;

    let bridge = new_mock_with_edges(
        vec![("CallerA".to_string(), "CalleeB".to_string())],
        vec![(
            "MethodX".to_string(),
            "TargetY".to_string(),
            "reads".to_string(),
        )],
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
        vec![],
    );

    let mut ctx = PassContext::new("source".to_string(), "file.ts".to_string(), Fidelity::Low);
    let pass = InferenceLayerPass::with_cbm(Some(bridge));
    let result = pass.run(&mut ctx);
    assert!(result.is_ok());

    let layer = ctx.inference_layer.expect("inference layer should be set");
    // 1 call edge + 1 dataflow edge
    assert_eq!(layer.inferred_edges.len(), 2);
    // importance annotation
    assert!(layer.has_annotation_key("importance"));
}

#[test]
fn test_inference_layer_pass_without_cbm_builds_empty_layer() {
    let mut ctx = PassContext::new("source".to_string(), "file.ts".to_string(), Fidelity::Low);
    let pass = InferenceLayerPass::with_cbm(None);
    let result = pass.run(&mut ctx);
    assert!(result.is_ok());

    let layer = ctx.inference_layer.expect("inference layer should be set");
    assert!(layer.inferred_edges.is_empty());
    assert!(layer.annotations.is_empty());
}
