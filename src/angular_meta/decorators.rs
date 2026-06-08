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

use crate::angular_meta::markers::{
    build_component_line, build_directive_line, build_injects_line, build_input_line,
    build_model_line, build_module_line, build_output_line, build_pipe_line, build_service_line,
    ComponentFields,
};

/// Extract every Angular decorator on the given class capture and
/// emit the corresponding `Φ` marker lines. Returns `None` if no
/// Angular decorator or signal-based API is present.
///
/// `raw_class` is the text of a `class_declaration` capture as
/// produced by the TS capture pipeline.
pub fn extract_decorators(raw_class: &str) -> Option<Vec<String>> {
    let head_end = find_class_head_end(raw_class);
    let head = &raw_class[..head_end];

    let class_name = extract_class_name(raw_class);
    let decorators = collect_decorators(head);

    let mut lines: Vec<String> = Vec::new();
    let mut input_output_lines: Vec<String> = Vec::new();
    let mut component_emit: Option<String> = None;
    let mut service_emit: Option<String> = None;
    let mut module_emit: Option<String> = None;
    let mut directive_emit: Option<String> = None;
    let mut pipe_emit: Option<String> = None;

    for dec in &decorators {
        match dec.kind {
            DecoratorKind::Component => {
                let fields = parse_object_literal(&dec.arg);
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
                let alias = parse_first_string_arg(&dec.arg);
                input_output_lines.push(build_input_line("?", alias.as_deref()));
            }
            DecoratorKind::Output => {
                let alias = parse_first_string_arg(&dec.arg);
                input_output_lines.push(build_output_line("?", alias.as_deref()));
            }
            DecoratorKind::Other => {}
        }
    }

    // Field-level markers: scan the class body for
    // `@Input(...)` / `@Output(...)` decorators attached to
    // individual field declarations.
    if let Some(class_body_start) = find_class_body_open(raw_class) {
        let body = &raw_class[class_body_start..];
        let body_end = find_matching_brace(body, 0);
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
    // calls in the class body (Angular 17.1+ signal API).
    let mut inject_fn_types: Vec<String> = Vec::new();
    if let Some(class_body_start) = find_class_body_open(raw_class) {
        let body = &raw_class[class_body_start..];
        let body_end = find_matching_brace(body, 0);
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

    if let Some(types) = extract_constructor_injects(raw_class)
        && !types.is_empty()
    {
        lines.push(build_injects_line(&types));
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

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
                    _ => unreachable!(),
                };

                let (_, arg) = consume_call_expression(body, open_paren);
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
        if i < len && bytes[i] == b'(' {
            let (consumed, arg_str) = consume_call_expression(head, i);
            i += consumed;
            arg = arg_str;
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

fn consume_call_expression(text: &str, open_paren: usize) -> (usize, String) {
    let bytes = text.as_bytes();
    let mut depth: i32 = 0;
    let mut i = open_paren;
    let len = bytes.len();
    while i < len {
        let c = bytes[i];
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let arg = text[open_paren + 1..i].to_string();
                    return (i - open_paren + 1, arg);
                }
            }
            b'"' | b'\'' => {
                let quote = c;
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            b'`' => {
                i += 1;
                while i < len && bytes[i] != b'`' {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    (i.saturating_sub(open_paren), text[open_paren + 1..i].to_string())
}

/// Find the byte offset of the `}` that matches the `{` at
/// `open_brace` in `text`, tracking nested braces, strings, and
/// template literals. Returns `text.len() - 1` if no match is
/// found (so the caller can safely slice up to the end).
fn find_matching_brace(text: &str, open_brace: usize) -> usize {
    let bytes = text.as_bytes();
    let mut depth: i32 = 0;
    let mut i = open_brace;
    let len = bytes.len();
    while i < len {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            b'`' => {
                i += 1;
                while i < len && bytes[i] != b'`' {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    len.saturating_sub(1)
}

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

fn find_class_head_end(raw: &str) -> usize {
    if let Some(pos) = raw.find("class ") {
        return pos;
    }
    if let Some(pos) = raw.find('{') {
        return pos;
    }
    raw.len()
}

/// Find the byte offset of the `{` that opens the class body, not
/// any `{` inside a decorator object literal. Scans from the
/// `class` keyword forward, tracking brace depth so that
/// `@Component({...})` braces are skipped.
fn find_class_body_open(raw: &str) -> Option<usize> {
    let class_pos = raw.find("class ")?;
    let search_start = class_pos + 6;
    let bytes = raw.as_bytes();
    let len = bytes.len();
    let mut depth: i32 = 0;
    let mut i = search_start;

    while i < len {
        match bytes[i] {
            b'{' => {
                if depth == 0 {
                    return Some(i);
                }
                depth += 1;
            }
            b'}' => {
                depth -= 1;
            }
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            b'`' => {
                i += 1;
                while i < len && bytes[i] != b'`' {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn extract_class_name(raw: &str) -> String {
    if let Some(class_pos) = raw.find("class ") {
        let after = &raw[class_pos + 6..];
        let trimmed = after.trim_start();
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '<' || c == '{' || c == ',')
            .unwrap_or(trimmed.len());
        let name = trimmed[..end].trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }
    "(anonymous)".to_string()
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

    for part in split_top_level_commas(&trimmed) {
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
                    let urls: Vec<String> = split_top_level_commas(inner)
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
    for part in split_top_level_commas(&trimmed) {
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

    for part in split_top_level_commas(&trimmed) {
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
    split_top_level_commas(inner)
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
    for part in split_top_level_commas(&trimmed) {
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
    let (_, params) = consume_call_expression(raw_class, open);

    let mut types: Vec<String> = Vec::new();
    for param in split_top_level_commas(&params) {
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
        if i < len && bytes[i] == b'(' {
            let (consumed, arg_str) = consume_call_expression(body, i);
            i += consumed;
            arg = arg_str;
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

fn split_top_level_commas(s: &str) -> Vec<String> {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut parts: Vec<String> = Vec::new();
    let mut start = 0;
    let mut depth_brace: i32 = 0;
    let mut depth_bracket: i32 = 0;
    let mut depth_paren: i32 = 0;
    let mut i = 0;

    while i < len {
        let c = bytes[i];
        match c {
            b',' if depth_brace == 0 && depth_bracket == 0 && depth_paren == 0 => {
                parts.push(s[start..i].to_string());
                start = i + 1;
            }
            b'{' => depth_brace += 1,
            b'}' => depth_brace -= 1,
            b'[' => depth_bracket += 1,
            b']' => depth_bracket -= 1,
            b'(' => depth_paren += 1,
            b')' => depth_paren -= 1,
            b'"' | b'\'' => {
                let quote = c;
                i += 1;
                while i < len && bytes[i] != quote {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            b'`' => {
                i += 1;
                while i < len && bytes[i] != b'`' {
                    if bytes[i] == b'\\' && i + 1 < len {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    if start < len {
        parts.push(s[start..].to_string());
    } else if start == len && !parts.is_empty() {
        // trailing empty
    } else if start == len {
        parts.push(String::new());
    }
    parts
}

#[cfg(test)]
#[path = "../tests/angular_meta/decorators.rs"]
mod tests;