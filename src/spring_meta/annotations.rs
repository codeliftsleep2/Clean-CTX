// src/spring_meta/annotations.rs
//
// Spring annotation extraction — Tier 1 of the Meta-Layer.
//
// Given the raw text of a class/interface/record capture, extract every
// Spring annotation and emit the corresponding `Φ` marker lines.
// Also detects field-level `@Autowired` and `@Value` annotations.
//
// # Strategy
//
// We do NOT re-parse the AST. The Java capture pipeline already
// produced a class/interface/record capture that may include the leading
// `@…` annotations. We scan the text **before** the first
// occurrence of the `class` / `interface` / `record` keyword, collect
// every `@…` annotation, classify it, and emit the appropriate `Φ`
// marker lines.
//
// The string walker is O(L) where L is the length of the class
// capture, which is bounded by the class body length.

use crate::compression::Fidelity;
use crate::meta_util::{consume_call_expression, split_top_level};
use crate::spring_meta::markers::{
    RequestMappingMapping, build_autowired_line, build_bean_line, build_configuration_line,
    build_configuration_properties_line, build_controller_line, build_repository_line,
    build_request_mapping_line, build_rest_controller_line, build_service_line, build_value_line,
};

/// Result of [`extract_annotations`]: the Φ marker lines.
pub struct AnnotationsResult {
    /// Φ marker lines (Φrest:, Φsvc:, etc.).
    pub lines: Vec<String>,
}

/// Extract every Spring annotation on the given class capture and
/// emit the corresponding `Φ` marker lines. Returns `None` if no
/// Spring annotation is present.
///
/// `raw_class` is the text of a `class.root` / `interface.root` /
/// `record.root` capture as produced by the Java capture pipeline.
///
/// `fidelity` controls the verbosity of the output:
/// - `Low`    → only class-level summaries (`@RestController`,
///   `@Service`, `@Repository`, `@Controller`, `@Configuration`).
///   No field-level `@Autowired`, no `@RequestMapping` details.
/// - `Medium` → add `@RequestMapping` method mappings; skip
///   field-level `@Autowired`.
/// - `High`   → emit everything including field-level `@Autowired`
///   and `@Value` / `@ConfigurationProperties` markers.
pub fn extract_annotations(raw_class: &str, fidelity: Fidelity) -> Option<AnnotationsResult> {
    let head_end = find_class_head_end(raw_class)?;
    let head = &raw_class[..head_end];

    let class_name = extract_class_name(raw_class).unwrap_or_else(|| "?".to_string());
    let annotations = collect_annotations(head);

    let mut lines: Vec<String> = Vec::new();
    let mut request_mappings: Vec<RequestMappingMapping> = Vec::new();
    let mut autowired_fields: Vec<String> = Vec::new();
    let mut value_fields: Vec<String> = Vec::new();
    let mut bean_methods: Vec<String> = Vec::new();
    let mut config_props_class: Option<String> = None;
    let mut class_kind: Option<AnnotationKind> = None; // Track class-level annotation

    for anno in &annotations {
        match anno.kind {
            AnnotationKind::RestController => {
                request_mappings.extend(parse_request_mappings(&anno.arg));
                class_kind = Some(AnnotationKind::RestController);
            }
            AnnotationKind::Controller => {
                request_mappings.extend(parse_request_mappings(&anno.arg));
                class_kind = Some(AnnotationKind::Controller);
            }
            AnnotationKind::Service => {
                lines.push(build_service_line(&class_name));
            }
            AnnotationKind::Repository => {
                lines.push(build_repository_line(&class_name));
            }
            AnnotationKind::Configuration => {
                lines.push(build_configuration_line(&class_name));
            }
            AnnotationKind::RequestMapping => {
                request_mappings.extend(parse_request_mappings(&anno.arg));
                // Don't set class_kind — @RequestMapping is supplementary to @RestController/@Controller
                if class_kind.is_none() {
                    class_kind = Some(AnnotationKind::RequestMapping);
                }
            }
            AnnotationKind::GetMapping
            | AnnotationKind::PostMapping
            | AnnotationKind::PutMapping
            | AnnotationKind::DeleteMapping
            | AnnotationKind::PatchMapping => {
                let method = annotation_kind_to_http_method(anno.kind);
                let paths = parse_mapping_paths(&anno.arg);
                for path in paths {
                    request_mappings.push(RequestMappingMapping {
                        method: Some(method.clone()),
                        path,
                    });
                }
            }
            AnnotationKind::Autowired => {
                if fidelity == Fidelity::High {
                    // Field-level @Autowired is handled in the body scan below
                }
            }
            AnnotationKind::Value => {
                if fidelity == Fidelity::High {
                    // Field-level @Value is handled in the body scan below
                }
            }
            AnnotationKind::Bean => {
                bean_methods.push(anno.arg.trim().to_string());
            }
            AnnotationKind::ConfigurationProperties => {
                config_props_class = Some(class_name.clone());
            }
            AnnotationKind::Other => {}
        }
    }

    // Method-level markers: scan the class body for @Bean, @GetMapping, etc.
    if fidelity != Fidelity::Low
        && let Some(class_body_start) = find_class_body_open(raw_class)
        && let Some(body_end) =
            crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
    {
        let body = &raw_class[class_body_start..];
        let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
        for (method_name, anno_kind, arg) in collect_method_annotations(body_inner) {
            match anno_kind {
                AnnotationKind::Bean => {
                    bean_methods.push(method_name.clone());
                }
                AnnotationKind::GetMapping
                | AnnotationKind::PostMapping
                | AnnotationKind::PutMapping
                | AnnotationKind::DeleteMapping
                | AnnotationKind::PatchMapping => {
                    let method = annotation_kind_to_http_method(anno_kind);
                    let paths = parse_mapping_paths(&arg);
                    for path in paths {
                        request_mappings.push(RequestMappingMapping {
                            method: Some(method.clone()),
                            path,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    // Emit class-level lines now that all request_mappings are collected
    if let Some(kind) = class_kind {
        match kind {
            AnnotationKind::RestController => {
                lines.push(build_rest_controller_line(&class_name, &request_mappings));
            }
            AnnotationKind::Controller => {
                lines.push(build_controller_line(&class_name, &request_mappings));
            }
            AnnotationKind::RequestMapping => {
                lines.push(build_request_mapping_line(&class_name, &request_mappings));
            }
            _ => {}
        }
    }

    // Emit separate Φmap: line for request mappings
    if !request_mappings.is_empty() {
        // Only emit a Φmap: line if one hasn't already been emitted
        if !lines.iter().any(|l| l.starts_with("Φmap:")) {
            lines.push(build_request_mapping_line(&class_name, &request_mappings));
        }
    }

    // Field-level markers: scan the class body for @Autowired and @Value
    if fidelity == Fidelity::High
        && let Some(class_body_start) = find_class_body_open(raw_class)
        && let Some(body_end) =
            crate::meta_util::find_matching_brace(&raw_class[class_body_start..], '{')
    {
        let body = &raw_class[class_body_start..];
        let body_inner = &body[..=body_end.min(body.len().saturating_sub(1))];
        for (field_name, anno_kind) in collect_field_annotations(body_inner) {
            match anno_kind {
                AnnotationKind::Autowired => {
                    autowired_fields.push(field_name.clone());
                }
                AnnotationKind::Value => {
                    value_fields.push(field_name.clone());
                }
                _ => {}
            }
        }
    }

    // Emit field-level @Autowired markers
    for field in &autowired_fields {
        lines.push(build_autowired_line(field));
    }

    // Emit field-level @Value markers
    for field in &value_fields {
        lines.push(build_value_line(field));
    }

    // Emit @Bean method markers
    for method in &bean_methods {
        lines.push(build_bean_line(method));
    }

    // Emit @ConfigurationProperties marker
    if let Some(class) = config_props_class {
        lines.push(build_configuration_properties_line(&class));
    }

    if lines.is_empty() {
        None
    } else {
        Some(AnnotationsResult { lines })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnnotationKind {
    RestController,
    Controller,
    Service,
    Repository,
    Configuration,
    RequestMapping,
    GetMapping,
    PostMapping,
    PutMapping,
    DeleteMapping,
    PatchMapping,
    Autowired,
    Value,
    Bean,
    ConfigurationProperties,
    Other,
}

fn collect_annotations(head: &str) -> Vec<Annotation> {
    let mut annotations: Vec<Annotation> = Vec::new();
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
            if let Some((consumed, arg_str)) = consume_call_expression(head, i) {
                i += consumed;
                arg = arg_str;
            } else {
                i += 1;
            }
        }
        let kind = classify_annotation(name);
        annotations.push(Annotation {
            name: name.to_string(),
            arg,
            kind,
        });
    }

    annotations
}

#[derive(Debug, Clone)]
struct Annotation {
    #[allow(dead_code)]
    name: String,
    arg: String,
    kind: AnnotationKind,
}

fn classify_annotation(name: &str) -> AnnotationKind {
    match name {
        "RestController" => AnnotationKind::RestController,
        "Controller" => AnnotationKind::Controller,
        "Service" => AnnotationKind::Service,
        "Repository" => AnnotationKind::Repository,
        "Configuration" => AnnotationKind::Configuration,
        "RequestMapping" => AnnotationKind::RequestMapping,
        "GetMapping" => AnnotationKind::GetMapping,
        "PostMapping" => AnnotationKind::PostMapping,
        "PutMapping" => AnnotationKind::PutMapping,
        "DeleteMapping" => AnnotationKind::DeleteMapping,
        "PatchMapping" => AnnotationKind::PatchMapping,
        "Autowired" => AnnotationKind::Autowired,
        "Value" => AnnotationKind::Value,
        "Bean" => AnnotationKind::Bean,
        "ConfigurationProperties" => AnnotationKind::ConfigurationProperties,
        _ => AnnotationKind::Other,
    }
}

fn annotation_kind_to_http_method(kind: AnnotationKind) -> String {
    match kind {
        AnnotationKind::GetMapping => "GET".to_string(),
        AnnotationKind::PostMapping => "POST".to_string(),
        AnnotationKind::PutMapping => "PUT".to_string(),
        AnnotationKind::DeleteMapping => "DELETE".to_string(),
        AnnotationKind::PatchMapping => "PATCH".to_string(),
        _ => "UNKNOWN".to_string(),
    }
}

fn parse_request_mappings(arg: &str) -> Vec<RequestMappingMapping> {
    let mut mappings = Vec::new();
    let trimmed = arg.trim();

    if trimmed.is_empty() {
        return mappings;
    }

    // Parse the object literal: {value = "/path", method = "GET"}
    // or just "/path"
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let mut value = None;
        let mut method = None;

        for part in split_top_level(inner, ',') {
            let part = part.trim();
            if let Some(colon) = part.find(':') {
                let key = part[..colon].trim();
                let val = part[colon + 1..].trim();
                match key {
                    "value" | "path" => {
                        value = Some(unquote(val).to_string());
                    }
                    "method" => {
                        method = Some(unquote(val).to_string());
                    }
                    _ => {}
                }
            }
        }

        if let Some(v) = value {
            mappings.push(RequestMappingMapping { method, path: v });
        }
    } else if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        mappings.push(RequestMappingMapping {
            method: None,
            path: unquote(trimmed).to_string(),
        });
    }

    mappings
}

fn parse_mapping_paths(arg: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let trimmed = arg.trim();

    if trimmed.is_empty() {
        return paths;
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let inner = &trimmed[1..trimmed.len() - 1];
        for part in split_top_level(inner, ',') {
            let part = part.trim();
            if let Some(colon) = part.find(':') {
                let key = part[..colon].trim();
                if key == "value" || key == "path" {
                    let val = part[colon + 1..].trim();
                    paths.push(unquote(val).to_string());
                }
            }
        }
    } else if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        paths.push(unquote(trimmed).to_string());
    }

    paths
}

fn collect_method_annotations(body: &str) -> Vec<(String, AnnotationKind, String)> {
    let mut out: Vec<(String, AnnotationKind, String)> = Vec::new();
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
        let kind = match name {
            "Bean" => Some(AnnotationKind::Bean),
            "GetMapping" => Some(AnnotationKind::GetMapping),
            "PostMapping" => Some(AnnotationKind::PostMapping),
            "PutMapping" => Some(AnnotationKind::PutMapping),
            "DeleteMapping" => Some(AnnotationKind::DeleteMapping),
            "PatchMapping" => Some(AnnotationKind::PatchMapping),
            _ => None,
        };
        if let Some(k) = kind {
            // Extract annotation arguments
            let mut arg = String::new();
            if i < len && bytes[i] == b'(' {
                if let Some((consumed, arg_str)) = consume_call_expression(body, i) {
                    arg = arg_str;
                    i += consumed;
                } else {
                    i += 1;
                }
            }

            // Scan forward to find method parameter list '('
            // Skip everything until we hit '(' or ';' or '{'
            while i < len {
                match bytes[i] {
                    b' ' | b'\t' | b'\n' | b'\r' => {
                        i += 1;
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
                        i += 1;
                    }
                    b'(' => {
                        // Found method params, backtrack to get method name
                        let mut j = i;
                        // Skip whitespace before '('
                        while j > 0 && (bytes[j - 1] == b' ' || bytes[j - 1] == b'\t') {
                            j -= 1;
                        }
                        // Find start of identifier
                        let name_end = j;
                        while j > 0
                            && (bytes[j - 1].is_ascii_alphanumeric()
                                || bytes[j - 1] == b'_'
                                || bytes[j - 1] == b'$')
                        {
                            j -= 1;
                        }
                        let method_name = body[j..name_end].trim().to_string();
                        if !method_name.is_empty() {
                            out.push((method_name, k, arg));
                        }
                        break;
                    }
                    b';' | b'{' => {
                        // End of annotation/method signature without finding method name
                        break;
                    }
                    _ => {
                        // Skip other characters (identifiers, <, >, etc.)
                        i += 1;
                    }
                }
            }
        }
    }

    out
}

fn collect_field_annotations(body: &str) -> Vec<(String, AnnotationKind)> {
    let mut out: Vec<(String, AnnotationKind)> = Vec::new();
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
        let kind = match name {
            "Autowired" => Some(AnnotationKind::Autowired),
            "Value" => Some(AnnotationKind::Value),
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
            if !field_name.is_empty() {
                out.push((field_name, k));
            }
        }
    }

    out
}

// F-ANG-09: `consume_call_expression` now comes from the shared
// layer-agnostic `meta_util` primitive set (Round-8 structural audit).
// It returns `None` if the call expression is unterminated (was
// returning `i-open_paren` and slicing to end of text — silent EOF
// behaviour).

fn find_class_head_end(raw: &str) -> Option<usize> {
    if let Some(pos) = raw.find("class ") {
        return Some(pos);
    }
    if let Some(pos) = raw.find("interface ") {
        return Some(pos);
    }
    if let Some(pos) = raw.find("record ") {
        return Some(pos);
    }
    if let Some(pos) = raw.find('{') {
        return Some(pos);
    }
    None
}

/// Find the byte offset of the `{` that opens the class body, not any `{`
/// inside an annotation object literal. Scans from the type declaration keyword
/// forward, tracking brace depth so that `@RequestMapping({...})` braces
/// are skipped.
///
/// Supports all type keywords: class, interface, enum, record, struct.
/// The brace-depth + string-literal scan delegates to the shared
/// `meta_util::find_first_top_level` primitive (Round-8 structural audit)
/// — no hand-rolled scanner remains in this file.
fn find_class_body_open(raw: &str) -> Option<usize> {
    const TYPE_KW: &[(&str, usize)] = &[
        ("class ", 6),
        ("interface ", 10),
        ("enum ", 5),
        ("record ", 7),
        ("struct ", 7),
    ];
    for (kw, kw_len) in TYPE_KW {
        if let Some(pos) = raw.find(kw) {
            return crate::meta_util::find_first_top_level(raw, '{', pos + kw_len);
        }
    }
    None
}

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
    if let Some(iface_pos) = raw.find("interface ") {
        let after = &raw[iface_pos + 10..];
        let trimmed = after.trim_start();
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '<' || c == '{' || c == ',')
            .unwrap_or(trimmed.len());
        let name = trimmed[..end].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    if let Some(record_pos) = raw.find("record ") {
        let after = &raw[record_pos + 7..];
        let trimmed = after.trim_start();
        let end = trimmed
            .find(|c: char| c.is_whitespace() || c == '(' || c == '{' || c == ',')
            .unwrap_or(trimmed.len());
        let name = trimmed[..end].trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
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
