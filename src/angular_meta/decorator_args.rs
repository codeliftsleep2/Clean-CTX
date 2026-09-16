// src/angular_meta/decorator_args.rs
//
// Decorator-argument parsing for the Angular meta-layer.
//
// Extracted from `decorators.rs` (active-file size policy) as the cohesive
// unit that turns a decorator's raw argument text (`@Component({...})`,
// `@NgModule({...})`, `@Pipe({...})`, `@Input('alias')`) into structured
// values. The text walkers that *locate* the decorators live in
// `decorator_scan.rs`; Φ marker construction stays in `decorators.rs` /
// `markers.rs`. Behaviour is unchanged from the original implementation.

use crate::angular_meta::markers::ComponentFields;
use crate::meta_util::split_top_level;

pub(crate) fn parse_object_literal(arg: &str) -> ComponentFields {
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
        let Some(colon) = part.find(':') else {
            continue;
        };
        let key = part[..colon]
            .trim()
            .trim_matches(|c: char| c == '"' || c == '\'');
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
                if (value.starts_with('`')
                    || value.starts_with('"')
                    || value.starts_with('\'')) =>
            {
                fields.styles = Some(unquote(value).to_string());
            }
            _ => {}
        }
    }
    fields
}

pub(crate) fn parse_provided_in(arg: &str) -> Option<String> {
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

pub(crate) fn parse_module_fields(arg: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut trimmed = arg.trim().to_string();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed = trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    let mut decl = Vec::new();
    let mut imp = Vec::new();
    let mut exp = Vec::new();

    for part in split_top_level(&trimmed, ',') {
        let part = part.trim();
        let Some(colon) = part.find(':') else {
            continue;
        };
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

pub(crate) fn parse_pipe_fields(arg: &str) -> (Option<String>, bool) {
    let mut trimmed = arg.trim().to_string();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        trimmed = trimmed[1..trimmed.len() - 1].trim().to_string();
    }
    let mut name: Option<String> = None;
    for part in split_top_level(&trimmed, ',') {
        let part = part.trim();
        let Some(colon) = part.find(':') else {
            continue;
        };
        let key = part[..colon]
            .trim()
            .trim_matches(|c: char| c == '"' || c == '\'');
        let value = part[colon + 1..].trim();
        if key == "name" && (value.starts_with('"') || value.starts_with('\'')) {
            name = Some(unquote(value).to_string());
        }
    }
    (name, false)
}

pub(crate) fn parse_first_string_arg(arg: &str) -> Option<String> {
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