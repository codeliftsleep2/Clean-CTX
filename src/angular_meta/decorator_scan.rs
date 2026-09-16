// src/angular_meta/decorator_scan.rs
//
// Textual decorator-token scanning for the Angular meta-layer.
//
// Extracted from `decorators.rs` (active-file size policy) as the cohesive
// unit that owns the raw text walkers locating `@Decorator(...)` occurrences
// in a class head and `@Input(...)` / `@Output(...)` occurrences in a class
// body, plus their shared token classification.
//
// Responsibilities deliberately NOT in this module:
// - decorator *argument* parsing          → `decorator_args.rs`
// - constructor dependency extraction     → `constructor_injects.rs`
// - Φ marker construction / orchestration → `decorators.rs`, `markers.rs`
//
// Strategy is unchanged from the original implementation: the TS capture
// pipeline already produced the class span, so an O(L) byte walk over the
// bounded span is sufficient — no AST re-parse.

use crate::angular_meta::decorator_args::parse_first_string_arg;
use crate::meta_util::consume_call_expression;

/// A collected decorator token. `name` is populated for potential
/// future inspection/debug use but the dispatch only consumes `kind`
/// and `arg` today, so it is kept under `#[allow(dead_code)]`.
#[allow(dead_code)]
pub(crate) struct Decorator {
    pub(crate) name: String,
    pub(crate) arg: String,
    pub(crate) kind: DecoratorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecoratorKind {
    Component,
    Injectable,
    NgModule,
    Directive,
    Pipe,
    Input,
    Output,
    Other,
}

pub(crate) fn collect_decorators(head: &str) -> Vec<Decorator> {
    let mut decorators: Vec<Decorator> = Vec::new();
    let bytes = head.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] != b'@' {
            i += 1;
            continue;
        }
        i += 1;
        let name_start = i;
        while i < len {
            let c = bytes[i];
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
                i += 1;
            } else {
                break;
            }
        }
        if i == name_start {
            continue;
        }
        let name = &head[name_start..i];
        while i < len && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n') {
            i += 1;
        }
        let mut arg = String::new();
        // F-ANG-09: if the decorator call is unterminated, advance
        // past the `(` to avoid an infinite loop and use an empty
        // arg. The audit's deferred fix note says callers tolerate
        // the fallback gracefully.
        if i < len && bytes[i] == b'(' {
            if let Some((consumed, arg_str)) = consume_call_expression(head, i) {
                i += consumed;
                arg = arg_str;
            } else {
                i += 1;
            }
        }
        let kind = classify_decorator(name);
        decorators.push(Decorator {
            name: name.to_string(),
            arg,
            kind,
        });
    }

    decorators
}

// F-ANG-09: `consume_call_expression` now comes from the shared
// layer-agnostic `meta_util` primitive set (Round-8 structural audit).
// It returns `None` if the call expression is unterminated (was
// returning `i-open_paren` and slicing to end of text — silent EOF
// behaviour).

pub(crate) fn classify_decorator(name: &str) -> DecoratorKind {
    match name {
        "Component" => DecoratorKind::Component,
        "Injectable" => DecoratorKind::Injectable,
        "NgModule" => DecoratorKind::NgModule,
        "Directive" => DecoratorKind::Directive,
        "Pipe" => DecoratorKind::Pipe,
        "Input" => DecoratorKind::Input,
        "Output" => DecoratorKind::Output,
        _ => DecoratorKind::Other,
    }
}

/// True when `c` can be part of a TypeScript identifier word.
///
/// Shared by the scanners that must respect word boundaries — e.g.
/// `constructor` must not match inside `constructorLike(...)`.
pub(crate) fn is_word_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Walk the class body and collect all `@Input(...)` /
/// `@Output(...)` decorator occurrences, pairing each decorator
/// with the field declaration line that follows it.
pub(crate) fn collect_field_decorators(body: &str) -> Vec<(DecoratorKind, Option<String>, String)> {
    let mut out: Vec<(DecoratorKind, Option<String>, String)> = Vec::new();
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] != b'@' {
            i += 1;
            continue;
        }
        i += 1;
        let name_start = i;
        while i < len {
            let c = bytes[i];
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'$' {
                i += 1;
            } else {
                break;
            }
        }
        if i == name_start {
            continue;
        }
        let name = &body[name_start..i];
        while i < len && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n') {
            i += 1;
        }
        let mut arg = String::new();
        // F-ANG-09: same pattern as `collect_decorators` — advance
        // past `(` on unterminated call, use empty arg.
        if i < len && bytes[i] == b'(' {
            if let Some((consumed, arg_str)) = consume_call_expression(body, i) {
                i += consumed;
                arg = arg_str;
            } else {
                i += 1;
            }
        }
        let kind = match name {
            "Input" => Some(DecoratorKind::Input),
            "Output" => Some(DecoratorKind::Output),
            _ => None,
        };
        if let Some(k) = kind {
            while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
                i += 1;
            }
            let field_start = i;
            while i < len {
                let c = bytes[i];
                if c == b'\n' || c == b'{' || c == b'=' || c == b';' || c == b':' {
                    break;
                }
                i += 1;
            }
            let field_segment = body[field_start..i].trim();
            let field_name = field_segment
                .split_whitespace()
                .next()
                .unwrap_or("?")
                .to_string();
            let field_name = if field_name.is_empty() {
                "?".to_string()
            } else {
                field_name
            };
            let alias = parse_first_string_arg(&arg);
            out.push((k, alias, field_name));
        }
    }
    out
}
