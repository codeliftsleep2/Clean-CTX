// src/ir/compiler_methods.rs
//
// Method signature parsing, import IR emission, and forward alias
// resolution extracted from compiler.rs during Phase 5 module split.
//
// Contains:
//   - MethodSig struct and parse_method_sig()
//   - resolve_forward_aliases() post-processor
//   - IRCompiler::emit_method_ir() and emit_import_ir() methods

use super::opcodes::*;

/// Parsed method signature — the result of parsing the string returned
/// by `compaction::extract_method_sig`.
///
/// F-26: This struct formalizes the output shape that `parse_method_sig`
/// returns, making the two-field `(name, params_str, return_type)` tuple
/// self-documenting. The upstream `extract_method_sig` (in `compaction/*`)
/// returns a string of the form `name(params):return_type`, and this
/// function re-parses that string into a structured form. This is an
/// acknowledged duplication — a future refactor could have
/// `extract_method_sig` return this struct directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSig {
    /// Method name (e.g., "processComplexData")
    pub name: String,
    /// Raw parameter list string (e.g., "payload:$s[],payload2:$n")
    pub params_str: String,
    /// Return type opcode (e.g., "$b", "$v")
    pub return_type: String,
}

/// Parse a method signature string into a `MethodSig`.
///
/// Input: "processComplexData(payload:$s[],payload2:$n):$b"
/// Output: MethodSig { name: "processComplexData", params_str: "payload:$s[],payload2:$n", return_type: "$b" }
pub(super) fn parse_method_sig(sig: &str) -> MethodSig {
    let sig = sig.trim();

    // Find the return type separator "):"
    // We need to handle nested parens in param types like "items:string[]"
    let mut paren_depth = 0i32;
    let mut params_start = None;
    let mut params_end = None;

    for (i, ch) in sig.char_indices() {
        match ch {
            '(' if params_start.is_none() => {
                params_start = Some(i);
                paren_depth = 1;
            }
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    params_end = Some(i);
                }
            }
            _ => {}
        }
    }

    let (name, params_str, return_type) = if let (Some(ps), Some(pe)) = (params_start, params_end) {
        let name = sig[..ps].trim().to_string();
        let params = sig[ps + 1..pe].trim().to_string();
        let rt = sig[pe + 1..].trim();
        let rt = if let Some(stripped) = rt.strip_prefix(':') {
            stripped.trim().to_string()
        } else if rt.is_empty() {
            TYPE_VOID.to_string()
        } else {
            rt.to_string()
        };
        (name, params, rt)
    } else {
        // No parens found — treat entire string as method name
        (sig.to_string(), String::new(), TYPE_VOID.to_string())
    };

    MethodSig {
        name,
        params_str,
        return_type: if return_type.is_empty() { TYPE_VOID.to_string() } else { return_type },
    }
}

/// Locate the byte index of the brace that opens a method **body**.
///
/// The `raw_text` for a `method.root` capture contains the full method
/// including the signature and body. This function locates the brace
/// that opens the **body** (not a parameter default-value object
/// literal, e.g. `function foo(x = {a:1}) { ... }`, and not a
/// return-type object literal, e.g. `function foo(): { a: number }`).
///
/// Returns `Some(index)` pointing at the `{` character, or `None` when
/// the method has no block body (expression-bodied arrow functions).
pub(super) fn find_body_start(raw_method: &str) -> Option<usize> {
    // Track paren depth so a `{` inside a parameter default (e.g.
    // `x = {a:1}`) is not mistaken for the body opening brace.
    let mut paren_depth = 0i32;
    // Track brace depth for return-type object literals (TS):
    // `function foo(): { a: number } { ... }` — the first `{` after
    // the `:` is a return type, not the body.
    let mut brace_depth = 0i32;
    // Set when we see `:` at paren depth 0 (a return type follows).
    // Cleared when the next non-whitespace char is not `{`.
    let mut pending_return_brace = false;
    for (i, ch) in raw_method.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth = (paren_depth - 1).max(0);
                if paren_depth == 0 {
                    pending_return_brace = false;
                }
            }
            ':' if paren_depth == 0 && brace_depth == 0 => {
                pending_return_brace = true;
            }
            '{' if paren_depth == 0 && brace_depth == 0 && !pending_return_brace => {
                return Some(i);
            }
            '{' if paren_depth == 0 && pending_return_brace => {
                // Return-type object literal — track its brace depth.
                brace_depth += 1;
                pending_return_brace = false;
            }
            '}' if paren_depth == 0 && brace_depth > 0 => {
                brace_depth -= 1;
            }
            // Any other non-whitespace char at paren 0 clears the
            // pending return-type flag (e.g. `: void { ... }`).
            _ if paren_depth == 0 && pending_return_brace && !ch.is_whitespace() => {
                pending_return_brace = false;
            }
            _ => {}
        }
    }
    None
}

/// Extract the verbatim method body from a raw method capture.
///
/// The `raw_text` for a `method.root` capture contains the full method
/// including the signature and body. This function locates the brace
/// that opens the **body** (not a parameter default-value object
/// literal, e.g. `function foo(x = {a:1}) { ... }`) and returns
/// everything from the **start of the line containing that brace** to
/// the end of the string (inclusive). Starting at the line start
/// preserves the brace's original leading indentation so the body is
/// byte-exact for `replace_in_file` SEARCH blocks.
///
/// Expression-bodied arrows (`const foo = () => bar()`) have no block
/// body; the expression following `=>` (with its line prefix) is
/// returned as the body so edit mode still carries the implementation.
///
/// The returned text is byte-exact — no trimming, no normalization.
/// This is the text that `replace_in_file` SEARCH blocks can safely
/// match against.
pub(super) fn extract_method_body(raw_method: &str) -> Option<String> {
    if let Some(i) = find_body_start(raw_method) {
        // If the `{` is the first non-whitespace character on its line
        // (multiline signature like `foo()\n    {`), start at the LINE
        // START so the opening brace keeps its original leading
        // indentation (the user-visible bug: nested braces were
        // indented but the first `{` was column 0). If the brace is on
        // the same line as the signature (`foo() {`), start at the
        // brace itself — the signature is emitted separately by the
        // renderer, so including it here would duplicate it.
        let line_start = raw_method[..i].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let prefix = &raw_method[line_start..i];
        if prefix.trim().is_empty() {
            return Some(raw_method[line_start..].to_string());
        }
        return Some(raw_method[i..].to_string());
    }

    // No block body — check for an expression-bodied arrow function.
    // The capture for TS/JS `const foo = () => bar()` ends with the
    // expression (optionally `;`). Return the arrow expression so the
    // body is not silently dropped in edit mode.
    if let Some(arrow_idx) = raw_method.rfind("=>") {
        let expr = &raw_method[arrow_idx + 2..];
        let trimmed = expr.trim();
        if !trimmed.is_empty() && trimmed != ";" {
            return Some(expr.to_string());
        }
    }

    None
}

/// F-FULL-08: Post-process the IR stream to resolve forward-declared class
/// aliases. When class B extends class A, but A is defined later in the file,
/// the TypeScript layer emits `Extends("C2", "A")` where "A" is a raw class
/// name (not an alias ID). This function builds a mapping from class name →
/// alias ID from the `DefClass` ops in the stream, then rewrites any
/// `Extends`/`Implements` ops that reference a raw class name.
pub(super) fn resolve_forward_aliases(instructions: &mut [CoreOp]) {
    // First pass: build the class-name → alias-id mapping from DefClass ops.
    let mut name_to_alias: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for op in instructions.iter() {
        if let CoreOp::DefClass(alias_id, name) = op {
            // name is the extracted class name like "FooService" (no modifiers).
            // Store the mapping so we can resolve Extends("C1", "Foo") → Extends("C1", "C2").
            if !name_to_alias.contains_key(name) {
                name_to_alias.insert(name.clone(), alias_id.clone());
            }
        }
    }

    // Second pass: rewrite Extends/Implements ops that reference raw class names.
    for op in instructions.iter_mut() {
        match op {
            CoreOp::Extends(_, target) => {
                // If target looks like a raw class name (not an alias ID starting with "C"),
                // try to resolve it to the alias ID.
                if !target.starts_with('C') {
                    if let Some(alias) = name_to_alias.get(target.as_str()) {
                        *target = alias.clone();
                    }
                }
            }
            CoreOp::Implements(_, target)
                if !target.starts_with('C') => {
                    if let Some(alias) = name_to_alias.get(target.as_str()) {
                        *target = alias.clone();
                    }
                }
            _ => {}
        }
    }
}

impl super::compiler::IRCompiler {
    /// Parse a method signature and emit DefMethod + Param + Return instructions.
    ///
    /// Accepts signatures in the format produced by `extract_method_sig`:
    ///   - `methodName(param1:$t,param2:$t):$t`
    ///   - `methodName():$t`
    pub(super) fn emit_method_ir(
        &mut self,
        instructions: &mut Vec<CoreOp>,
        class_id: &str,
        method_id: &str,
        raw_sig: &str,
    ) {
        // Parse the method signature: "name(params):return_type"
        //
        // At Edit/Verbatim fidelity, `extract_method_sig` (in
        // `compaction/method.rs`) intentionally returns the FULL raw
        // method text so the legacy text pipeline can emit bodies
        // byte-exact. The IR compiler must NOT pass that full text to
        // `parse_method_sig`, otherwise the body becomes part of the
        // `return_type` (everything after the closing `)`) and the
        // renderer duplicates it (garbled `→ ... { body }` immediately
        // followed by the verbatim body). Strip the body here so the
        // signature is parsed from the signature-only portion; the
        // verbatim body flows through `CoreOp::Body` separately.
        let sig_text = match find_body_start(raw_sig) {
            Some(i) => raw_sig[..i].trim_end().to_string(),
            None => {
                // Expression-bodied arrow: `const foo = () => bar()`.
                // No block brace exists, so strip everything from the
                // `=>` — the arrow expression flows through `CoreOp::Body`
                // via `extract_method_body` (which returns the text after
                // `=>`). Without this strip the arrow expression would be
                // swallowed into `return_type` and double-rendered.
                if let Some(arrow_idx) = raw_sig.rfind("=>") {
                    raw_sig[..arrow_idx].trim_end().to_string()
                } else {
                    raw_sig.to_string()
                }
            }
        };
        let sig = parse_method_sig(&sig_text);
        let name = sig.name;
        let params_str = sig.params_str;
        let return_type = sig.return_type;

        // Emit DefMethod
        instructions.push(CoreOp::DefMethod(
            class_id.to_string(),
            method_id.to_string(),
            name,
        ));

        // Emit Param instructions for each parameter
        if !params_str.is_empty() {
            for param in params_str.split(',') {
                let param = param.trim();
                if param.is_empty() {
                    continue;
                }
                // Parse "name:$type" or just "name"
                let (param_name, param_type) = if let Some(colon_pos) = param.find(':') {
                    let pname = param[..colon_pos].trim().to_string();
                    let ptype = param[colon_pos + 1..].trim().to_string();
                    (pname, ptype)
                } else {
                    (param.to_string(), TYPE_VOID.to_string())
                };

                let param_id = self.next_id("P");
                instructions.push(CoreOp::Param(
                    method_id.to_string(),
                    param_id,
                    param_type,
                    param_name,
                ));
            }
        }

        // Emit Return
        instructions.push(CoreOp::Return(
            method_id.to_string(),
            return_type,
        ));
    }

    /// Emit import IR from a raw import line.
    pub(super) fn emit_import_ir(&mut self, instructions: &mut Vec<CoreOp>, raw: &str) {
        // Raw import looks like: `import { Foo } from 'module'`
        // or compacted: `$im Foo.$fmmodule` / `$im $fm module`
        // We try to parse the compacted format first.
        let trimmed = raw.trim();

        // Try to parse: "$im <named>.$fm <module>" pattern
        if let Some(rest) = trimmed.strip_prefix("$im ") {
            if let Some(fm_pos) = rest.find(".$fm") {
                let named = rest[..fm_pos].trim().to_string();
                let module = rest[fm_pos + 4..].trim().to_string();
                let alias = self.next_id("IM");
                instructions.push(CoreOp::Import(alias, module, named));
                return;
            }
            // Just "$im something" without .$fm
            let named = rest.trim().to_string();
            let alias = self.next_id("IM");
            instructions.push(CoreOp::Import(alias, String::new(), named));
            return;
        }

        // Try to parse standard ES import: "import { X } from 'module'"
        if let Some(from_pos) = trimmed.find(" from ") {
            let named_part = trimmed[..from_pos].trim();
            let module_part = trimmed[from_pos + 6..].trim().trim_matches('\'').trim_matches('"');
            // Extract named imports: "import { Foo, Bar } from ..."
            let named = if let Some(start) = named_part.find('{') {
                if let Some(end) = named_part.find('}') {
                    named_part[start + 1..end].trim().to_string()
                } else {
                    named_part.to_string()
                }
            } else {
                named_part.to_string()
            };
            let alias = self.next_id("IM");
            instructions.push(CoreOp::Import(alias, module_part.to_string(), named));
            return;
        }

        // Fallback: just emit as-is
        let alias = self.next_id("IM");
        instructions.push(CoreOp::Import(alias, String::new(), trimmed.to_string()));
    }
}

#[cfg(test)]
#[path = "../tests/ir/compiler_methods.rs"]
mod tests;
