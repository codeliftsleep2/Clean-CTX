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

// ── C# return-type-first syntax ───────────────────────────────────

#[test]
fn parse_sig_csharp_return_type_first() {
    let sig = parse_method_sig("ActionResult<UserDto> GetAll(int id)");
    assert_eq!(sig.name, "GetAll");
    assert_eq!(sig.params_str, "int id");
    assert_eq!(sig.return_type, "$v");
}

#[test]
fn parse_sig_csharp_generic_return_type_first() {
    let sig = parse_method_sig("ActionResult<IEnumerable<UserDto>> GetAll()");
    assert_eq!(sig.name, "GetAll");
    assert_eq!(sig.params_str, "");
}

#[test]
fn parse_sig_csharp_with_modifiers() {
    let sig = parse_method_sig("public async Task<IActionResult> Create([FromBody] CreateUserRequest request)");
    assert_eq!(sig.name, "Create");
    assert!(sig.params_str.contains("request"));
}

// ── Edit Mode regression tests (F-32) ──────────────────────────────

/// Edit Mode: a multiline signature with the `{` on its own line must
/// preserve the brace's leading indentation (byte-exact body). The
/// previous implementation started at the `{` itself, dropping the
/// indentation so the opening brace was column 0 while nested braces
/// kept their indentation.
#[test]
fn extract_body_preserves_own_line_brace_indentation() {
    let raw = "function foo()\n    {\n        let x = 1;\n    }";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "    {\n        let x = 1;\n    }");
}

/// Edit Mode: a same-line brace still starts at the `{` (the signature
/// is emitted separately by the renderer — including it would duplicate
/// the method declaration).
#[test]
fn extract_body_same_line_brace_starts_at_brace() {
    let raw = "function foo() {\n  let x = 1;\n}";
    let body = extract_method_body(raw).expect("should extract body");
    assert_eq!(body, "{\n  let x = 1;\n}");
}

/// Edit Mode: `emit_method_ir` must NOT swallow the body as the return
/// type when given the full raw method text (the legacy pipeline's
/// `extract_method_sig` returns full text at Edit fidelity). The body
/// flows through `CoreOp::Body` separately; the Return op must carry
/// only the real return type. This guards the double-render bug where
/// the renderer emitted a garbled `→ ... { body }` immediately followed
/// by the verbatim body.
#[test]
fn emit_method_ir_strips_body_from_sig() {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::opcodes::CoreOp;

    let mut compiler = IRCompiler::new();
    let mut instructions = Vec::new();
    // Full raw method text as produced by extract_method_sig at Edit.
    let raw = "public async getUser(id: string): Promise<User> {\n  return this.users[id];\n}";
    compiler.emit_method_ir(&mut instructions, "C1", "M1", raw);

    // DefMethod + Param + Return — no body in the Return op.
    let return_op = instructions.iter().find_map(|op| {
        if let CoreOp::Return(mid, ty) = op {
            Some((mid.clone(), ty.clone()))
        } else {
            None
        }
    });
    let (mid, ty) = return_op.expect("should emit Return");
    assert_eq!(mid, "M1");
    assert_eq!(ty, "Promise<User>");
    // The body must NOT appear in the return type.
    assert!(!ty.contains('{'));
    assert!(!ty.contains("return"));
}

/// Edit Mode FAANG audit: expression-bodied arrows have no block brace,
/// so `find_body_start` returns None. The arrow expression must NOT be
/// swallowed into `return_type` — `emit_method_ir` must strip at `=>`.
#[test]
fn emit_method_ir_strips_arrow_expression_from_sig() {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::opcodes::CoreOp;

    let mut compiler = IRCompiler::new();
    let mut instructions = Vec::new();
    // Full raw arrow text as produced by extract_method_sig at Edit.
    let raw = "private getLabel = (id: string) => this.labels[id] ?? 'default';";
    compiler.emit_method_ir(&mut instructions, "C1", "M2", raw);

    // The arrow expression must NOT appear in the Return op.
    let return_op = instructions.iter().find_map(|op| {
        if let CoreOp::Return(mid, ty) = op {
            Some((mid.clone(), ty.clone()))
        } else {
            None
        }
    });
    let (mid, ty) = return_op.expect("should emit Return");
    assert_eq!(mid, "M2");
    assert!(!ty.contains("=>"), "arrow expression leaked into return type: {}", ty);
    assert!(!ty.contains("labels"), "arrow expression leaked into return type: {}", ty);
}

/// C# attribute handling: `extract_method_body` must strip leading
/// attribute lines from the raw Edit-fidelity text so the body starts
/// at the actual declaration brace (not the attribute's `{`), and
/// `emit_method_ir` must produce a clean DefMethod name.
#[test]
fn extract_method_body_and_emit_ir_strip_csharp_attributes() {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::opcodes::CoreOp;

    // Full raw C# method text as produced by extract_method_sig at Edit.
    let raw = "[HttpGet(\"{id}\")]\npublic ActionResult<UserDto> GetById(int id)\n{\n    return Ok(_userService.GetUserById(id));\n}";

    // Body extraction must ignore the attribute's `{` inside the string
    // literal and start at the real declaration brace.
    let body = extract_method_body(raw).expect("should extract body");
    assert!(body.contains("GetUserById"), "body should contain the real method body: {}", body);
    assert!(!body.contains("HttpGet"), "body should not contain the attribute: {}", body);

    // emit_method_ir must produce a clean name (not `[HttpGet`).
    let mut compiler = IRCompiler::new();
    let mut instructions = Vec::new();
    let name = compiler.emit_method_ir(&mut instructions, "C1", "M3", raw);
    assert_eq!(name, "GetById");

    let def = instructions.iter().find_map(|op| {
        if let CoreOp::DefMethod(cid, mid, n) = op {
            Some((cid.clone(), mid.clone(), n.clone()))
        } else {
            None
        }
    });
    let (cid, mid, n) = def.expect("should emit DefMethod");
    assert_eq!(cid, "C1");
    assert_eq!(mid, "M3");
    assert_eq!(n, "GetById");
}
