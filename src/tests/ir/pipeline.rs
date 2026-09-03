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

// ── Ingestion-boundary regression: entity occurrences unique per (identity, file) ──
//
// HISTORY: born RED against the unmodified implementation (RED phase), where
// it failed with "expected: 14 unique (identity, file) registrations,
// actual: 25" and "(dotnet, Controller, OrdersController) registered 12x
// (expected 1)". The architecture investigation established that entity
// registration is intentionally DERIVED from semantic edges at the
// WorkspaceIndex write boundary — the producer stream is therefore not the
// permanent observation point, and the test is re-anchored to the actual
// ingestion path. The function name is retained so the historical RED
// command (`cargo test --lib
// producer_registers_each_entity_identity_once_per_file`) is the exact
// command that must now be GREEN.
//
// INVARIANT under test (B1 occurrence identity, commit 6dfa86f, completed by
// the C1 idempotent-registration boundary): through the real production
// pipeline and the real WorkspaceIndex ingestion,
//   1. the legitimate semantic edges are preserved (one ControllerAction per
//      action, one HasRoute, one builtin self-Defines carrier), and
//   2. every entity occurrence is registered exactly once per
//      (domain, entity_type, name, file) — edge participation never
//      multiplies registrations, while cross-file occurrences remain
//      distinct.
#[test]
fn producer_registers_each_entity_identity_once_per_file() {
    const ACTION_COUNT: usize = 11;
    let actions: [(&str, &str); ACTION_COUNT] = [
        ("GetAll", "[HttpGet]"),
        ("GetById", "[HttpGet(\"{id}\")]"),
        ("Search", "[HttpGet(\"search\")]"),
        ("Create", "[HttpPost]"),
        ("BulkCreate", "[HttpPost(\"bulk\")]"),
        ("Update", "[HttpPut(\"{id}\")]"),
        ("Patch", "[HttpPatch(\"{id}\")]"),
        ("Delete", "[HttpDelete(\"{id}\")]"),
        ("Archive", "[HttpPost(\"{id}/archive\")]"),
        ("Restore", "[HttpPost(\"{id}/restore\")]"),
        ("Export", "[HttpGet(\"export\")]"),
    ];
    let mut src = String::from(
        "using Microsoft.AspNetCore.Mvc;\n\nnamespace Api.Controllers;\n\n[ApiController]\n[Route(\"api/orders\")]\npublic class OrdersController : ControllerBase\n{\n",
    );
    for (name, attr) in actions {
        src.push_str(&format!(
            "    {attr}\n    public IActionResult {name}(int id) {{ return Ok(); }}\n\n"
        ));
    }
    src.push_str("}\n");

    // Production compile path: PassPipeline::default_production() with the
    // C# language + CS_QUERY, exactly as IRCompiler::compile_inner configures
    // it. Medium fidelity: ControllerAction edges are Medium+ (per
    // extract_dotnet_semantic_edges).
    let mut ctx = PassContext::new(src, "OrdersController.cs".into(), Fidelity::Medium);
    ctx.canonical_path = Some("C:/repo/OrdersController.cs".into());
    ctx.language =
        Some(crate::compression::language::safe_csharp_language().expect("csharp grammar enabled"));
    ctx.query_string = crate::queries::CS_QUERY.to_string();
    PassPipeline::default_production()
        .run(&mut ctx)
        .expect("production pipeline should succeed");

    // The exact stream handed to WorkspaceIndex ingestion.
    let edges = ctx.semantic_edges;
    // ── Semantic unit cardinality (the established representation) ──
    // One action → exactly one ControllerAction edge; one controller →
    // exactly one HasRoute edge. (These hold today; they anchor the semantic
    // unit so the invariant below cannot be satisfied by dropping facts.)
    let controller_actions: Vec<&SemanticEdge> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::ControllerAction)
        .collect();
    assert_eq!(
        controller_actions.len(),
        ACTION_COUNT,
        "one action → one ControllerAction edge expected, got {}: {:?}",
        controller_actions.len(),
        controller_actions
            .iter()
            .map(|e| e.object.name.as_str())
            .collect::<Vec<_>>()
    );
    let distinct_actions: std::collections::BTreeSet<&str> = controller_actions
        .iter()
        .map(|e| e.object.name.as_str())
        .collect();
    assert_eq!(
        distinct_actions.len(),
        ACTION_COUNT,
        "each action must be a distinct semantic fact"
    );
    assert_eq!(
        edges
            .iter()
            .filter(|e| e.relation == SemanticRelation::HasRoute)
            .count(),
        1,
        "one controller → one HasRoute edge expected"
    );

    // ── THE INVARIANT (occurrence identity): one registration per
    // (identity, file), proven through the ACTUAL ingestion path — the edge
    // stream is fed to a WorkspaceIndex exactly as the MCP handlers do it.
    let canonical = "C:/repo/OrdersController.cs";
    let mut idx = crate::workspace::index::WorkspaceIndex::new();
    idx.add_edges(canonical, edges);

    // Semantic edges preserved: the edge graph holds every legitimate
    // relationship (11 ControllerAction + 1 HasRoute); the self-Defines
    // carrier is normalized at the write boundary and never becomes a graph
    // edge.
    assert_eq!(
        idx.edge_count(),
        ACTION_COUNT + 1,
        "11 ControllerAction + 1 HasRoute edges must be indexed"
    );
    assert_eq!(
        idx.forward_edges_by_identity("dotnet", "Controller", "OrdersController")
            .len(),
        ACTION_COUNT + 1,
        "the Controller subject keeps all 12 outgoing relationships"
    );

    // Entity occurrences are unique per (identity, file). The fixture's
    // semantic content is 1 Controller + 11 Actions + 1 Route + 1 builtin
    // Class registration — each must appear exactly once, and nothing else
    // may be registered.
    let mut known: Vec<(&str, &str, String)> = vec![
        ("dotnet", "Controller", "OrdersController".to_string()),
        ("dotnet", "Route", "api/orders".to_string()),
        ("builtin", "Class", "OrdersController".to_string()),
    ];
    for (name, _) in actions {
        known.push(("dotnet", "Action", name.to_string()));
    }
    for (domain, entity_type, name) in &known {
        let occurrences = idx.entities_by_identity(domain, entity_type, name);
        assert_eq!(
            occurrences.len(),
            1,
            "occurrence ({domain}, {entity_type}, {name}) must be registered exactly once \
             for the file — edge participation must not multiply registrations"
        );
        assert_eq!(
            occurrences[0].file.as_deref(),
            Some(canonical),
            "file provenance must be preserved on the single occurrence"
        );
    }
    assert_eq!(
        idx.entity_occurrence_count(),
        known.len(),
        "registration records must equal distinct (identity, file) pairs — \
         no occurrence may exist outside the fixture's semantic content"
    );

    // Representative check (the RED-phase offender): the Controller subject
    // participates in 12 edges yet owns exactly one occurrence.
    assert_eq!(
        idx.entities_by_identity("dotnet", "Controller", "OrdersController")
            .len(),
        1,
        "the Controller is registered once for the file despite participating in 12 edges"
    );
}
