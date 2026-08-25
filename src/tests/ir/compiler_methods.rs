// src/tests/ir/compiler_methods.rs
//
// Tests for functions that migrated from compiler_methods.rs to pipeline.rs
// during the PassPipeline migration. The original compiler_methods.rs now
// only contains resolve_forward_aliases. These tests cover the function-level
// behavior of find_body_start_in and extract_method_body which moved to pipeline.rs.

use crate::ir::pipeline::{find_body_start_in, extract_method_body};

// ── find_body_start_in ───────────────────────────────────────────

/// Basic function with block body.
#[test]
fn find_body_start_basic_block() {
    let raw = "function foo() {\n  let x = 1;\n}";
    let result = find_body_start_in(raw);
    assert!(result.is_some());
    // The brace after the closing paren of params
    assert_eq!(&raw[result.unwrap()..result.unwrap() + 1], "{");
}

/// H-3 regression: a `{` inside a parameter default-value object literal
/// must NOT be treated as the body opening brace.
#[test]
fn find_body_start_skips_param_default_object_literal() {
    let raw = "function foo(x = {a: 1, b: 2}) {\n  return x.a;\n}";
    let result = find_body_start_in(raw);
    assert!(result.is_some());
    let brace_index = result.unwrap();
    // The body brace is the one AFTER the closing paren — the `{` at the end
    // of "..., b: 2}) " — there should be a `)` before the brace
    assert!(raw[..brace_index].contains(')'), "brace should be after the closing paren");
}

/// H-3 regression: nested parens in a param default must be tracked correctly.
#[test]
fn find_body_start_skips_nested_parens_in_param_default() {
    let raw = "function foo(x = bar(1, 2)) {\n  return x;\n}";
    let result = find_body_start_in(raw);
    assert!(result.is_some());
    let brace_index = result.unwrap();
    // The body brace should be after the closing paren of bar(1, 2))
    assert!(raw[..brace_index].contains(')'));
}

/// H-3 regression: a `{` inside a return-type object literal (TS) must
/// NOT be treated as the body opening brace.
#[test]
fn find_body_start_skips_return_type_object_literal() {
    let raw = "function foo(): { a: number } {\n  return { a: 1 };\n}";
    let result = find_body_start_in(raw);
    assert!(result.is_some());
    let brace_index = result.unwrap();
    // Should skip the return type `{ a: number }` and find the body `{`
    // The body brace should come after the return type close `}`
    assert!(raw[..brace_index].contains("number } {"));
}

/// Expression-bodied arrow function — no block body.
#[test]
fn find_body_start_expression_arrow_returns_none() {
    let raw = "const foo = () => bar()";
    let result = find_body_start_in(raw);
    assert!(result.is_none());
}

// ── extract_method_body ──────────────────────────────────────────

/// Basic block body: everything from the first `{` to the end (inclusive).
#[test]
fn extract_body_basic_block() {
    let raw = "function foo() {\n  let x = 1;\n  return x;\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "{\n  let x = 1;\n  return x;\n}");
}

/// H-3 regression: a `{` inside a parameter default-value object literal
/// must NOT be treated as the body opening brace.
#[test]
fn extract_body_skips_param_default_object_literal() {
    let raw = "function foo(x = {a: 1, b: 2}) {\n  return x.a;\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "{\n  return x.a;\n}");
}

/// H-3 regression: nested parens in a param default must be tracked correctly.
#[test]
fn extract_body_skips_nested_parens_in_param_default() {
    let raw = "function foo(x = bar(1, 2)) {\n  return x;\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "{\n  return x;\n}");
}

/// H-3 regression: a `{` inside a return-type object literal (TS) must
/// NOT be treated as the body opening brace.
#[test]
fn extract_body_skips_return_type_object_literal() {
    let raw = "function foo(): { a: number } {\n  return { a: 1 };\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "{\n  return { a: 1 };\n}");
}

/// Expression-bodied arrow function — returns the expression after `=>`.
#[test]
fn extract_body_expression_arrow() {
    let raw = "const foo = () => bar()";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, " bar()");
}

/// Expression-bodied arrow with semicolon — semicolon excluded.
#[test]
fn extract_body_expression_arrow_with_semicolon() {
    let raw = "const foo = () => bar();";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, " bar()");
}

/// No body at all returns None.
#[test]
fn extract_body_no_body_returns_none() {
    let raw = "declare function foo(): void;";
    let body = extract_method_body(raw);
    assert!(body.is_none());
}

/// Empty arrow returns None.
#[test]
fn extract_body_empty_arrow_returns_none() {
    let raw = "const foo = () => ;";
    let body = extract_method_body(raw);
    assert!(body.is_none());
}

/// Byte-exact preservation.
#[test]
fn extract_body_is_byte_exact() {
    let raw = "function foo(  )\n  {\n    let x = 1;\n    return x;\n  }";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "  {\n    let x = 1;\n    return x;\n  }");
}

/// Multiline signature: brace on its own line — preserves leading indentation.
#[test]
fn extract_body_preserves_own_line_brace_indentation() {
    let raw = "function foo(param1: string)\n    {\n  console.log('test');\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert!(body.starts_with("    {"), "body should start with indented brace");
}

/// Same-line brace: starts at the brace itself (signature emitted separately).
#[test]
fn extract_body_same_line_brace_starts_at_brace() {
    let raw = "function foo(param1: string) {\n  console.log('test');\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(&body[..1], "{", "body should start with brace");
}

/// C# attributes are stripped before body detection.
#[test]
fn extract_method_body_and_emit_ir_strip_csharp_attributes() {
    let raw = "[HttpGet]\n[Route(\"api/[controller]\")]\npublic string Get(int id) {\n  return \"ok\";\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "{\n  return \"ok\";\n}");
}

// ── parse_method_sig via PassContext ─────────────────────────────
// Remaining tests for parse_method_sig behavior can be added here
// by constructing a PassContext and calling its emit_method_ir, or
// by testing the emission through CoreIRPass with known source.