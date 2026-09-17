// src/tests/ir/calls_java.rs
//
// Native call facts for Java: the production capture path.
//
// Every test compiles a real Java source string through the production
// compiler configuration (Java language layer + the two production pattern
// recognizers) with the base `JAVA_QUERY`, exactly like
// `compile_file_ir_focused` does, so the invocation captures under test ride
// the SAME tree-sitter parse.
//
// JAVA-CALL1..JAVA-CALL14 mirror the C# producer acceptance (arity shapes,
// caller identity, receiver exclusion, scope precision, semantic projection),
// and JAVA-CALL24..JAVA-CALL26 mirror the parse/compression/diff
// non-regression proof: adding a native call producer for Java must not add a
// parse, must not change compressed rendering, and must not change diff
// snapshots.
#![cfg(feature = "java")]

use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{parse_count, reset_parse_count, run_capture_pipeline};
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::layers::java::JavaLayer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::ir::render_llm::render_hierarchical_for_llm;
use crate::queries::JAVA_QUERY;

/// Compile a Java source string with the production compiler configuration.
fn compile_java(source: &str) -> CompiledIR {
    let language =
        crate::compression::language::safe_java_language().expect("java grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(JavaLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Example.java",
            language,
            JAVA_QUERY,
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

// ── JAVA-CALL1..JAVA-CALL8: arity shapes ────────────────────────────

#[test]
fn java_call1_simple_call_records_callee_and_one_argument() {
    let ir = compile_java("class Example { void a() { b(x); } }");
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("b".to_string(), 1)],
        "`a` must call `b` with the explicitly written arity"
    );
}

#[test]
fn java_call2_zero_argument_call_records_arity_zero() {
    let ir = compile_java("class Example { void a() { b(); } }");
    assert_eq!(calls_by(&ir, "a"), vec![("b".to_string(), 0)]);
}

#[test]
fn java_call3_nested_commas_do_not_inflate_arity() {
    let ir =
        compile_java("class Example { void a() { b(new HashMap<String, Integer>(), c(x, y)); } }");
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("b".to_string(), 2), ("c".to_string(), 2)],
        "generic type arguments and nested calls must not inflate the count"
    );
}

#[test]
fn java_call4_lambda_and_array_arguments_count_once() {
    let ir = compile_java("class Example { void a() { foo(v -> something(v, w)); } }");
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("foo".to_string(), 1), ("something".to_string(), 2)],
        "a lambda argument is one argument; its body is its own fact"
    );

    let ir = compile_java("class Example { void a() { foo(new int[] { 1, 2, 3 }); } }");
    assert_eq!(calls_by(&ir, "a"), vec![("foo".to_string(), 1)]);
}

#[test]
fn java_call5_member_invocation_excludes_the_receiver() {
    let ir = compile_java("class Example { void a() { this.items.orderBy(x, direction); } }");
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("orderBy".to_string(), 2)],
        "the receiver is not an argument and never enters the count"
    );
}

#[test]
fn java_call6_type_arguments_are_not_part_of_the_callee_name() {
    let ir = compile_java("class Example { void a() { this.<String>orderBy(x); } }");
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("orderBy".to_string(), 1)],
        "an explicit type argument list is a sibling field, never part of the name"
    );
}

#[test]
fn java_call7_varargs_call_shape_keeps_observed_arity() {
    let ir = compile_java(
        "class Example { \
           void target(int... values) { } \
           void a() { target(1, 2, 3); } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("target".to_string(), 3)],
        "an arity above the fixed parameter count is retained as written"
    );
    // RED-SPREAD7: Java varargs is a DECLARED parameter shape, not a call-site
    // expansion — the call site writes every argument — so the written count is
    // exact and must never be qualified.
    assert_eq!(
        calls_shape_by(&ir, "a"),
        vec![("target".to_string(), 3, false)],
        "a Java varargs call is exact written-arity evidence"
    );
    assert!(
        call_facts(&ir)
            .iter()
            .all(|(_, _, _, has_spread)| !has_spread),
        "no Java fact may carry a spread qualifier"
    );
}

#[test]
fn java_call8_receiver_less_and_static_calls_keep_observed_arity() {
    let ir = compile_java("class Example { void a() { save(); } }");
    assert_eq!(calls_by(&ir, "a"), vec![("save".to_string(), 0)]);

    let ir = compile_java("class Example { void a() { Service.save(x); } }");
    assert_eq!(calls_by(&ir, "a"), vec![("save".to_string(), 1)]);
}

// ── JAVA-CALL9..JAVA-CALL11: caller identity and ambiguity ──────────

#[test]
fn java_call9_two_callers_produce_two_caller_relationships() {
    let ir = compile_java("class Example { void a() { foo(); } void b() { foo(); } }");
    let facts = call_facts(&ir);
    assert_eq!(facts.len(), 2);
    let caller_ids: Vec<String> = facts.iter().map(|(caller, ..)| caller.clone()).collect();
    assert_ne!(
        caller_ids[0], caller_ids[1],
        "two declarations must own their own facts"
    );
    assert_eq!(calls_by(&ir, "a"), vec![("foo".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "b"), vec![("foo".to_string(), 0)]);
}

#[test]
fn java_call10_same_callee_different_arity_stays_two_facts() {
    let ir = compile_java("class Example { void a() { foo(x); foo(x, y); } }");
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("foo".to_string(), 1), ("foo".to_string(), 2)],
        "arity-distinct invocations must remain distinguishable"
    );
}

#[test]
fn java_call11_duplicate_identical_calls_are_two_ir_facts() {
    // The IR keeps one fact per invocation site; the WorkspaceIndex is where
    // identical facts of one asserting file collapse to one occurrence.
    let ir = compile_java("class Example { void a() { foo(x); foo(x); } }");
    assert_eq!(
        calls_by(&ir, "a"),
        vec![("foo".to_string(), 1), ("foo".to_string(), 1)],
        "the physical invocation count is not the semantic claim"
    );
}

// ── JAVA-CALL12..JAVA-CALL13: scope precision ───────────────────────

#[test]
fn java_call12_call_belongs_to_its_innermost_callable() {
    let ir = compile_java(
        "class Example { \
           void a() { foo(); } \
           void b() { bar(); } \
         }",
    );
    assert_eq!(calls_by(&ir, "a"), vec![("foo".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "b"), vec![("bar".to_string(), 0)]);
    for (caller, ..) in call_facts(&ir) {
        assert!(
            declared_method_ids(&ir).contains(&caller),
            "a caller must be a callable, never the enclosing class or file"
        );
    }
}

#[test]
fn java_call12_nested_class_method_owns_its_own_calls() {
    let ir = compile_java(
        "class Outer { \
           void a() { foo(); } \
           static class Inner { void b() { bar(); } } \
           void c() { baz(); } \
         }",
    );
    assert_eq!(calls_by(&ir, "a"), vec![("foo".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "b"), vec![("bar".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "c"), vec![("baz".to_string(), 0)]);
}

#[test]
fn java_call12_constructor_is_a_callable_owner() {
    let ir = compile_java("class Example { Example() { save(); } }");
    let facts = call_facts(&ir);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].1, "save");
    assert!(
        declared_method_ids(&ir).contains(&facts[0].0),
        "the constructor declaration owns its call"
    );
}

#[test]
fn java_call13_unsupported_contexts_emit_no_call_fact() {
    // Field initializer: no callable contains the invocation.
    let ir = compile_java("class Example { int x = foo(); }");
    assert!(
        call_facts(&ir).is_empty(),
        "a field initializer must not fabricate a caller"
    );

    // Object creation is not an invocation of a named method.
    let ir = compile_java("class Example { void a() { Object o = new Thing(x); } }");
    assert!(
        call_facts(&ir).is_empty(),
        "object creation is out of scope and must not be guessed into a call fact"
    );

    // An explicit constructor invocation is not a named method call.
    let ir = compile_java("class Example extends Base { Example() { super(x); } }");
    assert!(
        call_facts(&ir).is_empty(),
        "super(...) must not be guessed into a call fact"
    );
}

// ── JAVA-CALL14: the fact reaches the language-agnostic projection ──

#[test]
fn java_call14_call_fact_projects_onto_the_generic_calls_edge() {
    use crate::layers::meta::semantic::SemanticRelation;

    let ir = compile_java("class Example { void a() { b(x); } }");
    let edges = crate::ir::semantic_projection::project_generic_facts(
        &ir.instructions,
        "C:/repo/Example.java",
    );
    let call = edges
        .iter()
        .find(|edge| edge.relation == SemanticRelation::Calls)
        .expect("the call fact must project onto a Calls edge");
    assert_eq!(call.subject.domain, "builtin");
    assert_eq!(call.subject.entity_type, "Method");
    assert_eq!(call.subject.name, "a");
    assert_eq!(call.object.name, "b");
    let evidence = call.call_evidence.expect("the call must carry evidence");
    assert_eq!(evidence.explicit_arg_count, 1);
    assert!(
        !evidence.has_spread,
        "a non-expanding call must project exact written-arity evidence"
    );
    assert!(
        edges
            .iter()
            .any(|edge| edge.relation == SemanticRelation::Defines && edge.subject.name == "a"),
        "the callable declaration must register its own entity"
    );
}

// ── JAVA-CALL24..JAVA-CALL26: parse/compression/diff non-regression ─

#[test]
fn java_call24_call_capture_costs_no_additional_parse() {
    reset_parse_count();
    let ir = compile_java("class Example { void a() { b(x); c(); } }");
    assert_eq!(
        parse_count(),
        1,
        "the call producer must ride the single existing parse"
    );
    assert_eq!(calls_by(&ir, "a").len(), 2);
}

#[test]
fn java_call25_call_captures_do_not_touch_existing_compressed_output() {
    let language =
        crate::compression::language::safe_java_language().expect("java grammar enabled");
    let source = "class Example { void a() { b(x); } }";

    // 1. The compression path's query (JAVA_QUERY) exposes no call captures, so
    //    the positional capture consumers cannot observe them.
    let captures = run_capture_pipeline(
        language,
        JAVA_QUERY,
        source,
        Fidelity::Medium,
        |name, raw, _| Some(format!("{name}:{raw}")),
    )
    .expect("capture pipeline must succeed");
    assert!(
        captures
            .iter()
            .all(|capture| !capture.name.starts_with("call.")),
        "JAVA_QUERY must stay free of call captures: {:?}",
        captures.iter().map(|c| c.name.clone()).collect::<Vec<_>>()
    );

    // 2. The LLM-facing hierarchical render is byte-identical whether or not
    //    the native call facts are present in the instruction stream.
    let ir = compile_java(source);
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
fn java_call26_call_captures_do_not_change_diff_snapshots() {
    let source = "class Example { void a() { b(x); c(); } }";
    let snapshot =
        crate::diff::build_snapshot(source, Fidelity::Medium).expect("diff snapshot must build");
    let class = snapshot
        .classes
        .first()
        .expect("the class must be captured by the diff path");
    assert_eq!(class.name, "Example");
    assert_eq!(class.methods.len(), 1);
    // The diff snapshot is built from JAVA_QUERY only, so no call capture can
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
