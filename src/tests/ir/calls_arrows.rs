// src/tests/ir/calls_arrows.rs
//
// Bound-arrow callable identity for TypeScript: ownership, identity and scope.
//
// An arrow expression that is the value of a VARIABLE BINDING or of a CLASS
// PROPERTY DECLARATION carries a stable written owner name, so it establishes
// its own `builtin` / `Method` callable scope (the SAME innermost-span
// ownership the method/function declarations use). Everything else an arrow can
// be — a callback argument, an object-literal property, an IIFE, a `this.x =`
// assignment — establishes NO scope, so an invocation inside it stays owned by
// the nearest enclosing recognized callable.
//
// This module owns the ownership/identity/scope matrix:
//   RED-ARROW1  — class property arrow                      (`load -> save`)
//   RED-ARROW2  — `private` property arrow
//   RED-ARROW3  — `readonly` property arrow
//   RED-ARROW4  — `async` arrow (never a distinct identity)
//   RED-ARROW5  — concise expression body (no block required)
//   RED-ARROW6  — nested callback inside a named method (preserved behavior)
//   RED-ARROW7  — object-form `.subscribe({ next: ... })` (preserved behavior)
//   RED-ARROW8  — two nesting levels inside a named method
//   RED-ARROW9  — nested callback inside a property arrow
//   RED-ARROW10 — property arrow calling another property arrow
//   RED-ARROW11 — multiple property arrows in one class (no caller bleed)
//   RED-ARROW12 — same arrow name in two classes (distinct facts, same name)
//   RED-ARROW13 — generic arrow
//   RED-ARROW14 — typed property arrow
//   RED-ARROW15 — multiline / parenthesized formatting
//   RED-ARROW19 — top-level named arrow (supported: binding identity)
//   RED-ARROW20 — local variable-bound arrow (innermost callable wins)
//   RED-ARROW21 — anonymous top-level arrow (no fabricated caller)
//   RED-ARROW22 — anonymous initializer callback (no fabricated owner)
//   RED-ARROW23 — arrow + spread (ownership AND spread evidence)
//   RED-ARROW25 — one parse (no second `Parser::parse`)
//   RED-ARROW26 — base-query / compression / diff isolation
//
// RED-ARROW16..18 (RxJS, Promise, timer callbacks), RED-ARROW24 (nested arrows
// + spread) live in the sibling module `calls_arrows_callbacks.rs`, which
// reuses the compilation and assertion helpers below.
#![cfg(feature = "typescript")]

use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{parse_count, reset_parse_count, run_capture_pipeline};
use crate::ir::calls::capture_query;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::queries::TS_QUERY;

/// Compile a TypeScript source string with the production compiler
/// configuration, exactly like the TypeScript call-fact acceptance does, so the
/// arrow captures under test ride the SAME single tree-sitter parse.
pub(super) fn compile_ts(source: &str) -> CompiledIR {
    let language = crate::compression::language::safe_typescript_language()
        .expect("typescript grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Example.ts",
            language,
            TS_QUERY,
            Fidelity::Medium,
            None,
        )
        .expect("production compilation must succeed")
}

/// Every native call fact in the stream as
/// `(caller_id, callee_name, argc, has_spread)`.
pub(super) fn call_facts(ir: &CompiledIR) -> Vec<(String, String, usize, bool)> {
    ir.instructions
        .iter()
        .filter_map(|op| {
            op.call_parts().map(|(caller, callee, argc, has_spread)| {
                (caller.to_string(), callee.to_string(), argc, has_spread)
            })
        })
        .collect()
}

/// The method id declared for one callable NAME (the first declaration wins).
pub(super) fn declared_id(ir: &CompiledIR, name: &str) -> String {
    ir.instructions
        .iter()
        .find_map(|op| match op {
            CoreOp::DefMethod(_cid, mid, declared) if declared == name => Some(mid.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("callable {name} was not declared"))
}

/// Every declared callable name in the stream, in declaration order.
pub(super) fn declared_names(ir: &CompiledIR) -> Vec<String> {
    ir.instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_cid, _mid, name) => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// The declared NAME of every caller appearing in a call fact, sorted+deduped.
pub(super) fn caller_names(ir: &CompiledIR) -> Vec<String> {
    let declared: Vec<(String, String)> = ir
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_cid, mid, name) => Some((mid.clone(), name.clone())),
            _ => None,
        })
        .collect();
    let mut names: Vec<String> = call_facts(ir)
        .into_iter()
        .map(|(caller, ..)| {
            declared
                .iter()
                .find(|(mid, _)| *mid == caller)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| format!("<undeclared:{caller}>"))
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Call facts asserted by one named caller as `(callee, argc, has_spread)`.
pub(super) fn calls_shape_by(ir: &CompiledIR, caller_name: &str) -> Vec<(String, usize, bool)> {
    let caller_id = declared_id(ir, caller_name);
    call_facts(ir)
        .into_iter()
        .filter(|(caller, _, _, _)| *caller == caller_id)
        .map(|(_, callee, argc, has_spread)| (callee, argc, has_spread))
        .collect()
}

/// The written-arity projection of one caller's call facts.
pub(super) fn calls_by(ir: &CompiledIR, caller_name: &str) -> Vec<(String, usize)> {
    calls_shape_by(ir, caller_name)
        .into_iter()
        .map(|(callee, argc, _)| (callee, argc))
        .collect()
}

// ── RED-ARROW1..RED-ARROW5: the declaration forms ────────────────────

#[test]
fn red_arrow1_class_property_arrow_owns_its_call() {
    let ir = compile_ts("class Example { load = () => save(); }");
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("save".to_string(), 0)],
        "a class property arrow must own the invocation in its concise body"
    );
    assert_eq!(
        caller_names(&ir),
        vec!["load".to_string()],
        "the caller must be the property name, never the enclosing class"
    );
}

#[test]
fn red_arrow2_private_property_arrow_owns_its_call() {
    let ir = compile_ts("class Example { private load = () => { save(); }; }");
    assert_eq!(calls_by(&ir, "load"), vec![("save".to_string(), 0)]);
}

#[test]
fn red_arrow3_readonly_property_arrow_owns_its_call() {
    let ir = compile_ts("class Example { readonly load = () => { save(); }; }");
    assert_eq!(calls_by(&ir, "load"), vec![("save".to_string(), 0)]);
}

#[test]
fn red_arrow4_async_arrow_is_the_same_identity() {
    let block = compile_ts("class Example { load = async () => { await save(); }; }");
    let concise = compile_ts("class Example { load = async () => save(); }");
    assert_eq!(
        calls_by(&block, "load"),
        vec![("save".to_string(), 0)],
        "`async` is a modifier, never a distinct semantic identity"
    );
    assert_eq!(calls_by(&block, "load"), calls_by(&concise, "load"));
    assert_eq!(declared_names(&block), vec!["load".to_string()]);
}

#[test]
fn red_arrow5_concise_expression_body_needs_no_block() {
    let ir = compile_ts("class Example { load = () => save(); }");
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("save".to_string(), 0)],
        "a concise expression body is a body like any other"
    );
    assert_eq!(declared_names(&ir), vec!["load".to_string()]);
}

// ── RED-ARROW6..RED-ARROW9: scope precedence ─────────────────────────

#[test]
fn red_arrow6_nested_arrow_callback_inside_a_method_stays_with_the_method() {
    let ir = compile_ts(
        "class Example { \
           load(): void { this.source.subscribe(x => save(x)); } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("subscribe".to_string(), 1), ("save".to_string(), 1)],
        "an anonymous arrow argument adds no caller: the method keeps the fact"
    );
    assert_eq!(
        caller_names(&ir),
        vec!["load".to_string()],
        "no synthetic caller identity may be fabricated for the callback"
    );
}

#[test]
fn red_arrow7_object_form_subscribe_keeps_the_enclosing_method_as_owner() {
    let ir = compile_ts(
        "class Example { \
           load(): void { \
             x.subscribe({ \
               next: tp => save(tp), \
               error: err => log(err) \
             }); \
           } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("subscribe".to_string(), 1),
            ("save".to_string(), 1),
            ("log".to_string(), 1),
        ],
        "the object-form callback fields are not callable owners: the method \
         owns their invocations"
    );
    assert_eq!(
        caller_names(&ir),
        vec!["load".to_string()],
        "`next` / `error` must never appear as synthetic callers"
    );
}

#[test]
fn red_arrow8_two_nesting_levels_inside_a_method_keep_the_method() {
    let ir = compile_ts(
        "class Example { \
           load(): void { \
             this.source.subscribe(x => { \
               this.items.forEach(y => { \
                 save(y); \
               }); \
             }); \
           } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("subscribe".to_string(), 1),
            ("forEach".to_string(), 1),
            ("save".to_string(), 1),
        ],
        "nested anonymous arrow spans must not confuse the ownership algorithm"
    );
}

#[test]
fn red_arrow9_nested_arrow_callback_inside_a_property_arrow_takes_the_arrow() {
    let ir = compile_ts(
        "class Example { \
           load = () => { \
             this.source.subscribe(x => { \
               save(x); \
             }); \
           }; \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("subscribe".to_string(), 1), ("save".to_string(), 1)],
        "the property arrow is the recognized outer callable, so the nested \
         anonymous arrow inherits it"
    );
    assert_eq!(caller_names(&ir), vec!["load".to_string()]);
}

// ── RED-ARROW10..RED-ARROW15: identity across declaration shapes ──────

#[test]
fn red_arrow10_property_arrow_calling_another_property_arrow() {
    let ir = compile_ts(
        "class Example { \
           load = () => { this.refresh(); }; \
           refresh = () => { save(); }; \
         }",
    );
    assert_eq!(calls_by(&ir, "load"), vec![("refresh".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "refresh"), vec![("save".to_string(), 0)]);
    assert_eq!(
        call_facts(&ir).len(),
        2,
        "each arrow owns exactly its own invocation: no class-level attribution"
    );
}

#[test]
fn red_arrow11_multiple_property_arrows_in_one_class_do_not_bleed() {
    let ir = compile_ts(
        "class Example { \
           load = () => { saveA(); }; \
           cancel = () => { abort(); }; \
           refresh = () => { saveB(); }; \
         }",
    );
    assert_eq!(calls_by(&ir, "load"), vec![("saveA".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "cancel"), vec![("abort".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "refresh"), vec![("saveB".to_string(), 0)]);

    let mut callers: Vec<String> = call_facts(&ir).into_iter().map(|f| f.0).collect();
    callers.sort();
    callers.dedup();
    assert_eq!(
        callers.len(),
        3,
        "three arrows are three distinct callable declarations"
    );
}

#[test]
fn red_arrow12_same_arrow_name_in_two_classes_keeps_distinct_occurrences() {
    let ir = compile_ts("class A { load = () => saveA(); } class B { load = () => saveB(); }");
    let facts = call_facts(&ir);
    assert_eq!(facts.len(), 2, "both arrows own their own invocation");

    assert_eq!(
        caller_names(&ir),
        vec!["load".to_string()],
        "Model C identity is the NAME: both callables are `builtin`/`Method`/`load`"
    );
    let save_a = facts
        .iter()
        .find(|(_, callee, ..)| callee == "saveA")
        .expect("the first class's arrow owns `saveA`");
    let save_b = facts
        .iter()
        .find(|(_, callee, ..)| callee == "saveB")
        .expect("the second class's arrow owns `saveB`");
    assert_ne!(
        save_a.0, save_b.0,
        "the two occurrences are distinct declarations, so their facts stay distinct"
    );
}

#[test]
fn red_arrow13_generic_arrow_keeps_its_binding_name() {
    let ir = compile_ts("class Example { load = <T>(value: T) => { save(value); }; }");
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("save".to_string(), 1)],
        "a type-parameter list is a sibling of the value, never a new identity"
    );
    assert_eq!(declared_names(&ir), vec!["load".to_string()]);
}

#[test]
fn red_arrow14_typed_property_arrow_keeps_its_property_name() {
    let ir = compile_ts("class Example { load: Handler = (value: Item) => { save(value); }; }");
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("save".to_string(), 1)],
        "a type annotation does not separate the name from the arrow value"
    );
}

#[test]
fn red_arrow15_multiline_parenthesized_arrow_keeps_stable_ownership() {
    let multiline = compile_ts(
        "class Example {\n    load =\n        (\n            value: Item,\n        ) =>\n            save(value);\n}\n",
    );
    let compact = compile_ts("class Example { load = (value: Item) => save(value); }");
    assert_eq!(
        calls_by(&multiline, "load"),
        calls_by(&compact, "load"),
        "formatting must not change ownership"
    );
    assert_eq!(calls_by(&multiline, "load"), vec![("save".to_string(), 1)]);
}

// ─ RED-ARROW19..RED-ARROW23: binding forms and honest omissions ──────

#[test]
fn red_arrow19_top_level_named_arrow_is_supported_with_binding_identity() {
    let ir = compile_ts("const load = () => save();");
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("save".to_string(), 0)],
        "a top-level binding name is a stable callable identity"
    );
    // No enclosing type exists, so the callable is hosted by the same synthetic
    // file class a top-level `function` declaration already uses.
    assert!(
        ir.instructions
            .iter()
            .any(|op| matches!(op, CoreOp::DefClass(_, name) if name.starts_with("__file_"))),
        "a top-level callable belongs to the file-wide synthetic class"
    );
    assert_eq!(
        caller_names(&ir),
        vec!["load".to_string()],
        "the caller is the binding, never the file"
    );
}

#[test]
fn red_arrow20_local_bound_arrow_becomes_the_innermost_callable() {
    let ir = compile_ts(
        "class Example { \
           load(): void { \
             const process = () => { save(); }; \
             process(); \
           } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "process"),
        vec![("save".to_string(), 0)],
        "the named local arrow is the innermost recognized callable"
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![("process".to_string(), 0)],
        "the enclosing method still owns its own invocation"
    );
    assert_eq!(
        caller_names(&ir),
        vec!["load".to_string(), "process".to_string()],
        "two owners, no third invented one"
    );
}

#[test]
fn red_arrow21_anonymous_top_level_arrow_fabricates_no_caller() {
    let ir = compile_ts("items.map(x => transform(x));");
    assert!(
        call_facts(&ir).is_empty(),
        "an anonymous top-level arrow has no representable owner"
    );

    let iife = compile_ts("(() => { save(); })();");
    assert!(
        call_facts(&iife).is_empty(),
        "an IIFE body has no representable owner either"
    );
}

#[test]
fn red_arrow22_anonymous_initializer_callback_fabricates_no_owner() {
    // A property whose value is an expression (not an arrow) declares no
    // callable, so the callbacks inside its initializer are unowned.
    let property = compile_ts("class Example { vm$ = source.pipe(map(x => transform(x))); }");
    assert!(
        call_facts(&property).is_empty(),
        "no enclosing callable exists for a field initializer's callbacks"
    );
    assert!(
        declared_names(&property).is_empty(),
        "no callable may be invented for the initializer"
    );

    // An object-literal property arrow is deliberately NOT a callable owner:
    // its invocations belong to the enclosing recognized callable (RED-ARROW7),
    // and at top level no such callable exists, so nothing is emitted.
    let object = compile_ts("const handlers = { save: () => persist() };");
    assert!(
        call_facts(&object).is_empty(),
        "an object-literal property arrow must not become a synthetic caller"
    );
    assert!(
        declared_names(&object).is_empty(),
        "`save` must not be invented as a callable from an object literal"
    );
}

#[test]
fn red_arrow23_arrow_ownership_composes_with_spread_evidence() {
    let ir = compile_ts("class Example { load = () => { save(...args); }; }");
    assert_eq!(
        calls_shape_by(&ir, "load"),
        vec![("save".to_string(), 1, true)],
        "one written argument that expands, owned by the arrow"
    );
    assert_eq!(calls_by(&ir, "load"), vec![("save".to_string(), 1)]);
}

// ─ RED-ARROW25..RED-ARROW26: one parse, no consumer drift ────────────

#[test]
fn red_arrow25_arrow_captures_cost_no_additional_parse() {
    reset_parse_count();
    let ir =
        compile_ts("class Example { load = () => { save(); }; } const helper = () => update();");
    assert_eq!(
        parse_count(),
        1,
        "the arrow captures must ride the single existing parse"
    );
    assert_eq!(calls_by(&ir, "load"), vec![("save".to_string(), 0)]);
    assert_eq!(calls_by(&ir, "helper"), vec![("update".to_string(), 0)]);
}

#[test]
fn red_arrow26_arrow_captures_do_not_touch_the_base_query_consumers() {
    let source = "class Example { load = () => { save(); }; }";

    // 1. The base query (compression + diff) carries no call or arrow capture.
    let base = run_capture_pipeline(
        crate::compression::language::safe_typescript_language().expect("typescript grammar"),
        TS_QUERY,
        source,
        Fidelity::Medium,
        |name, raw, _| Some(format!("{name}:{raw}")),
    )
    .expect("capture pipeline must succeed");
    let base_names: Vec<String> = base.iter().map(|capture| capture.name.clone()).collect();
    assert!(
        base_names.iter().all(|name| !name.starts_with("call.")),
        "TS_QUERY must carry no call capture: {base_names:?}"
    );
    assert!(
        base_names.iter().all(|name| !name.starts_with("arrow.")),
        "TS_QUERY must carry no arrow capture: {base_names:?}"
    );

    // 2. The composed query adds the arrow captures, and every capture the base
    //    consumers observe is identical to the base walk's.
    let composed = run_capture_pipeline(
        crate::compression::language::safe_typescript_language().expect("typescript grammar"),
        &capture_query(TS_QUERY),
        source,
        Fidelity::Medium,
        |name, raw, _| Some(format!("{name}:{raw}")),
    )
    .expect("capture pipeline must succeed");
    assert!(
        composed.iter().any(|capture| capture.name == "arrow.name"),
        "the composed query must bind the arrow name capture"
    );
    assert!(
        composed.iter().any(|capture| capture.name == "arrow.root"),
        "the composed query must bind the arrow declaration capture"
    );

    let mut base_set: Vec<(String, String)> = base
        .iter()
        .map(|capture| (capture.name.clone(), capture.text.clone()))
        .collect();
    let mut composed_base: Vec<(String, String)> = composed
        .iter()
        .filter(|capture| !capture.name.starts_with("call.") && !capture.name.starts_with("arrow."))
        .map(|capture| (capture.name.clone(), capture.text.clone()))
        .collect();
    base_set.sort();
    composed_base.sort();
    assert_eq!(
        base_set, composed_base,
        "adding arrow captures must not change any base capture"
    );

    // 3. The diff snapshot is built from TS_QUERY alone, so no arrow capture can
    //    appear in a method's marker set.
    let snapshot = crate::diff::build_snapshot(source, Fidelity::Medium)
        .expect("the diff snapshot must build");
    let markers: Vec<String> = snapshot
        .classes
        .iter()
        .flat_map(|class| class.methods.iter())
        .flat_map(|method| method.markers.iter().cloned())
        .collect();
    assert!(
        markers.iter().all(|marker| !marker.contains("call.")),
        "call captures must never reach the diff snapshot: {markers:?}"
    );
    assert!(
        markers.iter().all(|marker| !marker.contains("arrow.")),
        "arrow captures must never reach the diff snapshot: {markers:?}"
    );
}
