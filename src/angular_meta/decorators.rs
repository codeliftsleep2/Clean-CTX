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

use crate::angular_meta::graph::ClassKind;
use crate::angular_meta::markers::{
    build_component_line, build_directive_line, build_injects_line, build_input_line,
    build_model_line, build_module_line, build_output_line, build_pipe_line, build_service_line,
    ComponentFields,
};
use crate::compression::Fidelity;
use crate::meta_util::{consume_call_expression, split_top_level};

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
        Some(DecoratorsResult { lines, inline_template })
    }
}

/// A collected decorator token. `name` is populated for potential
/// future inspection/debug use but the dispatch only consumes `kind`
/// and `arg` today, so it is kept under `#[allow(dead_code)]`.
#[allow(dead_code)]
struct Decorator {
    name: String,
    arg: String,
    kind: DecoratorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecoratorKind {
    Component,
    Injectable,
    NgModule,
    Directive,
    Pipe,
    Input,
    Output,
    Other,
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
                    if scan + 5 < len && &body[scan..scan + 6] == "input(" {
                        ("input", scan + 5)
                    } else if scan + 5 < len && &body[scan..scan + 6] == "model(" {
                        ("model", scan + 5)
                    } else if scan + 6 < len && &body[scan..scan + 7] == "output(" {
                        ("output", scan + 6)
                    } else if scan + 5 < len
                        && &body[scan..scan + 6] == "inject"
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
                let name = if name.is_empty() { "?".to_string() } else { name };

                out.push(SignalField { kind, name, alias });
            }
        }
        i += 1;
    }
    out
}

fn collect_decorators(head: &str) -> Vec<Decorator> {
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

fn classify_decorator(name: &str) -> DecoratorKind {
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
pub(crate) fn find_class_body_open(raw: &str) -> Option<usize> {
    let class_pos = raw.find("class ")?;
    crate::meta_util::find_first_top_level(raw, '{', class_pos + 6)
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

fn parse_object_literal(arg: &str) -> ComponentFields {
    let mut trimmed = arg.trim().to_string();
    if trimmed.is_empty() {
        return ComponentFields::default();
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed = trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    if trimmed.is_empty() {
        return ComponentFields::default();
    }
    let mut fields = ComponentFields::default();

    for part in split_top_level(&trimmed, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some(colon) = part.find(':') else { continue; };
        let key = part[..colon].trim().trim_matches(|c: char| c == '"' || c == '\'');
        let value = part[colon + 1..].trim();

        match key {
            "selector" => fields.selector = Some(unquote(value).to_string()),
            "templateUrl" => fields.template_url = Some(unquote(value).to_string()),
            "template" => {
                if value.starts_with('`') || value.starts_with('"') || value.starts_with('\'') {
                    fields.template = Some(unquote(value).to_string());
                }
            }
            "styleUrls" => {
                if value.starts_with('[') {
                    let inner = value.trim_start_matches('[').trim_end_matches(']').trim();
                    let urls: Vec<String> = split_top_level(inner, ',')
                        .into_iter()
                        .map(|s| unquote(s.trim()).to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !urls.is_empty() {
                        fields.style_urls = Some(urls);
                    }
                }
            }
            "styles"
                if (value.starts_with('`') || value.starts_with('"') || value.starts_with('\'')) =>
            {
                fields.styles = Some(unquote(value).to_string());
            }
            _ => {}
        }
    }
    fields
}

fn parse_provided_in(arg: &str) -> Option<String> {
    let mut trimmed = arg.trim().to_string();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed = trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    for part in split_top_level(&trimmed, ',') {
        let part = part.trim();
        if let Some(colon) = part.find(':') {
            let key = part[..colon].trim();
            if key == "providedIn" {
                let value = part[colon + 1..].trim();
                if value.starts_with('"') || value.starts_with('\'') {
                    return Some(unquote(value).to_string());
                }
            }
        }
    }
    None
}

fn parse_module_fields(arg: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut trimmed = arg.trim().to_string();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed = trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    let mut decl = Vec::new();
    let mut imp = Vec::new();
    let mut exp = Vec::new();

    for part in split_top_level(&trimmed, ',') {
        let part = part.trim();
        let Some(colon) = part.find(':') else { continue; };
        let key = part[..colon].trim();
        let value = part[colon + 1..].trim();
        match key {
            "declarations" => decl = parse_identifier_list(value),
            "imports" => imp = parse_identifier_list(value),
            "exports" => exp = parse_identifier_list(value),
            _ => {}
        }
    }
    (decl, imp, exp)
}

fn parse_identifier_list(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if !trimmed.starts_with('[') {
        return Vec::new();
    }
    let inner = trimmed.trim_start_matches('[').trim_end_matches(']').trim();
    split_top_level(inner, ',')
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_pipe_fields(arg: &str) -> (Option<String>, bool) {
    let mut trimmed = arg.trim().to_string();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed = trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    let mut name: Option<String> = None;
    for part in split_top_level(&trimmed, ',') {
        let part = part.trim();
        let Some(colon) = part.find(':') else { continue; };
        let key = part[..colon].trim().trim_matches(|c: char| c == '"' || c == '\'');
        let value = part[colon + 1..].trim();
        if key == "name" && (value.starts_with('"') || value.starts_with('\'')) {
            name = Some(unquote(value).to_string());
        }
    }
    (name, false)
}

fn parse_first_string_arg(arg: &str) -> Option<String> {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return None;
    }
    let unquoted = unquote(trimmed);
    if unquoted == trimmed {
        return None;
    }
    Some(unquoted.to_string())
}

fn extract_constructor_injects(raw_class: &str) -> Option<Vec<String>> {
    let body_start = raw_class.find('{')?;
    let body = &raw_class[body_start..];

    let mut search_from = 0;
    let mut ctor_paren: Option<usize> = None;
    while let Some(pos) = body[search_from..].find("constructor") {
        let abs = search_from + pos;
        let before_ok = abs == 0 || !is_word_byte(body.as_bytes()[abs - 1]);
        let after_pos = abs + "constructor".len();
        let after_ok = after_pos >= body.len() || !is_word_byte(body.as_bytes()[after_pos]);
        if before_ok && after_ok {
            let rest = &body[after_pos..];
            let trimmed = rest.trim_start();
            if let Some(after_trim) = trimmed.find('(') {
                let offset = body_start + after_pos + (rest.len() - trimmed.len()) + after_trim;
                ctor_paren = Some(offset);
                break;
            }
        }
        search_from = abs + 1;
    }

    let open = ctor_paren?;
    // F-ANG-09: an unterminated constructor has no params (no
    // injects to extract). Fall back to an empty param list.
    let params = consume_call_expression(raw_class, open)
        .map(|(_, p)| p)
        .unwrap_or_default();

    let mut types: Vec<String> = Vec::new();
    for param in split_top_level(&params, ',') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }
        let has_inject_modifier = param.starts_with("private ")
            || param.starts_with("protected ")
            || param.starts_with("public ")
            || param.starts_with("readonly private ")
            || param.starts_with("readonly protected ")
            || param.starts_with("readonly public ");
        if !has_inject_modifier {
            continue;
        }
        let Some(colon) = param.find(':') else { continue; };
        let type_part = param[colon + 1..].trim();
        let type_part = type_part.split('=').next().unwrap_or(type_part).trim();
        let type_name: String = type_part
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !type_name.is_empty() {
            types.push(type_name);
        }
    }

    if types.is_empty() {
        None
    } else {
        Some(types)
    }
}

fn is_word_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Walk the class body and collect all `@Input(...)` /
/// `@Output(...)` decorator occurrences, pairing each decorator
/// with the field declaration line that follows it.
fn collect_field_decorators(body: &str) -> Vec<(DecoratorKind, Option<String>, String)> {
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

fn unquote(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Extract graph-compatible metadata from the class capture for the
/// cross-file dependency graph (Phase 3, Tier 3).
///
/// Returns `(class_name, kind, selector, injects, pipe_name)` for
/// the class, suitable for pushing into a `GraphCollector`.
// F-ANG-12: use `?` since the enclosing function returns `Option`.
// F-ANG-13: substitute `?` for missing class names at the call site.
// (Same as `extract_decorators`; the `let-else` → `?` swap is
// required by clippy::question_mark when both apply.)
// Used by `mcp::workspace_util` and `mcp::workspace` when the
// `angular` feature is enabled. `#[allow(dead_code)]` is required
// because it is only reachable from those feature-gated call sites.
#[allow(dead_code)]
#[allow(clippy::type_complexity)]
pub fn extract_graph_entries(raw_class: &str) -> Option<(String, ClassKind, Option<String>, Vec<String>, Option<String>)> {
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