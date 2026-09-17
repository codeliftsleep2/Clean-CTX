// src/tests/ir/calls_csharp.rs
//
// Native call facts for C#: the production capture path.
//
// Every test compiles a real C# source string through the production compiler
// configuration (C# language layer + the two production pattern recognizers)
// with the base `CS_QUERY`, exactly like `compile_file_ir_focused` does, so the
// invocation captures under test ride the SAME tree-sitter parse.
//
// RED-CALL1..RED-CALL13, RED-CALL24 (one parse), RED-CALL25 (compression
// non-regression) and RED-CALL26 (diff non-regression) live here.
#![cfg(feature = "csharp")]

use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{parse_count, reset_parse_count, run_capture_pipeline};
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::layers::csharp::CSharpLayer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::queries::CS_QUERY;

/// Compile a C# source string with the production compiler configuration.
fn compile_cs(source: &str) -> CompiledIR {
    let language =
        crate::compression::language::safe_csharp_language().expect("csharp grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(CSharpLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Example.cs",
            language,
            CS_QUERY,
            Fidelity::Medium,
            None,
        )
        .expect("production compilation must succeed")
}

/// Every native call fact in the stream as
/// `(caller_id, callee_name, argc, has_spread)`.
fn call_facts(ir: &CompiledIR) -> Vec<(String, String, usize, bool)> {
    ir.instructions
        .iter()
        .filter_map(|op| {
            op.call_parts().map(|(caller, callee, argc, has_spread)| {
                (caller.to_string(), callee.to_string(), argc, has_spread)
            })
        })
        .collect()
}

/// The declared name of a callable, by `DefMethod` id.
fn method_name(ir: &CompiledIR, method_id: &str) -> String {
    ir.instructions
        .iter()
        .find_map(|op| match op {
            CoreOp::DefMethod(_cid, mid, name) if mid == method_id => Some(name.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("unknown caller method id {method_id}"))
}

/// Every callable id declared in the stream.
fn declared_method_ids(ir: &CompiledIR) -> Vec<String> {
    ir.instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_cid, mid, _name) => Some(mid.clone()),
            _ => None,
        })
        .collect()
}

/// Call facts asserted by one named caller as `(callee, argc, has_spread)`, in
/// order — the full-evidence projection of one caller's invocations.
fn calls_shape_by(ir: &CompiledIR, caller_name: &str) -> Vec<(String, usize, bool)> {
    let caller_id = ir
        .instructions
        .iter()
        .find_map(|op| match op {
            CoreOp::DefMethod(_cid, mid, name) if name == caller_name => Some(mid.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("caller {caller_name} was not declared"));
    call_facts(ir)
        .into_iter()
        .filter(|(caller, _, _, _)| *caller == caller_id)
        .map(|(_, callee, argc, has_spread)| (callee, argc, has_spread))
        .collect()
}

/// The written-arity projection of one caller's call facts. The spread
/// qualifier is deliberately dropped HERE only, in a named projection; every
/// assertion that depends on the shape uses `calls_shape_by`.
fn calls_by(ir: &CompiledIR, caller_name: &str) -> Vec<(String, usize)> {
    calls_shape_by(ir, caller_name)
        .into_iter()
        .map(|(callee, argc, _)| (callee, argc))
        .collect()
}

// ── RED-CALL1..RED-CALL3: arity shapes ───────────────────────────────

#[test]
fn red_call1_simple_call_records_callee_and_one_argument() {
    let ir = compile_cs("class Example { void A() { B(x); } }");
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("B".to_string(), 1)],
        "A must call B with the explicitly written arity"
    );
}

#[test]
fn red_call2_zero_argument_call_records_arity_zero() {
    let ir = compile_cs("class Example { void A() { B(); } }");
    assert_eq!(calls_by(&ir, "A"), vec![("B".to_string(), 0)]);
}

#[test]
fn red_call3_nested_commas_do_not_inflate_arity() {
    let ir =
        compile_cs("class Example { void A() { B(new Dictionary<string, int>(), C(x, y)); } }");
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("B".to_string(), 2), ("C".to_string(), 2)],
        "generic/type arguments and nested calls must not inflate the count"
    );
}

#[test]
fn red_call3_lambda_and_array_arguments_count_once() {
    let ir = compile_cs("class Example { void A() { Foo(x => Something(x, y)); } }");
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("Foo".to_string(), 1), ("Something".to_string(), 2)],
        "a lambda argument is one argument; its body is its own fact"
    );

    let ir = compile_cs("class Example { void A() { Foo(new[] { 1, 2, 3 }); } }");
    assert_eq!(calls_by(&ir, "A"), vec![("Foo".to_string(), 1)]);
}

// ── RED-CALL4..RED-CALL7: caller identity and ambiguity ──────────────

#[test]
fn red_call4_two_callers_produce_two_caller_relationships() {
    let ir = compile_cs("class Example { void A() { Foo(); } void B() { Foo(); } }");
    let facts = call_facts(&ir);
    assert_eq!(facts.len(), 2);
    let caller_names: Vec<String> = facts
        .iter()
        .map(|(caller, ..)| method_name(&ir, caller))
        .collect();
    assert_eq!(caller_names, vec!["A".to_string(), "B".to_string()]);
    for (_, callee, argc, has_spread) in &facts {
        assert_eq!(callee, "Foo");
        assert_eq!(*argc, 0);
        assert!(!has_spread, "a C# call fact is always exact");
    }
}

#[test]
fn red_call5_same_callee_different_arity_stays_two_facts() {
    let ir = compile_cs("class Example { void A() { Foo(x); Foo(x, y); } }");
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("Foo".to_string(), 1), ("Foo".to_string(), 2)],
        "arity-distinct invocations must remain distinguishable"
    );
}

#[test]
fn red_call6_duplicate_identical_calls_are_two_ir_facts() {
    // The IR keeps one fact per invocation site; the WorkspaceIndex is where
    // identical facts of one asserting file collapse to one occurrence
    // (see workspace index calls_identical_facts_collapse_to_one).
    let ir = compile_cs("class Example { void A() { Foo(x); Foo(x); } }");
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("Foo".to_string(), 1), ("Foo".to_string(), 1)],
        "the physical invocation count is not the semantic claim"
    );
}

#[test]
fn red_call7_same_name_same_arity_overloads_stay_unresolved() {
    let ir = compile_cs(
        "class Example { \
           void Foo(string x) {} \
           void Foo(int x) {} \
           void A() { Foo(value); } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("Foo".to_string(), 1)],
        "the fact is the callee NAME and the observed arity, never a resolved overload"
    );
    assert_eq!(
        declared_method_ids(&ir).len(),
        3,
        "both overload declarations remain declarations"
    );
}

// ── RED-CALL8..RED-CALL10: receiver/arity truthfulness ───────────────

#[test]
fn red_call8_extension_invocation_excludes_the_receiver() {
    let ir = compile_cs("class Example { void A() { items.OrderBy(x => x.Name, direction); } }");
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("OrderBy".to_string(), 2)],
        "the receiver is not an argument and never enters the count"
    );
}

#[test]
fn red_call9_optional_parameter_call_shape_keeps_observed_arity() {
    let ir = compile_cs(
        "class Example { \
           void Target(string value = \"x\") {} \
           void A() { Target(); } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("Target".to_string(), 0)],
        "the observed arity is retained without declaration-equality assumptions"
    );
}

#[test]
fn red_call10_params_call_shape_keeps_observed_arity() {
    let ir = compile_cs(
        "class Example { \
           void Target(params int[] values) {} \
           void A() { Target(1, 2, 3); } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "A"),
        vec![("Target".to_string(), 3)],
        "an arity above the fixed parameter count is retained as written"
    );
}

// ── RED-CALL11..RED-CALL12: scope precision ─────────────────────────

#[test]
fn red_call11_call_belongs_to_its_innermost_callable() {
    let ir = compile_cs(
        "class Example { \
           void A() { Foo(); } \
           void B() { Bar(); } \
         }",
    );
    assert_eq!(calls_by(&ir, "A"), vec![("Foo".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "B"), vec![("Bar".to_string(), 0)]);
    for (caller, ..) in call_facts(&ir) {
        assert!(
            declared_method_ids(&ir).contains(&caller),
            "a caller must be a callable, never the enclosing class or file"
        );
    }
}

#[test]
fn red_call11_nested_class_method_owns_its_own_calls() {
    let ir = compile_cs(
        "class Outer { \
           void A() { Foo(); } \
           class Inner { void B() { Bar(); } } \
           void C() { Baz(); } \
         }",
    );
    assert_eq!(calls_by(&ir, "A"), vec![("Foo".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "B"), vec![("Bar".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "C"), vec![("Baz".to_string(), 0)]);
}

#[test]
fn red_call11_constructor_is_a_callable_owner() {
    let ir = compile_cs("class Example { Example() { Save(); } }");
    let facts = call_facts(&ir);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].1, "Save");
    assert!(
        declared_method_ids(&ir).contains(&facts[0].0),
        "the constructor declaration owns its call"
    );
}

#[test]
fn red_call12_unsupported_contexts_emit_no_call_fact() {
    // Field initializer: no callable contains the invocation.
    let ir = compile_cs("class Example { int X = Foo(); }");
    assert!(
        call_facts(&ir).is_empty(),
        "a field initializer must not fabricate a caller"
    );

    // Property accessor body: not a supported callable declaration.
    let ir = compile_cs("class Example { int X { get { return Foo(); } } }");
    assert!(
        call_facts(&ir).is_empty(),
        "a property accessor must not fabricate a caller"
    );
}

// ── Spread qualifier: C# has no caller-side expansion syntax ─────────

#[test]
fn red_spread_csharp_facts_are_always_exact() {
    // `params` is a DECLARED parameter shape, not a call-site expansion: a C#
    // call site writes every argument, so no C# fact may ever be qualified.
    let ir = compile_cs(
        "class Example { \
           void Target(params int[] values) { } \
           void A() { Target(1, 2, 3); Foo(new Dictionary<string, int>(), x); } \
         }",
    );
    let facts = call_facts(&ir);
    assert_eq!(facts.len(), 2);
    assert!(
        facts.iter().all(|(_, _, _, has_spread)| !has_spread),
        "no C# fact may carry a spread qualifier: {facts:?}"
    );
    assert_eq!(
        calls_shape_by(&ir, "A"),
        vec![
            ("Target".to_string(), 3, false),
            ("Foo".to_string(), 2, false),
        ],
        "the written count stays exact for every C# call"
    );
}

// ── RED-CALL24..RED-CALL26: parse/compression/diff non-regression ────

#[test]
fn red_call24_call_capture_costs_no_additional_parse() {
    reset_parse_count();
    let ir = compile_cs("class Example { void A() { B(x); C(); } }");
    assert_eq!(
        parse_count(),
        1,
        "the call producer must ride the single existing parse"
    );
    assert_eq!(calls_by(&ir, "A").len(), 2);
}

#[test]
fn red_call25_call_captures_do_not_touch_existing_compressed_output() {
    let language =
        crate::compression::language::safe_csharp_language().expect("csharp grammar enabled");
    let source = "class Example { void A() { B(x); } }";

    // 1. The compression path's query (CS_QUERY) exposes no call captures, so
    //    the positional capture consumers cannot observe them.
    let captures = run_capture_pipeline(
        language,
        CS_QUERY,
        source,
        Fidelity::Medium,
        |name, raw, _| Some(format!("{name}:{raw}")),
    )
    .expect("capture pipeline must succeed");
    assert!(
        captures
            .iter()
            .all(|capture| !capture.name.starts_with("call.")),
        "CS_QUERY must stay free of call captures: {:?}",
        captures.iter().map(|c| c.name.clone()).collect::<Vec<_>>()
    );

    // 2. The LLM-facing hierarchical render is byte-identical whether or not
    //    the native call facts are present in the instruction stream.
    let ir = compile_cs(source);
    let with_calls = render_hierarchical_for_llm(
        &crate::ir::hierarchical::ir_to_hierarchical(&ir),
        Fidelity::Medium,
    );
    let stripped = CompiledIR {
        file_id: ir.file_id.clone(),
        instructions: ir
            .instructions
            .iter()
            .filter(|op| op.call_parts().is_none())
            .cloned()
            .collect(),
        version: ir.version,
    };
    let without_calls = render_hierarchical_for_llm(
        &crate::ir::hierarchical::ir_to_hierarchical(&stripped),
        Fidelity::Medium,
    );
    assert_eq!(
        with_calls, without_calls,
        "adding call captures must not change compressed rendering"
    );
}

#[test]
fn red_call26_call_captures_do_not_change_diff_snapshots() {
    let source = "class Example { void A() { B(x); C(); } }";
    let snapshot =
        crate::diff::build_snapshot(source, Fidelity::Medium).expect("diff snapshot must build");
    let class = snapshot
        .classes
        .first()
        .expect("the class must be captured by the diff path");
    assert_eq!(class.name, "Example");
    assert_eq!(class.methods.len(), 1);
    // The diff snapshot is built from CS_QUERY only, so no call capture can
    // appear in a method's marker set.
    assert!(
        class.methods.iter().all(|method| method
            .markers
            .iter()
            .all(|marker| !marker.contains("call."))),
        "call captures must never reach the diff snapshot: {:?}",
        class
            .methods
            .iter()
            .map(|method| method.markers.clone())
            .collect::<Vec<_>>()
    );
}
