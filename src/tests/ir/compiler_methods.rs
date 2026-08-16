// src/tests/ir/compiler_methods.rs
//
// Direct unit tests for `extract_method_body` (Edit Mode verbatim body
// extraction) and `parse_method_sig` (method signature parsing).
//
// These tests guard the H-3 fix: paren-depth tracking so a `{` inside a
// parameter default-value object literal is not mistaken for the body
// opening brace, and the expression-bodied arrow fallback.

use super::*;

// ── extract_method_body ────────────────────────────────────────────

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
    // The body starts at the brace AFTER the closing paren of the params.
    assert_eq!(body, "{\n  return x.a;\n}");
}

/// H-3 regression: nested parens in a param default (e.g. a function call)
/// must be tracked correctly.
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

/// H-3: expression-bodied arrow function — the expression after `=>`
/// is returned as the body.
#[test]
fn extract_body_expression_arrow() {
    let raw = "const foo = () => bar()";
    let body = extract_method_body(raw).expect("should extract arrow body");
    assert_eq!(body, " bar()");
}

/// H-3: expression-bodied arrow with a trailing semicolon.
#[test]
fn extract_body_expression_arrow_with_semicolon() {
    let raw = "const foo = () => bar();";
    let body = extract_method_body(raw).expect("should extract arrow body");
    assert_eq!(body, " bar();");
}

/// H-3: a method with no body and no arrow returns None.
#[test]
fn extract_body_no_body_returns_none() {
    let raw = "function foo()";
    assert!(extract_method_body(raw).is_none());
}

/// H-3: an arrow with an empty expression returns None.
#[test]
fn extract_body_empty_arrow_returns_none() {
    let raw = "const foo = () => ;";
    assert!(extract_method_body(raw).is_none());
}

/// H-3: byte-exactness — the returned body must be a verbatim slice
/// (no trimming, no normalization).
#[test]
fn extract_body_is_byte_exact() {
    let raw = "function foo() {\n  let x = 1;  // comment\n}";
    let body = extract_method_body(raw).unwrap();
    assert_eq!(body, "{\n  let x = 1;  // comment\n}");
    // The body must be a suffix of the raw text (byte-exact slice).
    assert!(raw.ends_with(&body));
}

// ── parse_method_sig ───────────────────────────────────────────────

#[test]
fn parse_sig_basic() {
    let sig = parse_method_sig("processComplexData(payload:$s[],payload2:$n):$b");
    assert_eq!(sig.name, "processComplexData");
    assert_eq!(sig.params_str, "payload:$s[],payload2:$n");
    assert_eq!(sig.return_type, "$b");
}

#[test]
fn parse_sig_no_params() {
    let sig = parse_method_sig("doWork():$v");
    assert_eq!(sig.name, "doWork");
    assert_eq!(sig.params_str, "");
    assert_eq!(sig.return_type, "$v");
}

#[test]
fn parse_sig_no_return_type_defaults_void() {
    let sig = parse_method_sig("doWork()");
    assert_eq!(sig.name, "doWork");
    assert_eq!(sig.return_type, "$v");
}

#[test]
fn parse_sig_nested_parens_in_param_type() {
    let sig = parse_method_sig("foo(cb:() => void):$b");
    assert_eq!(sig.name, "foo");
    assert_eq!(sig.params_str, "cb:() => void");
    assert_eq!(sig.return_type, "$b");
}

#[test]
fn parse_sig_no_parens_treats_whole_as_name() {
    let sig = parse_method_sig("justAName");
    assert_eq!(sig.name, "justAName");
    assert_eq!(sig.params_str, "");
    assert_eq!(sig.return_type, "$v");
}