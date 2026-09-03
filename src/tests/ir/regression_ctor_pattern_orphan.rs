// src/tests/ir/regression_ctor_pattern_orphan.rs
//
// RED regression - Issue #36 / Addendum #37 (supersedes the generic
// nested-callback regression, which asserted a passing invariant for shapes
// that do not reproduce the production failure).
//
// Root cause (proven from source):
//   try_compress_pattern consumes [leading Flags(M)] + [DEF_M + Param* +
//   Return] + [trailing Flags(M)] into PAT(CTOR, C, M, ...). The validator
//   registers methods ONLY from CoreOp::DefMethod, so any M-annotating op
//   left outside that window becomes orphaned. TypeScriptLayer emits, for a
//   constructor whose body contains `.subscribe(`:
//     Flags(M,[PRIVATE]) then DataFlow(M, reads observable) - and
//     flush_method_flags emits a SECOND Flags(M,[...]) AFTER the DataFlow
//     when a control-flow capture (if/return/try/...) occurs inside the ctor
//     body (including inside subscribe callbacks). Trailing-Flags cleanup
//     stops at the DataFlow, orphaning DataFlow (E007) and the second Flags
//     (E003) - the exact reported [E007]+[E003] pair on M16.
//
// Run: cargo test --release --all-features --lib -- ctor_pattern_orphan --nocapture 2>&1

use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::ir::validator::{DefaultValidator, IRValidator};

// Minimal failing construct: constructor + DI param + `.subscribe(` in body
// + a control-flow capture (if) inside the subscribe callback.
const FAILING: &str = r#"
import { Component } from '@angular/core';
@Component({ selector: 'test', template: '' })
export class TestComponent {
    constructor(private service: MyService) {
        this.service.getData().subscribe(x => {
            if (x) { console.log(x); }
        });
    }
}
"#;

// Same constructor, subscribe but NO control-flow capture anywhere in the body.
const SUBSCRIBE_NO_CTRL: &str = r#"
import { Component } from '@angular/core';
@Component({ selector: 'test', template: '' })
export class TestComponent {
    constructor(private service: MyService) {
        this.service.getData().subscribe(x => { console.log(x); });
    }
}
"#;

// Control: constructor + control flow but NO subscribe (no DataFlow emitted).
const CTRL_NO_SUBSCRIBE: &str = r#"
import { Component } from '@angular/core';
@Component({ selector: 'test', template: '' })
export class TestComponent {
    constructor(private service: MyService) {
        if (false) { console.log('never'); }
    }
}
"#;

fn compile_with(source: &str, file_id: &str, with_patterns: bool) -> Vec<CoreOp> {
    let (language, query_string) =
        crate::compression::language::language_for_extension("ts").expect("TS language");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    if with_patterns {
        // Exact production two-pass ordering (mirrors diagnose_m6.rs harness):
        // additive CodePatternRecognizer first, consumptive second.
        compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
        compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    }
    let ir = compiler
        .compile(source, file_id, language, query_string, Fidelity::Low, None)
        .unwrap_or_else(|e| panic!("{file_id} should compile: {e}"));
    ir.instructions
}

/// Production passes WITHOUT ValidationPass, so the post-pattern stream can
/// be inspected even while the orphan defect makes the full pipeline fail.
fn compile_stream_no_validation(source: &str, file_id: &str) -> Vec<CoreOp> {
    use crate::ir::pipeline::{
        CoreIRPass, LanguageLayerPass, PassContext, PassPipeline, PatternRecognitionPass,
    };
    let (language, query_string) =
        crate::compression::language::language_for_extension("ts").expect("TS language");
    let mut ctx = PassContext::new(source.to_string(), file_id.to_string(), Fidelity::Low);
    ctx.language = Some(language);
    ctx.query_string = query_string.to_string();
    ctx.set_language_layers(vec![Box::new(TypeScriptLayer::new())]);
    // Exact production two-pass recognizer ordering.
    ctx.set_pattern_recognizers(vec![
        Box::new(CodePatternRecognizer::new()),
        Box::new(CompressingPatternRecognizer::new()),
    ]);
    let mut pipeline = PassPipeline::new();
    pipeline.add_pass(Box::new(CoreIRPass::new()));
    pipeline.add_pass(Box::new(LanguageLayerPass::new()));
    pipeline.add_pass(Box::new(PatternRecognitionPass::new()));
    pipeline.run(&mut ctx).expect("stream pipeline should run");
    ctx.instructions
}

fn registered_methods(instructions: &[CoreOp]) -> Vec<String> {
    instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_, mid, _) => Some(mid.clone()),
            _ => None,
        })
        .collect()
}

fn orphaned_refs(instructions: &[CoreOp]) -> Vec<String> {
    let mids = registered_methods(instructions);
    let mut out = Vec::new();
    for op in instructions {
        let mid = match op {
            CoreOp::Flags(mid, _)
            | CoreOp::DataFlow(mid, _, _)
            | CoreOp::SideEffect(mid, _)
            | CoreOp::ExecutionContext(mid, _)
            | CoreOp::ControlFlow(mid, _, _)
            | CoreOp::Return(mid, _)
            | CoreOp::Param(mid, _, _, _) => Some(mid),
            _ => None,
        };
        if let Some(mid) = mid {
            if !mids.contains(mid) {
                out.push(format!("{op:?}"));
            }
        }
    }
    out
}

// === RED: the actual production failure =====================================
//
// These assert the GOOD invariant (no orphans / no E003+E007). They are RED
// on the current tree and must turn GREEN after the fix.

#[test]
fn red_ctor_subscribe_if_references_only_registered_methods() {
    let instructions = compile_with(FAILING, "failing.ts", true);
    println!("=== FULL PRODUCTION STREAM (two-pass) ===");
    for (i, op) in instructions.iter().enumerate() {
        println!("  [{i:3}] {op:?}");
    }
    let orphans = orphaned_refs(&instructions);
    // Underlying invariant, not just "compile() succeeds": every
    // M-referencing op must have a DefMethod owner.
    assert!(
        orphans.is_empty(),
        "RED (production failure reproduced): orphaned references: {orphans:?}"
    );
    // Ownership preserved: the constructor region stays uncompressed, because
    // PatternOp has no payload for DataFlow/SideEffect/ExecutionContext/
    // ControlFlow facts — consuming DefMethod(M) into a PAT would leave them
    // unowned. The ctor therefore keeps its DefMethod and its CTOR flag.
    assert!(
        instructions
            .iter()
            .any(|op| matches!(op, CoreOp::DefMethod(_, _, name) if name == "constructor")),
        "constructor DefMethod must remain (ownership for its annotations)"
    );
    assert!(
        !instructions
            .iter()
            .any(|op| matches!(op, CoreOp::Pattern(name, _) if name == "CTOR")),
        "CTOR region with unrepresentable annotations must NOT be compressed"
    );
}

#[test]
fn red_failing_fixture_validator_reports_no_e003_e007() {
    let instructions = compile_with(FAILING, "failing.ts", true);
    let ir = crate::ir::compiler::CompiledIR {
        file_id: "failing.ts".into(),
        instructions: instructions.clone(),
        version: 1,
    };
    let errors = DefaultValidator::new().validate(&ir);
    let relevant: Vec<_> = errors
        .iter()
        .filter(|e| e.code == "E003" || e.code == "E007")
        .collect();
    assert!(
        relevant.is_empty(),
        "RED (production failure reproduced): validator reported: {relevant:?}"
    );
}

// === Pre-pattern baseline: the core pass is clean ===========================

#[test]
fn pre_pattern_stream_has_no_orphans() {
    // TS layer only - the stream BEFORE PatternRecognitionPass. Every
    // annotation references a registered DefMethod. This proves the orphan
    // is CREATED by the consumptive pattern pass, not by core allocation.
    let pre = compile_with(FAILING, "pre.ts", false);
    println!("=== PRE-PATTERN STREAM (TS layer only) ===");
    for (i, op) in pre.iter().enumerate() {
        println!("  [{i:3}] {op:?}");
    }
    let orphans = orphaned_refs(&pre);
    assert!(
        orphans.is_empty(),
        "pre-pattern stream must be clean; got: {orphans:?}"
    );
}

// === Bisection controls =====================================================

#[test]
fn ctor_subscribe_without_control_flow_orphans_dataflow_only() {
    // subscribe but no if/return/try => the ONLY orphan is the DataFlow op
    // (the single trailing FLAGS(PRIVATE) is adjacent to RET and consumed).
    // This isolates the E007 half of the live failure. RED: asserts empty.
    let stream = compile_stream_no_validation(SUBSCRIBE_NO_CTRL, "sub-only.ts");
    println!("=== SUBSCRIBE-ONLY STREAM (no validation pass) ===");
    for (i, op) in stream.iter().enumerate() {
        println!("  [{i:3}] {op:?}");
    }
    let orphans = orphaned_refs(&stream);
    println!("orphaned: {orphans:?}");
    assert!(
        orphans.is_empty(),
        "RED (E007-only shape): orphaned references: {orphans:?}"
    );
}

#[test]
fn control_ctor_without_subscribe_is_clean() {
    // Control flow WITHOUT subscribe: FLAGS(M,[IF]) is trailing-adjacent to
    // RET (no DataFlow in between) => consumed; validator must be clean.
    let instructions = compile_with(CTRL_NO_SUBSCRIBE, "ctrl.ts", true);
    println!("=== CONTROL STREAM ===");
    for (i, op) in instructions.iter().enumerate() {
        println!("  [{i:3}] {op:?}");
    }
    let orphans = orphaned_refs(&instructions);
    assert!(
        orphans.is_empty(),
        "control must be clean, got: {orphans:?}"
    );
    let ir = crate::ir::compiler::CompiledIR {
        file_id: "ctrl.ts".into(),
        instructions,
        version: 1,
    };
    let errors = DefaultValidator::new().validate(&ir);
    assert!(
        errors.is_empty(),
        "control validator must be clean, got: {errors:?}"
    );
}

// === Sibling path: EMPTY ctor with subscribe ================================

// No DI param => the EmptyConstructor pattern is the consumption path. The
// same orphan invariant applies: DefMethod(M) must not be consumed while a
// DataFlow(M) survives after the span.
const EMPTY_CTOR_SUBSCRIBE: &str = r#"
import { Component } from '@angular/core';
@Component({ selector: 'test', template: '' })
export class TestComponent {
    constructor() {
        this.service.getData().subscribe(x => { console.log(x); });
    }
}
"#;

#[test]
fn empty_ctor_subscribe_references_only_registered_methods() {
    let instructions = compile_with(EMPTY_CTOR_SUBSCRIBE, "empty-ctor.ts", true);
    println!("=== EMPTY-CTOR SUBSCRIBE STREAM (full pipeline) ===");
    for (i, op) in instructions.iter().enumerate() {
        println!("  [{i:3}] {op:?}");
    }
    let orphans = orphaned_refs(&instructions);
    assert!(
        orphans.is_empty(),
        "RED (empty-ctor path): orphaned references: {orphans:?}"
    );
    assert!(
        instructions
            .iter()
            .any(|op| matches!(op, CoreOp::DefMethod(_, _, name) if name == "constructor")),
        "constructor DefMethod must remain (ownership for its annotations)"
    );
}

// === DIS-2026-003: Edit-fidelity Body op breaks the trailing-Flags window ===
//
// At `Fidelity::Edit` the TypeScript layer emits `Body(M, ...)` operations.
// For an empty constructor with a parameter property the Edit-fidelity
// pre-pattern stream is:
//
//   DefMethod(M, constructor)
//   Param(M, ...)
//   Return(M, ...)
//   Body(M, "{}", ...)              ← Edit-only: sits between Return and Flags
//   Flags(M, ["PRIVATE"])           ← parameter-property modifier
//
// The CTOR compression window (`[leading Flags] + [DEF_M + Param* + Return]
// + [trailing Flags(M)]`) is broken by the Body op: the trailing `Flags(M)`
// is no longer adjacent to the consumed span, so the wrapper cannot consume
// it. Pre-fix, the orphan guard did not consider `Body(M)` an unrepresentable
// M-reference, so compression proceeded and orphaned `Flags(M)` (E003).
// The fix adds `Body` to `op_is_unrepresentable_method_ref`, making the
// guard decline compression and preserve the full valid sequence.
//
// Minimal structural reproducer (independent of the original application
// fixture — no NgRx, no Angular, no `this.store` reference required):

const EDIT_FIDELITY_PARAM_PROPERTY: &str = r#"
class Example {
    constructor(private store: Store) {}

    ngOnInit() {
        console.log('unrelated operation outside the constructor');
    }
}
"#;

/// Compile at an explicit fidelity (the shared `compile_with` helper pins
/// `Fidelity::Low`, which does not emit `Body` ops — the DIS-2026-003
/// trigger requires Edit fidelity).
fn compile_with_fidelity(source: &str, file_id: &str, fidelity: Fidelity) -> Vec<CoreOp> {
    let (language, query_string) =
        crate::compression::language::language_for_extension("ts").expect("TS language");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    let ir = compiler
        .compile(source, file_id, language, query_string, fidelity, None)
        .unwrap_or_else(|e| panic!("{file_id} should compile: {e}"));
    ir.instructions
}

#[test]
fn edit_fidelity_param_property_ctor_does_not_orphan_flags() {
    // Full production pipeline at Fidelity::Edit — the fidelity that emits
    // `Body(M)` ops. Compilation itself must succeed (pre-fix it failed with
    // `[E003] FLAGS references unknown method 'M…'` inside the pipeline's
    // ValidationPass).
    let instructions =
        compile_with_fidelity(EDIT_FIDELITY_PARAM_PROPERTY, "edit-ctor.ts", Fidelity::Edit);
    println!("=== EDIT-FIDELITY PARAM-PROPERTY STREAM (full pipeline) ===");
    for (i, op) in instructions.iter().enumerate() {
        println!("  [{i:3}] {op:?}");
    }

    // The constructor's DefMethod must still be present (the pattern must
    // have declined compression, preserving the identity for its Flags).
    assert!(
        instructions
            .iter()
            .any(|op| matches!(op, CoreOp::DefMethod(_, _, name) if name == "constructor")),
        "constructor DefMethod must remain — CTOR compression must decline \
         when an Edit-fidelity Body op detaches the trailing Flags from the \
         consumed span"
    );

    // No orphaned M-references anywhere in the stream.
    let orphans = orphaned_refs(&instructions);
    assert!(
        orphans.is_empty(),
        "Edit-fidelity param-property ctor must not orphan method refs; got: {orphans:?}"
    );

    // Validator must be clean — no E003 (orphaned Flags), no E007 (orphaned
    // DataFlow), and no other validation violation.
    let ir = crate::ir::compiler::CompiledIR {
        file_id: "edit-ctor.ts".into(),
        instructions,
        version: 1,
    };
    let errors = DefaultValidator::new().validate(&ir);
    assert!(
        errors.is_empty(),
        "Edit-fidelity param-property ctor must validate cleanly (no E003/E007); got: {errors:?}"
    );
}
