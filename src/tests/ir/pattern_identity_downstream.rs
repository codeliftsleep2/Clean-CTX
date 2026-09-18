// src/tests/ir/pattern_identity_downstream.rs
//
// F2 — PATTERN IDENTITY PRESERVATION (downstream contracts).
//
// The producer-side contract lives in `src/tests/ir/pattern_identity.rs`. These
// regressions pin WHY it matters, each against a REAL production compilation:
// when a consumptive pattern used to delete `DefMethod`, the recognized method
// disappeared from every downstream representation while the pattern op kept
// referring to it:
//
//     hierarchical MethodNode  → no method node at all
//     rendered `M <name>`      → no method line
//     `UnitTable`              → `apply_edit` could not target the method
//     semantic registration    → no declaring occurrence, so no entity
//     `Calls` subject          → a caller id that resolves to no name
//
// Nothing in this file repairs those consumers. They are downstream victims:
// the fix belongs at the pattern producer/transform boundary, and after it they
// receive the declaration they already know how to consume.

use crate::compression::Fidelity;
use crate::edit::locate::UnitTable;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::hierarchical::{HierarchicalIR, MethodNode, ir_to_hierarchical};
use crate::ir::layers::PatternRecognizer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::ir::semantic_projection::{project_calls, project_method_declarations};

/// A TypeScript constructor with a DI parameter property: a real production
/// shape that the CTOR classification recognizes.
const TS_CTOR: &str = r#"
export class Panel {
    constructor(private greeter: Greeter) {}

    greet(): string {
        return 'hi';
    }
}
"#;

/// A Promise-returning, non-`async`, zero-parameter method: a real production
/// shape that the PROMISE classification recognizes.
const TS_PROMISE: &str = r#"
export class DataService {
    load(): Promise<string> {}
}
"#;

/// A Promise-returning method whose body contains a call fact. The IRPAT-001
/// decline guard protects this shape in production (a `Call` for the method is
/// adjacent to the pattern span), so it is a control for the call contract.
const TS_PROMISE_WITH_CALL: &str = r#"
export class DataService {
    load(): Promise<string> {
        return Promise.resolve('x');
    }
}
"#;

// ─ Helpers ────────────────────────────────────────────────────────────

/// Compile TypeScript through the real production compiler configuration.
fn compile_ts(source: &str, file_id: &str, fidelity: Fidelity) -> CompiledIR {
    let (language, query_string) =
        crate::compression::language::language_for_extension("ts").expect("TS language");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(source, file_id, language, query_string, fidelity, None)
        .unwrap_or_else(|e| panic!("{file_id} should compile: {e}"))
}

/// The method id the flat stream registers for a declaration named `name`.
fn method_id(instructions: &[CoreOp], name: &str) -> Option<String> {
    instructions.iter().find_map(|op| match op {
        CoreOp::DefMethod(_, mid, n) if n == name => Some(mid.clone()),
        _ => None,
    })
}

/// The method node for a declaration named `name`, if the hierarchical
/// projection produced one.
fn method_node<'a>(hir: &'a HierarchicalIR, name: &str) -> Option<&'a MethodNode> {
    hir.classes
        .iter()
        .flat_map(|class| &class.methods)
        .find(|method| method.name == name)
}

// ─ RED-F2-7 / production shape: the projection keeps the method ───────

#[test]
fn production_pattern_output_projects_to_a_retained_method_node() {
    // The mandatory production-shape regression: the stream under test comes
    // from the REAL pipeline (`compile` → CoreIRPass → LanguageLayerPass →
    // MetaLayerPass → PatternRecognitionPass → ValidationPass), not from a
    // hand-constructed idealization, and it is then projected through
    // `ir_to_hierarchical` — the exact boundary where the method used to
    // vanish.
    let ir = compile_ts(TS_CTOR, "Panel.ts", Fidelity::Low);
    let ctor_id = method_id(&ir.instructions, "constructor")
        .expect("the production stream must still declare the constructor");
    let hir = ir_to_hierarchical(&ir);

    let node = method_node(&hir, "constructor").unwrap_or_else(|| {
        let shape: Vec<(String, Vec<String>)> = hir
            .classes
            .iter()
            .map(|c| {
                (
                    c.name.clone(),
                    c.methods.iter().map(|m| m.name.clone()).collect(),
                )
            })
            .collect();
        panic!(
            "the classified constructor must still produce a hierarchical \
             MethodNode; classes: {shape:?}"
        )
    });
    assert_eq!(
        node.id, ctor_id,
        "the method node must keep the declaration's original method id"
    );
    assert!(
        node.patterns.iter().any(|p| p.name == "CTOR"),
        "the classification must be attached to that method node: {:?}",
        node.patterns
    );
}

#[test]
fn pattern_bearing_method_renders_its_m_line_alongside_its_classification() {
    // `Fidelity::Medium` is the compressed path under test: the PROMISE
    // classification is ACTIVE (it is not a raw passthrough), so both the
    // method line and the classification must be present in the rendered text.
    let ir = compile_ts(TS_PROMISE, "DataService.ts", Fidelity::Medium);
    let hir = ir_to_hierarchical(&ir);
    let rendered = render_hierarchical_for_llm(&hir, Fidelity::Medium);
    println!("=== rendered (Medium) ===\n{rendered}");

    assert!(
        rendered.contains("M load"),
        "the recognized method must still render an `M` line: {rendered}"
    );
    assert!(
        rendered.contains("P PROMISE"),
        "the classification must still render: {rendered}"
    );
}

// ─ RED-F2-9 / RED-F2-10 / RED-F2-11 ──────────────────────────────────

#[test]
fn pattern_bearing_method_remains_a_unit_table_target() {
    // Edit fidelity is the fidelity that emits spanned `Body` ops, and the
    // `UnitTable` only materializes records for methods that have BOTH a
    // `DefMethod` (the identity/fingerprint/metadata carrier) and a spanned
    // `Body`. Losing the `DefMethod` — even though the `Body` survives — made
    // the method unaddressable by `apply_edit`.
    let ir = compile_ts(TS_PROMISE, "DataService.ts", Fidelity::Edit);
    let load_id = method_id(&ir.instructions, "load").expect("declaration must survive");
    assert!(
        ir.instructions
            .iter()
            .any(|op| matches!(op, CoreOp::Pattern(name, args)
                if name == "PROMISE" && args.get(1).is_some_and(|a| a == &load_id))),
        "this fixture must actually exercise the pattern path: {:?}",
        ir.instructions
    );

    let table = UnitTable::from_instructions(&ir.instructions);
    let record = table.resolve(&load_id).unwrap_or_else(|e| {
        panic!(
            "a pattern-bearing method must stay an addressable unit: {e}; units: {:?}",
            table
                .iter()
                .map(|u| u.qualified_name.clone())
                .collect::<Vec<_>>()
        )
    });
    assert_eq!(record.name, "load");
    assert_eq!(
        record.class_name.as_deref(),
        Some("DataService"),
        "the unit must keep its real owner class"
    );
}

#[test]
fn pattern_bearing_method_still_projects_its_declaration() {
    // `Fidelity::Medium`: PROMISE is active here, so this is the TRUE
    // pattern-bearing shape. Pre-fix the recognizer consumed the `DefMethod`,
    // and the projection — which registers entities from `DefMethod` only —
    // produced no occurrence at all.
    let ir = compile_ts(TS_PROMISE, "DataService.ts", Fidelity::Medium);
    let load_id = method_id(&ir.instructions, "load").expect("declaration must survive");
    assert!(
        ir.instructions
            .iter()
            .any(|op| matches!(op, CoreOp::Pattern(name, args)
                if name == "PROMISE" && args.get(1).is_some_and(|a| a == &load_id))),
        "this fixture must actually exercise the pattern path: {:?}",
        ir.instructions
    );

    let edges = project_method_declarations(&ir.instructions, "C:/repo/DataService.ts");
    let registered: Vec<&str> = edges
        .iter()
        .map(|edge| edge.subject.name.as_str())
        .collect();

    assert!(
        registered.contains(&"load"),
        "the recognized method must still produce its registering occurrence: {registered:?}"
    );
    assert_eq!(
        registered.iter().filter(|name| **name == "load").count(),
        1,
        "the declaration occurrence must be projected exactly once: {registered:?}"
    );
}

#[test]
fn pattern_compression_cannot_detach_a_callers_identity() {
    // (a) Production control: a Promise-returning method that contains a call.
    // In production the call fact settles next to the method's own ops, so the
    // established IRPAT-001 guard declines compression for the region; either
    // way the caller-side identity must resolve to the REAL method name.
    let with_call = compile_ts(TS_PROMISE_WITH_CALL, "Caller.ts", Fidelity::Low);
    let call_edges = project_calls(&with_call.instructions, "C:/repo/Caller.ts");
    let from_load: Vec<&str> = call_edges
        .iter()
        .filter(|edge| edge.subject.name == "load")
        .map(|edge| edge.object.name.as_str())
        .collect();
    assert!(
        !from_load.is_empty(),
        "the call must be attributed to the real method; edges: {:?}",
        call_edges
            .iter()
            .map(|e| (e.subject.name.clone(), e.object.name.clone()))
            .collect::<Vec<_>>()
    );
    assert!(
        from_load.iter().any(|callee| callee.ends_with("resolve")),
        "the callee name must survive as written: {from_load:?}"
    );
    assert!(
        with_call
            .instructions
            .iter()
            .any(|op| matches!(op, CoreOp::DefMethod(_, _, name) if name == "load")),
        "the caller's declaration must survive pattern recognition"
    );

    // (b) The regression proper: a call fact positioned BEYOND the region the
    // IRPAT-001 guard inspects (the guard stops at the first op that does not
    // reference the method). Here the pattern DOES fire, so only the producer
    // boundary can keep the caller's identity alive — previously the
    // `DefMethod` was consumed and the `Calls` edge lost its subject name
    // entirely.
    let recognizer = CompressingPatternRecognizer::new();
    let input = vec![
        CoreOp::DefClass("C1".into(), "Example".into()),
        CoreOp::DefMethod("C1".into(), "M1".into(), "load".into()),
        CoreOp::Return("M1".into(), "$P".into()),
        CoreOp::DefMethod("C1".into(), "M2".into(), "other".into()),
        CoreOp::Call("M1".into(), "Save".into(), 1, false),
    ];
    let output = recognizer.recognize(&input);
    assert!(
        output
            .iter()
            .any(|op| matches!(op, CoreOp::Pattern(name, args)
                if name == "PROMISE" && args.get(1).is_some_and(|a| a == "M1"))),
        "the regression must actually exercise compression: {output:?}"
    );

    let edges = project_calls(&output, "C:/repo/Example.ts");
    assert!(
        edges.iter().any(|edge| edge.subject.name == "load"),
        "a caller id must always resolve to its declared method name: {:?}",
        edges
            .iter()
            .map(|e| (e.subject.name.clone(), e.object.name.clone()))
            .collect::<Vec<_>>()
    );
}
