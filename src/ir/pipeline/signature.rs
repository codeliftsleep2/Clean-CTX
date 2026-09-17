// src/ir/pipeline/signature.rs
//
// METHOD-DECLARATION parsing and body location — the declaration-level helpers
// `CoreIRPass` consumes, split out of `pipeline.rs` (active-file size policy)
// along a real responsibility boundary: "read a declaration" versus "run the
// pass pipeline". Every body below is an unchanged relocation, and the
// pipeline re-exports the public names so `crate::ir::pipeline::{MethodSig,
// find_body_start_in, locate_method_body}` keep resolving.
//
// The structural identity rules these helpers apply (which identifier owns the
// parameter list, which prefix declares the return type) live in
// `compaction::signature` and are consumed here — this module adds no second
// parsing opinion.

use crate::compaction::method::{find_method_params, strip_base_initializer_clause};
use crate::compaction::modifiers::strip_csharp_attributes;
use crate::compaction::signature::{return_type_from_prefix, split_head_parts};
use crate::ir::opcodes::TYPE_VOID;

/// Parsed method signature — the result of parsing the string returned
/// by `compaction::extract_method_sig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSig {
    pub name: String,
    pub params_str: String,
    pub return_type: String,
}

/// Locate the byte index of the brace that opens a method body.
pub(crate) fn find_body_start_in(raw_method: &str) -> Option<usize> {
    let mut paren_depth = 0i32;
    let mut brace_depth = 0i32;
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
                brace_depth += 1;
                pending_return_brace = false;
            }
            '}' if paren_depth == 0 && brace_depth > 0 => {
                brace_depth -= 1;
            }
            _ if paren_depth == 0 && pending_return_brace && !ch.is_whitespace() => {
                pending_return_brace = false;
            }
            _ => {}
        }
    }
    None
}

/// Extract the verbatim method body from a raw method capture *and* the
/// byte offset at which that body slice begins within `raw_method`
/// (apply_edit plan Phase 1).
///
/// The offset lets the IR compiler emit `CoreOp::Body` ops whose span
/// fields address the exact source bytes the text came from:
/// `absolute_start = capture.start_byte + offset`, and the slice always
/// runs to the end of the capture, so `absolute_end = capture.end_byte`.
///
/// Block bodies are located within the attribute-stripped view; C#
/// attribute stripping only removes a leading prefix, so pointer
/// arithmetic maps the stripped-view index back onto the original bytes.
/// Expression (`=>`) bodies are located directly on the raw capture,
/// preserving the historical behavior of `extract_method_body`.
pub(crate) fn locate_method_body(raw_method: &str) -> Option<(String, usize)> {
    let stripped = strip_csharp_attributes(raw_method);
    // strip_csharp_attributes returns a subslice of its input (leading
    // trim/strip only), so this pointer diff is the byte offset of the
    // stripped view inside the original capture.
    let stripped_offset = stripped.as_ptr() as usize - raw_method.as_ptr() as usize;

    if let Some(i) = find_body_start_in(stripped) {
        // Body units are BRACE-DELIMITED: text and span start AT the
        // opening `{`, never at the line start. The previous behavior
        // backed up to the start of the line when `{` sat alone (Allman
        // style / brace-on-next-line), embedding leading indentation in
        // the tracked body — so every natural agent extraction (`{`
        // through `}`) was rejected as a permanent byte-count mismatch.
        // Regression: src/tests/edit/spans.rs
        // `lf_csharp_allman_attributes_spans_address_exact_disk_bytes`.
        return Some((stripped[i..].to_string(), stripped_offset + i));
    }

    if let Some(arrow_idx) = raw_method.rfind("=>") {
        let expr_start = arrow_idx + "=>".len();
        let expr = &raw_method[expr_start..];
        let trimmed = expr.trim();
        if !trimmed.is_empty() && trimmed != ";" {
            return Some((expr.to_string(), expr_start));
        }
    }

    None
}

/// Parse a method signature string into a `MethodSig`.
///
/// The name, its type parameters, and the return type are derived
/// STRUCTURALLY — from the declaration's own parameter list (located by
/// `find_method_params`, which skips a parenthesized return type) and from
/// balanced angle-bracket structure (`compaction::signature`) — never from
/// whitespace-token position. `Pair<TFirst, TSecond>` contains a `, `
/// inside its type-parameter list, so a token-position rule reads the name
/// as `TSecond>`; a tuple return type puts a modifier before the name, so
/// the same rule reads it as `static`.
pub(super) fn parse_method_sig(sig: &str) -> MethodSig {
    let sig = sig.trim();
    // C# constructor initializers (`: base(...)` / `: this(...)`) are
    // call sites, never signature content: drop the clause so the name,
    // params, and return type derive from the bare declaration — a
    // base/this call must not become a synthesized "return type".
    let sig = strip_base_initializer_clause(sig);

    let (name, params_str, return_type) = if let Some((ps, pe)) = find_method_params(sig) {
        let params = sig[ps + 1..pe].trim().to_string();
        let head_parts = split_head_parts(sig, ps);
        let name = match head_parts {
            Some(parts) => parts.name.to_string(),
            // Defensive: an unrecognized head keeps the legacy reading.
            //
            // No `trim()` before `split_whitespace()`: that iterator already
            // skips leading/trailing whitespace, so the token sequence is
            // identical. The `unwrap_or` fallback still trims, because it is
            // the value returned when the head holds no token at all.
            None => sig[..ps]
                .split_whitespace()
                .last()
                .unwrap_or(sig[..ps].trim())
                .to_string(),
        };
        let tail = sig[pe + 1..].trim();
        let rt = if let Some(stripped) = tail.strip_prefix(':') {
            stripped.trim().to_string()
        } else if !tail.is_empty() {
            // `-> T` (Rust) and any other trailing annotation.
            tail.to_string()
        } else {
            // Nothing follows the parameter list, so the declaration is
            // return-type-first (C#): take the declared type read from the
            // prefix, with modifiers and the declaration's own
            // type-parameter list excluded.
            head_parts
                .and_then(|parts| return_type_from_prefix(parts.prefix))
                .map(str::to_string)
                .unwrap_or_else(|| TYPE_VOID.to_string())
        };
        (name, params, rt)
    } else {
        (sig.to_string(), String::new(), TYPE_VOID.to_string())
    };

    MethodSig {
        name,
        params_str,
        return_type: if return_type.is_empty() {
            TYPE_VOID.to_string()
        } else {
            return_type
        },
    }
}
