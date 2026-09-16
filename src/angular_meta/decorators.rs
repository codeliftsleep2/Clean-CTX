// src/angular_meta/decorators.rs
//
// Angular decorator extraction — Tier 1 + 2.5b of the Meta-Layer.
//
// Given the raw text of a `class_declaration` capture, extract every
// Angular decorator and emit one `Φ` marker line per class.
// Also detects signal-based function calls (`input()`, `output()`,
// `model()`, `inject()`) in the class body (Phase 2.5b).
//
// # Strategy
//
// We do NOT re-parse the AST. The TS capture pipeline already
// produced a `class.root` capture that may include the leading
// `@…` decorators. We scan the text **before** the first
// occurrence of the `class <Name>` keyword, collect every
// `@…(...)` decorator, classify it, and emit the appropriate `Φ`
// marker lines.
//
// The string walker is O(L) where L is the length of the class
// capture, which is bounded by the class body length.

use crate::angular_meta::constructor_injects::extract_constructor_injects;
use crate::angular_meta::decorator_args::{
    parse_first_string_arg, parse_module_fields, parse_object_literal, parse_pipe_fields,
    parse_provided_in,
};
use crate::angular_meta::decorator_scan::{
    DecoratorKind, collect_decorators, collect_field_decorators, is_word_byte,
};
use crate::angular_meta::markers::{
    build_component_line, build_directive_line, build_injects_line, build_input_line,
    build_model_line, build_module_line, build_output_line, build_pipe_line, build_service_line,
};
use crate::compression::Fidelity;
use crate::meta_util::consume_call_expression;

/// The kind of Angular class that can be extracted from decorators.
/// Used as a private extraction helper — not a shared architectural type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassKind {
    Component,
    Service,
    Directive,
    Pipe,
    Module,
}
/// Result of [`extract_decorators`]: the Φ marker lines plus any
/// inline template content that should be fed through the HTML
/// template extractor.
pub struct DecoratorsResult {
    /// Φ marker lines (Φcmp:, Φsvc:, etc.).
    pub lines: Vec<String>,
    /// Raw inline template content (from `template: '...'`), when
    /// the component uses an inline template instead of `templateUrl`.
    /// `None` when no inline template is present.
    /// Consumed by `angular_meta::mod::run_meta_layer` when the
    /// `angular` feature is enabled. `#[allow(dead_code)]` is required
    /// because the field is written by `extract_decorators` and read
    /// only by the `angular_meta` module when the feature is on.
    #[allow(dead_code)]
    pub inline_template: Option<String>,
}

/// Extract every Angular decorator on the given class capture and
/// emit the corresponding `Φ` marker lines. Returns `None` if no
/// Angular decorator or signal-based API is present.
///
/// `raw_class` is the text of a `class_declaration` capture as
/// produced by the TS capture pipeline.
///
/// `fidelity` (F-ANG-23) controls the verbosity of the output:
/// - `Low`    → only class-level summaries (`@Component`,
///   `@Injectable`, `@Directive`, `@Pipe`, `@NgModule`). No
///   field-level `@Input` / `@Output`, no `Φinjects:`, no
///   signal-based lines.
/// - `Medium` → add field-level `@Input` / `@Output` markers; skip
///   `Φinjects:` (the class summary already shows the class).
/// - `High`   → emit everything including `Φinjects:` and the
///   modern `input()`/`output()`/`model()`/`inject()` signal lines.
// F-ANG-12: use `?` since the enclosing function returns `Option`
// (clippy::question_mark prefers `?` over `let-else` when both
// apply). The pre-audit behaviour was to slice to `raw.len()` and
// scan the whole string, which always yielded zero decorators
// and returned `None` anyway, just wasted work.
// F-ANG-13: substitute `?` for missing class names at the call site
// (the audit notes the call site already did this implicitly via
// the `"(anonymous)"` literal).
pub fn extract_decorators(raw_class: &str, fidelity: Fidelity) -> Option<DecoratorsResult> {
    let head_end = find_class_head_end(raw_class)?;
    let head = &raw_class[..head_end];

    let class_name = extract_class_name(raw_class).unwrap_or_else(|| "?".to_string());
    let decorators = collect_decorators(head);

    let mut lines: Vec<String> = Vec::new();
    let mut input_output_lines: Vec<String> = Vec::new();
    let mut component_emit: Option<String> = None;
    let mut service_emit: Option<String> = None;
    let mut module_emit: Option<String> = None;
    let mut directive_emit: Option<String> = None;
    let mut pipe_emit: Option<String> = None;
    let mut inline_template: Option<String> = None;

    for dec in &decorators {
        match dec.kind {
            DecoratorKind::Component => {
                let fields = parse_object_literal(&dec.arg);
                // Capture inline template for later shape extraction.
                // Only use when there's no templateUrl (the external
                // .html file takes precedence in workspace mode).
                if inline_template.is_none() {
                    inline_template = fields.template.clone();
                }
                component_emit = Some(build_component_line(&class_name, &fields));
            }
            DecoratorKind::Injectable => {
                let provided_in = parse_provided_in(&dec.arg);
                service_emit = Some(build_service_line(&class_name, provided_in.as_deref()));
            }
            DecoratorKind::NgModule => {
                let (decl, imp, exp) = parse_module_fields(&dec.arg);
                module_emit = Some(build_module_line(&class_name, &decl, &imp, &exp));
            }
            DecoratorKind::Directive => {
                let fields = parse_object_literal(&dec.arg);
                let selector = fields.selector.clone();
                directive_emit = Some(build_directive_line(&class_name, selector.as_deref()));
            }
            DecoratorKind::Pipe => {
                let (name, _pure) = parse_pipe_fields(&dec.arg);
                pipe_emit = Some(build_pipe_line(&class_name, name.as_deref()));
            }
            DecoratorKind::Input => {
                if fidelity != Fidelity::Low {
                    let alias = parse_first_string_arg(&dec.arg);
                    input_output_lines.push(build_input_line("?", alias.as_deref()));
                }
            }
            DecoratorKind::Output => {
                if fidelity != Fidelity::Low {
                    let alias = parse_first_string_arg(&dec.arg);
                    input_output_lines.push(build_output_line("?", alias.as_deref()));
                }
            }
            DecoratorKind::Other => {}
        }
    }

    // Field-level markers: scan the class body for
    // `@Input(...)` / `@Output(...)` decorators attached to
    // individual field declarations.
    // F-ANG-08: skip the body scan if no matching `}` is found.
    // (clippy::collapsible_if: the `if` conditions are combined
    // with `&&` so the nested block is unnecessary.)
    if fidelity != Fidelity::Low
        && let Some(class_body_start) = find_class_body_open(raw_class)
        && let Some(body_end) =
            crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
    {
        let body = &raw_class[class_body_start..];
        let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
        for (kind, alias, field_name) in collect_field_decorators(body_inner) {
            match kind {
                DecoratorKind::Input => {
                    input_output_lines.push(build_input_line(&field_name, alias.as_deref()));
                }
                DecoratorKind::Output => {
                    input_output_lines.push(build_output_line(&field_name, alias.as_deref()));
                }
                _ => {}
            }
        }
    }

    if let Some(line) = component_emit {
        lines.push(line);
    }
    if let Some(line) = service_emit {
        lines.push(line);
    }
    if let Some(line) = module_emit {
        lines.push(line);
    }
    if let Some(line) = directive_emit {
        lines.push(line);
    }
    if let Some(line) = pipe_emit {
        lines.push(line);
    }

    // --- Phase 2.5b: Signal-based function calls ---
    // Detect `input()`, `output()`, `model()`, and `inject()` function
    // calls in the class body (Angular 17.1+ signal API). High
    // fidelity only (F-ANG-23) — these are the most verbose lines.
    // F-ANG-08: skip the body scan if no matching `}` is found.
    let mut inject_fn_types: Vec<String> = Vec::new();
    if fidelity == Fidelity::High
        && let Some(class_body_start) = find_class_body_open(raw_class)
        && let Some(body_end) =
            crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
    {
        let body = &raw_class[class_body_start..];
        let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
        for sf in collect_signal_fields(body_inner) {
            match sf.kind {
                SignalKind::Input => {
                    let mut line = build_input_line(&sf.name, sf.alias.as_deref());
                    if !line.ends_with(" signal") {
                        line.push_str(" signal");
                    }
                    input_output_lines.push(line);
                }
                SignalKind::Output => {
                    let mut line = build_output_line(&sf.name, sf.alias.as_deref());
                    if !line.ends_with(" signal") {
                        line.push_str(" signal");
                    }
                    input_output_lines.push(line);
                }
                SignalKind::Model => {
                    lines.push(build_model_line(&sf.name, sf.alias.as_deref()));
                }
                SignalKind::Inject => {
                    inject_fn_types.push(sf.name.clone());
                }
            }
        }
    }

    // Emit inject() function calls.
    if !inject_fn_types.is_empty() {
        let mut sorted = inject_fn_types.clone();
        sorted.sort();
        sorted.dedup();
        lines.push(build_injects_line(&sorted));
    }

    lines.extend(input_output_lines);

    // Φinjects: is high-fidelity only (F-ANG-23). Medium already
    // shows the class summary line which carries the type.
    if fidelity == Fidelity::High
        && let Some(types) = extract_constructor_injects(raw_class)
        && !types.is_empty()
    {
        lines.push(build_injects_line(&types));
    }

    if lines.is_empty() {
        None
    } else {
        Some(DecoratorsResult {
            lines,
            inline_template,
        })
    }
}

// --- Phase 2.5b: Signal-based function calls ---

/// Kind of signal-based Angular API function call detected in
/// the class body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalKind {
    Input,
    Output,
    Model,
    Inject,
}

/// A detected signal-function call with its field name and optional alias.
#[derive(Debug, Clone)]
struct SignalField {
    kind: SignalKind,
    name: String,
    alias: Option<String>,
}

/// Scan the class body for signal-based Angular function calls:
/// `input()`, `output()`, `model()`, `inject()`.
///
/// These are `= funcName(...)` assignments at the field level. We scan
/// for the pattern: `fieldName = funcName(...)`.
fn collect_signal_fields(body: &str) -> Vec<SignalField> {
    let mut out: Vec<SignalField> = Vec::new();
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'=' {
            let after_eq = i + 1;
            let mut scan = after_eq;
            while scan < len && (bytes[scan] == b' ' || bytes[scan] == b'\t') {
                scan += 1;
            }
            if scan < len {
                let (func_name, open_paren) =
                    if scan + 5 < len && &bytes[scan..scan + 6] == b"input(" {
                        ("input", scan + 5)
                    } else if scan + 5 < len && &bytes[scan..scan + 6] == b"model(" {
                        ("model", scan + 5)
                    } else if scan + 6 < len && &bytes[scan..scan + 7] == b"output(" {
                        ("output", scan + 6)
                    } else if scan + 5 < len
                        && &bytes[scan..scan + 6] == b"inject"
                        && scan + 6 < len
                        && bytes[scan + 6] == b'('
                    {
                        ("inject", scan + 6)
                    } else {
                        i += 1;
                        continue;
                    };

                let kind = match func_name {
                    "input" => SignalKind::Input,
                    "output" => SignalKind::Output,
                    "model" => SignalKind::Model,
                    "inject" => SignalKind::Inject,
                    // This match is exhaustive — the if/else above only matches these four.
                    // Fallback to `Input` with a debug_assert for development safety.
                    _ => {
                        debug_assert!(false, "Unhandled signal kind: {}", func_name);
                        SignalKind::Input
                    }
                };

                // F-ANG-09: if the call is unterminated, treat as no alias.
                let arg = consume_call_expression(body, open_paren)
                    .map(|(_, arg)| arg)
                    .unwrap_or_default();
                let alias = parse_first_string_arg(&arg);

                // Walk backwards from `=` to find the field name.
                let name_end = if i > 0 { i } else { 0 };
                let mut name_start = name_end;
                while name_start > 0 {
                    if is_word_byte(bytes[name_start - 1]) {
                        name_start -= 1;
                    } else {
                        break;
                    }
                }
                let name = body[name_start..name_end].trim().to_string();
                let name = if name.is_empty() {
                    "?".to_string()
                } else {
                    name
                };

                out.push(SignalField { kind, name, alias });
            }
        }
        i += 1;
    }
    out
}

// F-ANG-12: returns `None` if neither `class ` nor `{` is found (was
// `raw.len()` — silently included the rest of the file as part of
// the "head").
fn find_class_head_end(raw: &str) -> Option<usize> {
    if let Some(pos) = raw.find("class ") {
        return Some(pos);
    }
    if let Some(pos) = raw.find('{') {
        return Some(pos);
    }
    None
}

/// Find the byte offset of the `{` that opens the class body, not
/// any `{` inside a decorator object literal. Scans from the
/// `class` keyword forward, tracking brace depth so that
/// `@Component({...})` braces are skipped.
///
// F-ANG-07: the function still uses `?` rather than `let-else`
// (clippy::question_mark prefers `?` when the enclosing function
// returns `Option` — both produce identical control flow but `?`
// is the canonical idiom). Promoting to `pub(crate)` lets Track D's
// `extract_class_blocks` rewrite use it
// (see `docs/FAANG_AUDIT_ANGULAR_DEFERRED_PLAN.md`). The brace-depth
// + string-literal scan itself delegates to the shared
// `meta_util::find_first_top_level` primitive (Round-8 structural
// audit) — no hand-rolled scanner remains in this file.
/// Find the byte offset of the `{` that opens the class body, not any `{`
/// inside a decorator object literal. Scans from the type declaration keyword
/// forward, tracking brace depth so that `@Component({...})` braces
/// are skipped.
///
/// Supports all type keywords: class, interface, enum, record.
/// The brace-depth + string-literal scan delegates to the shared
/// `meta_util::find_first_top_level` primitive (Round-8 structural
/// audit) — no hand-rolled scanner remains in this file.
pub(crate) fn find_class_body_open(raw: &str) -> Option<usize> {
    const TYPE_KW: &[(&str, usize)] = &[
        ("class ", 6),
        ("interface ", 10),
        ("enum ", 5),
        ("record ", 7),
    ];
    for (kw, kw_len) in TYPE_KW {
        if let Some(pos) = raw.find(kw) {
            return crate::meta_util::find_first_top_level(raw, '{', pos + kw_len);
        }
    }
    None
}

// F-ANG-13: returns `None` when no class name can be found (was
// `"(anonymous)"` literal — callers had to know to substitute `?`
// for unknown names). Callers in `extract_decorators` and
// `extract_graph_entries` now do the `?` substitution at the
// call site, which the audit notes they did anyway.
fn extract_class_name(raw: &str) -> Option<String> {
    if let Some(class_pos) = raw.find("class ") {
        let after = &raw[class_pos + 6..];
        let trimmed = after.trim_start();
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '<' || c == '{' || c == ',')
            .unwrap_or(trimmed.len());
        let name = trimmed[..end].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// Extract input/output field names from a class capture for semantic edge
/// construction.
///
/// Reuses the same field-decorator scanning logic as `extract_decorators`
/// but returns structured data instead of Φ markers. Returns
/// `Vec<(is_input: bool, field_name: String)>`.
pub(crate) fn extract_io_fields(raw_class: &str, fidelity: Fidelity) -> Vec<(bool, String)> {
    let _head_end = match find_class_head_end(raw_class) {
        Some(e) => e,
        None => return Vec::new(),
    };
    let class_body_start = match find_class_body_open(raw_class) {
        Some(s) => s,
        None => return Vec::new(),
    };
    let body_end = match crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
    {
        Some(e) => e,
        None => return Vec::new(),
    };
    let body = &raw_class[class_body_start..];
    let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];

    let mut fields: Vec<(bool, String)> = Vec::new();

    // Collect @Input/@Output field decorators
    for (kind, _alias, field_name) in collect_field_decorators(body_inner) {
        match kind {
            DecoratorKind::Input => fields.push((true, field_name)),
            DecoratorKind::Output => fields.push((false, field_name)),
            _ => {}
        }
    }

    // Collect signal-based fields (input(), output() calls) at High fidelity
    if fidelity == Fidelity::High {
        for sf in collect_signal_fields(body_inner) {
            match sf.kind {
                SignalKind::Input => fields.push((true, sf.name)),
                SignalKind::Output => fields.push((false, sf.name)),
                _ => {}
            }
        }
    }

    fields
}

/// Extract NgModule declarations/imports/exports from a class capture,
/// reusing the same parser as `extract_decorators`. Returns
/// `(declarations, imports, exports)`.
pub(crate) fn extract_module_declarations(
    raw_class: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let head_end = match find_class_head_end(raw_class) {
        Some(e) => e,
        None => return (Vec::new(), Vec::new(), Vec::new()),
    };
    let head = &raw_class[..head_end];
    let decorators = collect_decorators(head);

    for dec in &decorators {
        if dec.kind == DecoratorKind::NgModule {
            return parse_module_fields(&dec.arg);
        }
    }

    (Vec::new(), Vec::new(), Vec::new())
}

/// Extract graph-compatible metadata from the class capture for the
/// cross-file dependency graph (Phase 3, Tier 3).
///
/// Returns `(class_name, kind, selector, injects, pipe_name)` for
/// the class, suitable for feeding into the semantic edge model.
// F-ANG-12: use `?` since the enclosing function returns `Option`.
// F-ANG-13: substitute `?` for missing class names at the call site.
// (Same as `extract_decorators`; the `let-else` → `?` swap is
// required by clippy::question_mark when both apply.)
// Used by `mcp::workspace_util` and `mcp::workspace` when the
// `angular` feature is enabled. `#[allow(dead_code)]` is required
// because it is only reachable from those feature-gated call sites.
#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn extract_graph_entries(
    raw_class: &str,
) -> Option<(
    String,
    ClassKind,
    Option<String>,
    Vec<String>,
    Option<String>,
)> {
    let head_end = find_class_head_end(raw_class)?;
    let head = &raw_class[..head_end];

    let class_name = extract_class_name(raw_class).unwrap_or_else(|| "?".to_string());
    let decorators = collect_decorators(head);

    let mut injects: Vec<String> = Vec::new();
    let mut kind: Option<ClassKind> = None;
    let mut selector: Option<String> = None;
    let mut pipe_name: Option<String> = None;

    for dec in &decorators {
        match dec.kind {
            DecoratorKind::Component => {
                kind = Some(ClassKind::Component);
                let fields = parse_object_literal(&dec.arg);
                selector = fields.selector;
            }
            DecoratorKind::Injectable => {
                kind = Some(ClassKind::Service);
            }
            DecoratorKind::NgModule => {
                kind = Some(ClassKind::Module);
            }
            DecoratorKind::Directive => {
                kind = Some(ClassKind::Directive);
                let fields = parse_object_literal(&dec.arg);
                selector = fields.selector;
            }
            DecoratorKind::Pipe => {
                kind = Some(ClassKind::Pipe);
                let (name, _) = parse_pipe_fields(&dec.arg);
                pipe_name = name;
            }
            _ => {}
        }
    }

    // Extract constructor DI types.
    if let Some(types) = extract_constructor_injects(raw_class) {
        injects = types;
    }

    // Also check for signal-based inject() calls.
    // F-ANG-08: skip the body scan if no matching `}` is found.
    if let Some(class_body_start) = find_class_body_open(raw_class)
        && let Some(body_end) =
            crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
    {
        let body = &raw_class[class_body_start..];
        let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
        for sf in collect_signal_fields(body_inner) {
            if let SignalKind::Inject = sf.kind {
                injects.push(sf.name.clone());
            }
        }
    }

    kind.map(|k| (class_name, k, selector, injects, pipe_name))
}

#[cfg(test)]
#[path = "../tests/angular_meta/decorators.rs"]
mod tests;
