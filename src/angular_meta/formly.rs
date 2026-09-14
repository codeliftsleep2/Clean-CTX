//! ngx-formly source-shape detector.
//!
//! This layer compresses statically declared Formly field arrays without
//! evaluating TypeScript or reproducing user-facing configuration text.

use crate::angular_meta::phi::PhiMarker;
use crate::angular_meta::util::{
    find_matching_brace, is_inside_comment_or_string, split_top_level,
};
use crate::compression::Fidelity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormlyKind {
    Field,
    Group,
    Expression,
}

impl PhiMarker for FormlyKind {
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Field => "Φffield:",
            Self::Group => "Φfgroup:",
            Self::Expression => "Φfexpr:",
        }
    }

    fn expansion(self) -> &'static str {
        match self {
            Self::Field => "Formly field",
            Self::Group => "Formly field group",
            Self::Expression => "Formly expression",
        }
    }

    fn all_in_expand_order() -> &'static [Self] {
        &[Self::Expression, Self::Group, Self::Field]
    }

    fn from_token(token: &str) -> Option<Self> {
        match token {
            "Φffield" => Some(Self::Field),
            "Φfgroup" => Some(Self::Group),
            "Φfexpr" => Some(Self::Expression),
            _ => None,
        }
    }

    fn token(self) -> &'static str {
        match self {
            Self::Field => "Φffield",
            Self::Group => "Φfgroup",
            Self::Expression => "Φfexpr",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct FormlyField {
    key: Option<String>,
    field_type: Option<String>,
    children: Vec<FormlyField>,
    expressions: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormlyShape {
    fields: Vec<FormlyField>,
}

impl FormlyShape {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn render(&self, fidelity: Fidelity) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        if fidelity == Fidelity::Low {
            lines.push(format!(
                "{}count={}",
                FormlyKind::Field.marker_prefix(),
                self.fields.len()
            ));
        } else {
            lines.extend(self.fields.iter().filter_map(render_top_level_field));
            if is_high_detail(fidelity) {
                collect_groups(&self.fields, &mut lines);
                collect_expressions(&self.fields, &mut lines);
            }
        }

        if lines.is_empty() {
            return String::new();
        }
        format!("// --- Φ Formly Meta ---\n{}\n", lines.join("\n"))
    }
}

fn render_top_level_field(field: &FormlyField) -> Option<String> {
    let key = field.key.as_deref()?;
    let mut line = format!("{}{key}", FormlyKind::Field.marker_prefix());
    if let Some(field_type) = &field.field_type {
        line.push_str(" type=");
        line.push_str(field_type);
    }
    Some(line)
}

fn collect_groups(fields: &[FormlyField], lines: &mut Vec<String>) {
    for field in fields {
        if !field.children.is_empty() {
            if let Some(key) = &field.key {
                let children = field
                    .children
                    .iter()
                    .filter_map(|child| child.key.as_deref())
                    .collect::<Vec<_>>()
                    .join(",");
                lines.push(format!(
                    "{}{key} fields=[{children}]",
                    FormlyKind::Group.marker_prefix()
                ));
            }
            collect_groups(&field.children, lines);
        }
    }
}

fn collect_expressions(fields: &[FormlyField], lines: &mut Vec<String>) {
    for field in fields {
        if let Some(key) = &field.key {
            lines.extend(field.expressions.iter().map(|expression| {
                format!(
                    "{}{key} → {expression}",
                    FormlyKind::Expression.marker_prefix()
                )
            }));
        }
        collect_expressions(&field.children, lines);
    }
}

fn is_high_detail(fidelity: Fidelity) -> bool {
    matches!(
        fidelity,
        Fidelity::High | Fidelity::Edit | Fidelity::Verbatim
    )
}

pub fn has_formly(source: &str) -> bool {
    extract_formly_shape(source, Fidelity::Low).is_some()
}

pub fn extract_formly_shape(source: &str, _fidelity: Fidelity) -> Option<FormlyShape> {
    let import_gated = has_formly_import(source);
    if !import_gated && !source.contains("FormlyFieldConfig") {
        return None;
    }

    let mut fields = Vec::new();
    let mut consumed_until = 0usize;
    for (equals, _) in source.match_indices('=') {
        if equals < consumed_until || is_inside_comment_or_string(source, equals) {
            continue;
        }
        let value_start = skip_whitespace(source, equals + 1);
        if source.as_bytes().get(value_start) != Some(&b'[') {
            continue;
        }
        let Some(close_relative) = find_matching_brace(&source[value_start..], '[') else {
            continue;
        };
        let close = value_start + close_relative;
        let lhs = assignment_lhs(source, equals);
        let typed = lhs.contains("FormlyFieldConfig") && lhs.contains('[') && lhs.contains(']');
        let plain_fields = import_gated && assignment_name(lhs) == Some("fields");
        if !typed && !plain_fields {
            continue;
        }

        fields.extend(parse_field_array(&source[value_start + 1..close]));
        consumed_until = close + 1;
    }

    (!fields.is_empty()).then_some(FormlyShape { fields })
}

fn has_formly_import(source: &str) -> bool {
    for (start, _) in source.match_indices("import") {
        if is_inside_comment_or_string(source, start) || !word_boundary(source, start, "import") {
            continue;
        }
        let end = source[start..]
            .find(';')
            .map_or(source.len(), |relative| start + relative + 1);
        let declaration = &source[start..end];
        if declaration.contains("'@ngx-formly/core'")
            || declaration.contains("\"@ngx-formly/core\"")
        {
            return true;
        }
    }
    false
}

fn word_boundary(source: &str, start: usize, word: &str) -> bool {
    let before = start == 0 || !is_identifier_byte(source.as_bytes()[start - 1]);
    let after = start + word.len();
    before && (after == source.len() || !is_identifier_byte(source.as_bytes()[after]))
}

fn assignment_lhs(source: &str, equals: usize) -> &str {
    let start = source[..equals]
        .rfind([';', '\n', '{', '}'])
        .map_or(0, |index| index + 1);
    source[start..equals].trim()
}

fn assignment_name(lhs: &str) -> Option<&str> {
    let tail = lhs.split_whitespace().last()?;
    let name = tail.rsplit('.').next()?;
    is_identifier(name).then_some(name)
}

fn parse_field_array(array: &str) -> Vec<FormlyField> {
    split_top_level(array, ',')
        .into_iter()
        .filter_map(|entry| parse_field_object(entry.trim()))
        .collect()
}

fn parse_field_object(object: &str) -> Option<FormlyField> {
    if !object.starts_with('{') {
        return None;
    }
    let close = find_matching_brace(object, '{')?;
    if !object[close + 1..].trim().is_empty() {
        return None;
    }

    let mut field = FormlyField::default();
    for property in split_top_level(&object[1..close], ',') {
        let parts = split_top_level(&property, ':');
        if parts.len() < 2 {
            continue;
        }
        let Some(name) = static_property_name(&parts[0]) else {
            continue;
        };
        let value = parts[1..].join(":");
        match name.as_str() {
            "key" => field.key = static_literal(&value),
            "type" => field.field_type = static_literal(&value),
            "fieldGroup" => field.children = parse_array_value(&value),
            "hideExpression" => field.expressions.push("hideExpression".to_string()),
            "expressionProperties" => {
                field.expressions.extend(expression_targets(&value));
            }
            _ => {}
        }
    }

    // A dynamic or absent key has no stable identity. Omitting that entry
    // keeps Low's count aligned with the population Medium can expose.
    field.key.is_some().then_some(field)
}

fn parse_array_value(value: &str) -> Vec<FormlyField> {
    let value = value.trim();
    if !value.starts_with('[') {
        return Vec::new();
    }
    let Some(close) = find_matching_brace(value, '[') else {
        return Vec::new();
    };
    parse_field_array(&value[1..close])
}

fn expression_targets(value: &str) -> Vec<String> {
    let value = value.trim();
    if !value.starts_with('{') {
        return Vec::new();
    }
    let Some(close) = find_matching_brace(value, '{') else {
        return Vec::new();
    };
    split_top_level(&value[1..close], ',')
        .into_iter()
        .filter_map(|property| {
            let parts = split_top_level(&property, ':');
            let target = static_property_name(parts.first()?)?;
            matches!(
                target.as_str(),
                "props.disabled" | "props.required" | "className"
            )
            .then_some(target)
        })
        .collect()
}

fn static_property_name(raw: &str) -> Option<String> {
    static_literal(raw).or_else(|| {
        let value = raw.trim();
        is_identifier(value).then(|| value.to_string())
    })
}

fn static_literal(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.len() < 2 {
        return None;
    }
    let delimiter = value.as_bytes()[0];
    if !matches!(delimiter, b'\'' | b'"' | b'`') || value.as_bytes().last() != Some(&delimiter) {
        return None;
    }
    let mut escaped = false;
    for (index, byte) in value.as_bytes()[1..].iter().enumerate() {
        if escaped {
            escaped = false;
        } else if *byte == b'\\' {
            escaped = true;
        } else if *byte == delimiter && index + 2 != value.len() {
            return None;
        }
    }
    let inner = &value[1..value.len() - 1];
    if inner.is_empty() || delimiter == b'`' && inner.contains("${") {
        return None;
    }
    Some(inner.to_string())
}

fn skip_whitespace(source: &str, mut index: usize) -> usize {
    while source
        .as_bytes()
        .get(index)
        .is_some_and(u8::is_ascii_whitespace)
    {
        index += 1;
    }
    index
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<FormlyKind>(line)
}

pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<FormlyKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/formly.rs"]
mod tests;
