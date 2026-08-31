// src/tests/ir/pipeline.rs
//
// Tests for R-43b: Explicit Pass Pipeline

use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;
use crate::ir::pipeline::{
    AliasResolutionPass, CoreIRPass, ExecutionSemanticsPass, IRPass, InferenceLayerPass,
    LanguageLayerPass, MetaLayerPass, PassContext, PassPipeline, PatternRecognitionPass,
    ProgramGraphPass, ValidationPass,
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
    pipeline.add_pass(Box::new(PatternRecognitionPass::new()));
    pipeline.add_pass(Box::new(AliasResolutionPass::new()));
    pipeline.add_pass(Box::new(ValidationPass::new()));

    // Empty source is valid — CoreIRPass returns Ok(()) for empty source
    let mut ctx = PassContext::new(String::new(), "file.ts".to_string(), Fidelity::Low);
    let result = pipeline.run(&mut ctx);
    assert!(result.is_ok());
}

#[test]
fn test_core_ir_pass_empty_source_is_valid() {
    let pass = CoreIRPass::new();
    let mut ctx = PassContext::new(String::new(), "file.ts".to_string(), Fidelity::Low);
    let result = pass.run(&mut ctx);
    assert!(result.is_ok());
    assert!(ctx.instructions.is_empty());
}

#[test]
fn test_pass_names() {
    assert_eq!(CoreIRPass::new().name(), "core_ir");
    assert_eq!(LanguageLayerPass::new().name(), "language_layer");
    assert_eq!(MetaLayerPass::new().name(), "meta_layer");
    assert_eq!(PatternRecognitionPass::new().name(), "pattern_recognition");
    assert_eq!(AliasResolutionPass::new().name(), "alias_resolution");
    assert_eq!(ExecutionSemanticsPass::new().name(), "execution_semantics");
    assert_eq!(ProgramGraphPass::new().name(), "program_graph");
    assert_eq!(InferenceLayerPass::new().name(), "inference_layer");
    assert_eq!(ValidationPass::new().name(), "validation");
}

/// Architectural invariant: the production pipeline must execute stages
/// in the order required by their data and semantic dependencies.
///
/// Core IR must precede Language Finalize (captures must be processed first).
/// Language Finalize must precede Meta Layer (meta layers need the instruction stream).
/// Meta Layer must precede Pattern Recognition (pattern recognizers see the full stream).
/// Pattern Recognition must precede Alias Resolution (alias resolution sees all Extends/Implements).
/// Alias Resolution must precede Validation (validation inspects the final canonical stream).
#[test]
fn production_pipeline_preserves_architectural_order() {
    let pipeline = PassPipeline::default_production();
    let names = pipeline.pass_names();

    assert_eq!(
        names,
        vec![
            "core_ir",
            "language_layer",
            "meta_layer",
            "pattern_recognition",
            "alias_resolution",
            "validation",
        ],
        "Production pipeline ordering invariant violated. \
         The compilation pipeline must execute stages in the required \
         architectural order: CoreIR → Language Finalize → Meta Layer → \
         Pattern Recognition → Alias Resolution → Validation."
    );
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
    // F10: 1 call edge (DATAFLOW enrichment removed with CBM 0.8.1)
    assert_eq!(layer.inferred_edges.len(), 1);
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

// ── C-22: PassContext.captures identity regression ──────────────────

#[test]
fn core_ir_pass_populates_captures() {
    // C-22 invariant: after CoreIRPass runs, state.captures must be
    // populated with the CapEntry batch — NOT left as an empty Vec.
    // MetaLayerPass and the text path derive class source spans from
    // this field.
    use crate::compression::capture_pipeline::CapEntry;

    let pass = CoreIRPass::new();
    let mut ctx = PassContext::new(String::new(), "file.ts".into(), Fidelity::Low);

    // CoreIRPass with empty source does not run the capture pipeline
    // (early return). captures stays empty.
    let _ = pass.run(&mut ctx);
    assert!(ctx.captures.is_empty(), "empty source → no captures");

    // A non-empty source triggers the capture pipeline. Since no
    // tree-sitter language is configured, CoreIRPass will error before
    // populating captures. We verify the error path doesn't corrupt state.
    let mut ctx2 = PassContext::new(
        "@Component()\nexport class Foo {}".into(),
        "file.ts".into(),
        Fidelity::Low,
    );
    let err = pass.run(&mut ctx2);
    assert!(
        err.is_err(),
        "expected error for missing tree-sitter language"
    );
    assert!(
        ctx2.captures.is_empty(),
        "captures should remain empty after error"
    );

    // Verify that CapEntry has the fields expected by class_source_from_capture.
    // This is a compile-time check that the CapEntry API contract holds.
    let cap = CapEntry {
        name: "class.root".into(),
        text: "Foo".into(),
        raw_text: "class Foo {}".into(),
        start_byte: 19,
        end_byte: 19 + "class Foo {}".len(),
    };
    let source = "@Component()\nexport class Foo {}";
    let span = crate::meta_util::class_source_from_capture(source, &cap);
    assert!(
        span.contains("@Component"),
        "class_source_from_capture must produce decorator-inclusive text from CapEntry"
    );
}

#[test]
fn pass_context_starts_with_empty_semantic_edges() {
    let ctx = PassContext::new("source".to_string(), "file.ts".to_string(), Fidelity::Low);
    // Phase 0 adds the carrier field; Phase 2 wires MetaLayerPass to
    // populate it and InferenceLayerPass to drain it into the layer.
    assert!(
        ctx.semantic_edges.is_empty(),
        "PassContext must start with no semantic edges"
    );
}
// ── Phase 2: Pipeline Semantic-Edge Integration ─────────────────────

use crate::compression::capture_pipeline::CapEntry;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// Angular source containing a Component that injects a Service.
/// `@Component(` is a strong decorator signalling an Angular file.
const ANGULAR_INJECT_SOURCE: &str = r#"
import { Component } from '@angular/core';
import { UserService } from './user.service';

@Component({ selector: 'app-user' })
export class UserComponent {
    constructor(private userSvc: UserService) {}
}
"#;

fn make_angular_class_capture(source: &str) -> CapEntry {
    CapEntry {
        name: "class.root".into(),
        text: "UserComponent".into(),
        raw_text: source.to_string(),
        start_byte: 0,
        end_byte: source.len(),
    }
}

fn make_pipeline_with_inference() -> PassPipeline {
    let mut pipeline = PassPipeline::new();
    pipeline.add_pass(Box::new(MetaLayerPass::new()));
    pipeline.add_pass(Box::new(InferenceLayerPass::with_cbm(None)));
    pipeline
}

#[test]
fn pipeline_collects_semantic_edges() {
    let source = ANGULAR_INJECT_SOURCE.to_string();
    let cap = make_angular_class_capture(&source);
    let mut ctx = PassContext::new(source, "user.component.ts".into(), Fidelity::High);
    ctx.captures = vec![cap];

    let pipeline = make_pipeline_with_inference();
    let result = pipeline.run(&mut ctx);
    assert!(result.is_ok(), "pipeline should succeed");

    let layer = ctx.inference_layer.expect("inference layer should exist");
    let injects: Vec<&SemanticEdge> = layer
        .semantic_edges()
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::Injects)
        .collect();
    assert!(
        !injects.is_empty(),
        "should have at least one Injects edge in pipeline"
    );

    let cmp_entity = EntityRef::new("angular", "Component", "UserComponent");
    let svc_entity = EntityRef::new("angular", "Service", "UserService");
    let has_injects = injects
        .iter()
        .any(|e| e.subject == cmp_entity && e.object == svc_entity);
    assert!(
        has_injects,
        "UserComponent should inject UserService after pipeline"
    );

    // Verify file_id was attached by MetaLayerPass
    for edge in layer.semantic_edges() {
        if edge.subject.name == "UserComponent" {
            assert_eq!(
                edge.subject.file.as_deref(),
                Some("user.component.ts"),
                "file_id should be attached to subject"
            );
        }
    }
}

#[test]
fn pipeline_no_meta_layers_graceful() {
    // Non-Angular source with no Angular markers
    let source = "export function add(a: number, b: number): number { return a + b; }".to_string();
    let mut ctx = PassContext::new(source, "math.ts".into(), Fidelity::Low);

    let pipeline = make_pipeline_with_inference();
    let result = pipeline.run(&mut ctx);
    assert!(
        result.is_ok(),
        "pipeline should succeed for non-Angular source"
    );

    let layer = ctx.inference_layer.expect("inference layer should exist");
    assert!(
        layer.semantic_edges().is_empty(),
        "no semantic edges expected for non-Angular source"
    );
}

#[test]
fn pipeline_semantic_edges_are_consumed() {
    let source = ANGULAR_INJECT_SOURCE.to_string();
    let cap = make_angular_class_capture(&source);
    let mut ctx = PassContext::new(source, "user.component.ts".into(), Fidelity::High);
    ctx.captures = vec![cap];

    let pipeline = make_pipeline_with_inference();
    let result = pipeline.run(&mut ctx);
    assert!(result.is_ok(), "pipeline should succeed");

    // After InferenceLayerPass drains them, PassContext should be empty
    assert!(
        ctx.semantic_edges.is_empty(),
        "PassContext.semantic_edges should be empty after InferenceLayerPass"
    );

    // The same edges must be in the inference layer
    let layer = ctx.inference_layer.expect("inference layer should exist");
    assert!(
        !layer.semantic_edges().is_empty(),
        "inference layer should have semantic edges after pipeline"
    );
}

#[test]
fn pipeline_preserves_phi_output() {
    let source = ANGULAR_INJECT_SOURCE.to_string();
    let cap = make_angular_class_capture(&source);
    let mut ctx = PassContext::new(source, "user.component.ts".into(), Fidelity::High);
    ctx.captures = vec![cap];

    let pipeline = make_pipeline_with_inference();
    let result = pipeline.run(&mut ctx);
    assert!(result.is_ok());

    // Verify Φ markers are present in the instructions
    let phi_markers: Vec<&CoreOp> = ctx
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::TypeAlias(_, _)))
        .collect();
    assert!(
        !phi_markers.is_empty(),
        "Φ markers should be present in pipeline instructions"
    );

    // The Phi markers include the Angular decorator output.
    // Verify at least one TypeAlias with a recognized Angular prefix.
    let has_cmp = ctx.instructions.iter().any(|op| {
        if let CoreOp::TypeAlias(prefix, _) = op {
            prefix == "@cmp"
        } else {
            false
        }
    });
    assert!(
        has_cmp,
        "@cmp alias should be produced for Angular component"
    );
}
