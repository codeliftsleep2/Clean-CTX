//! Angular Reactive Forms source-shape detector.
//!
//! The layer is deliberately string-based and independent from template
//! extraction. It activates only when an `@angular/forms` import and a
//! statically recognizable reactive-form construction coexist.

use crate::angular_meta::phi::PhiMarker;
use crate::angular_meta::util::{
    consume_call_expression, extract_decl_name, find_matching_brace, is_inside_comment_or_string,
    split_top_level,
};
use crate::compression::Fidelity;

#[path = "reactive_forms_validators.rs"]
mod validators;
use validators::{validators_from_control_body, validators_from_value};

#[path = "reactive_forms_imports.rs"]
mod imports;
use imports::{
    ImportedForms, extract_builder_aliases, extract_forms_imports, has_construction_hint,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReactiveFormKind {
    Form,
    Control,
    Array,
    Validator,
}

impl PhiMarker for ReactiveFormKind {
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Form => "Φform:",
            Self::Control => "Φcontrol:",
            Self::Array => "Φarray:",
            Self::Validator => "Φvalidator:",
        }
    }

    fn expansion(self) -> &'static str {
        match self {
            Self::Form => "FormGroup",
            Self::Control => "FormControl",
            Self::Array => "FormArray",
            Self::Validator => "Validator",
        }
    }

    fn all_in_expand_order() -> &'static [Self] {
        &[Self::Validator, Self::Control, Self::Array, Self::Form]
    }

    fn from_token(token: &str) -> Option<Self> {
        match token {
            "Φform" => Some(Self::Form),
            "Φcontrol" => Some(Self::Control),
            "Φarray" => Some(Self::Array),
            "Φvalidator" => Some(Self::Validator),
            _ => None,
        }
    }

    fn token(self) -> &'static str {
        match self {
            Self::Form => "Φform",
            Self::Control => "Φcontrol",
            Self::Array => "Φarray",
            Self::Validator => "Φvalidator",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    Control,
    Array,
}

#[derive(Debug, Clone)]
struct FieldDecl {
    name: String,
    kind: FieldKind,
    validators: Vec<String>,
    nested_controls: Vec<String>,
}

#[derive(Debug, Clone)]
struct FormDecl {
    name: String,
    fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone)]
enum FormArtifact {
    Form(FormDecl),
    Field(FieldDecl),
}

#[derive(Debug, Clone, Default)]
pub struct ReactiveFormShape {
    artifacts: Vec<FormArtifact>,
}

impl ReactiveFormShape {
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    pub fn render(&self, fidelity: Fidelity) -> String {
        let mut lines = Vec::new();
        for artifact in &self.artifacts {
            match artifact {
                FormArtifact::Form(form) => render_form(form, fidelity, &mut lines),
                FormArtifact::Field(field) if fidelity != Fidelity::Low => {
                    render_field(field, fidelity, &mut lines);
                }
                FormArtifact::Field(_) => {}
            }
        }
        if lines.is_empty() {
            return String::new();
        }
        format!("// --- Φ Forms Meta ---\n{}\n", lines.join("\n"))
    }
}

fn render_form(form: &FormDecl, fidelity: Fidelity, lines: &mut Vec<String>) {
    if fidelity == Fidelity::Low {
        lines.push(format!("  Φform:{}", form.name));
        return;
    }
    let controls = form
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>()
        .join(",");
    if controls.is_empty() {
        lines.push(format!("  Φform:{}", form.name));
    } else {
        lines.push(format!("  Φform:{} ctrls=[{}]", form.name, controls));
    }
    for field in &form.fields {
        render_field(field, fidelity, lines);
    }
}

fn render_field(field: &FieldDecl, fidelity: Fidelity, lines: &mut Vec<String>) {
    match field.kind {
        FieldKind::Control => lines.push(format!("  Φcontrol:{}", field.name)),
        FieldKind::Array if is_high_detail(fidelity) && !field.nested_controls.is_empty() => {
            lines.push(format!(
                "  Φarray:{} ctrls=[{}]",
                field.name,
                field.nested_controls.join(",")
            ));
        }
        FieldKind::Array => lines.push(format!("  Φarray:{}", field.name)),
    }
    if is_high_detail(fidelity) && !field.validators.is_empty() {
        lines.push(format!(
            "  Φvalidator:{} → [{}]",
            field.name,
            field.validators.join(",")
        ));
    }
}

fn is_high_detail(fidelity: Fidelity) -> bool {
    matches!(
        fidelity,
        Fidelity::High | Fidelity::Edit | Fidelity::Verbatim
    )
}

pub fn has_reactive_forms(source: &str) -> bool {
    extract_reactive_form_shape(source, Fidelity::Low).is_some()
}

pub fn extract_reactive_form_shape(source: &str, _fidelity: Fidelity) -> Option<ReactiveFormShape> {
    let imports = extract_forms_imports(source);
    if imports.is_empty() || !has_construction_hint(source, &imports) {
        return None;
    }
    let builder_aliases = extract_builder_aliases(source, &imports.builders);
    let candidates = find_candidates(source, &imports, &builder_aliases);
    let mut artifacts = Vec::new();
    let mut consumed_until = 0usize;

    for candidate in candidates {
        if candidate.start < consumed_until {
            continue;
        }
        let Some((consumed, body)) = consume_call_expression(source, candidate.open_paren) else {
            continue;
        };
        let end = candidate.open_paren + consumed;
        let Some(name) = extract_decl_name(&source[..candidate.start]) else {
            continue;
        };
        match candidate.kind {
            ConstructionKind::Group => {
                artifacts.push(FormArtifact::Form(FormDecl {
                    name,
                    fields: extract_group_fields(&body, &imports, &builder_aliases),
                }));
            }
            ConstructionKind::Control => artifacts.push(FormArtifact::Field(FieldDecl {
                name,
                kind: FieldKind::Control,
                validators: validators_from_control_body(&body),
                nested_controls: Vec::new(),
            })),
            ConstructionKind::Array => artifacts.push(FormArtifact::Field(FieldDecl {
                name,
                kind: FieldKind::Array,
                validators: Vec::new(),
                nested_controls: nested_array_controls(&body, &imports, &builder_aliases),
            })),
        }
        consumed_until = end;
    }

    (!artifacts.is_empty()).then_some(ReactiveFormShape { artifacts })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstructionKind {
    Group,
    Control,
    Array,
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    start: usize,
    open_paren: usize,
    kind: ConstructionKind,
}

fn find_candidates(
    source: &str,
    imports: &ImportedForms,
    builder_aliases: &[String],
) -> Vec<Candidate> {
    let mut candidates = Vec::new();
    collect_explicit_candidates(
        source,
        &imports.groups,
        ConstructionKind::Group,
        &mut candidates,
    );
    collect_explicit_candidates(
        source,
        &imports.controls,
        ConstructionKind::Control,
        &mut candidates,
    );
    collect_explicit_candidates(
        source,
        &imports.arrays,
        ConstructionKind::Array,
        &mut candidates,
    );
    collect_builder_candidates(
        source,
        builder_aliases,
        "group",
        ConstructionKind::Group,
        &mut candidates,
    );
    collect_builder_candidates(
        source,
        builder_aliases,
        "control",
        ConstructionKind::Control,
        &mut candidates,
    );
    collect_builder_candidates(
        source,
        builder_aliases,
        "array",
        ConstructionKind::Array,
        &mut candidates,
    );
    candidates.sort_by_key(|candidate| (candidate.start, candidate.open_paren));
    candidates
}

fn collect_explicit_candidates(
    source: &str,
    names: &[String],
    kind: ConstructionKind,
    candidates: &mut Vec<Candidate>,
) {
    for name in names {
        let pattern = format!("new {name}");
        for (start, _) in source.match_indices(&pattern) {
            if is_inside_comment_or_string(source, start) || !has_word_boundary(source, start) {
                continue;
            }
            let after_name = start + pattern.len();
            let whitespace = source[after_name..].len() - source[after_name..].trim_start().len();
            let open = after_name + whitespace;
            if source.as_bytes().get(open) == Some(&b'(') {
                candidates.push(Candidate {
                    start,
                    open_paren: open,
                    kind,
                });
            }
        }
    }
}

fn collect_builder_candidates(
    source: &str,
    aliases: &[String],
    method: &str,
    kind: ConstructionKind,
    candidates: &mut Vec<Candidate>,
) {
    let pattern = format!(".{method}");
    for (dot, _) in source.match_indices(&pattern) {
        if is_inside_comment_or_string(source, dot) {
            continue;
        }
        let after_method = dot + pattern.len();
        let whitespace = source[after_method..].len() - source[after_method..].trim_start().len();
        let open = after_method + whitespace;
        if source.as_bytes().get(open) != Some(&b'(') {
            continue;
        }
        let Some((receiver_start, receiver)) = receiver_before(source, dot) else {
            continue;
        };
        if !aliases.iter().any(|alias| alias == receiver) {
            continue;
        }
        candidates.push(Candidate {
            start: expression_start(source, receiver_start),
            open_paren: open,
            kind,
        });
    }
}

fn receiver_before(source: &str, dot: usize) -> Option<(usize, &str)> {
    let bytes = source.as_bytes();
    let mut end = dot;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    (start < end).then_some((start, &source[start..end]))
}

fn expression_start(source: &str, receiver_start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut start = receiver_start;
    while start > 0 && bytes[start - 1] == b'.' {
        let mut preceding = start - 1;
        while preceding > 0 && is_identifier_byte(bytes[preceding - 1]) {
            preceding -= 1;
        }
        if preceding == start - 1 {
            break;
        }
        start = preceding;
    }
    start
}

fn has_word_boundary(source: &str, start: usize) -> bool {
    start == 0 || !is_identifier_byte(source.as_bytes()[start - 1])
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first.is_alphabetic() || matches!(first, '_' | '$'))
        && chars.all(|ch| ch.is_alphanumeric() || matches!(ch, '_' | '$'))
}

fn extract_group_fields(
    body: &str,
    imports: &ImportedForms,
    builder_aliases: &[String],
) -> Vec<FieldDecl> {
    let Some(object) = first_object_argument(body) else {
        return Vec::new();
    };
    let mut fields = Vec::new();
    for entry in split_top_level(&object, ',') {
        let parts = split_top_level(&entry, ':');
        if parts.len() < 2 {
            continue;
        }
        let Some(name) = static_property_name(&parts[0]) else {
            continue;
        };
        let value = parts[1..].join(":");
        let construction = find_candidates(&value, imports, builder_aliases)
            .first()
            .map(|candidate| candidate.kind);
        let kind = if construction == Some(ConstructionKind::Array) {
            FieldKind::Array
        } else {
            FieldKind::Control
        };
        let validators = if kind == FieldKind::Control {
            validators_from_value(&value, imports, builder_aliases)
        } else {
            Vec::new()
        };
        let nested_controls = if kind == FieldKind::Array {
            nested_array_controls(&value, imports, builder_aliases)
        } else {
            Vec::new()
        };
        fields.push(FieldDecl {
            name,
            kind,
            validators,
            nested_controls,
        });
    }
    fields
}

fn first_object_argument(body: &str) -> Option<String> {
    let first = split_top_level(body, ',').into_iter().next()?;
    let trimmed = first.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    let close = find_matching_brace(trimmed, '{')?;
    (close == trimmed.len() - 1).then(|| trimmed[1..close].to_string())
}

fn static_property_name(raw: &str) -> Option<String> {
    let key = raw.trim();
    if key.len() >= 2
        && ((key.starts_with('\'') && key.ends_with('\''))
            || (key.starts_with('"') && key.ends_with('"')))
    {
        let unquoted = &key[1..key.len() - 1];
        return is_identifier(unquoted).then(|| unquoted.to_string());
    }
    is_identifier(key).then(|| key.to_string())
}

fn nested_array_controls(
    body: &str,
    imports: &ImportedForms,
    builder_aliases: &[String],
) -> Vec<String> {
    let candidates = find_candidates(body, imports, builder_aliases);
    let Some(group) = candidates
        .iter()
        .find(|candidate| candidate.kind == ConstructionKind::Group)
    else {
        return Vec::new();
    };
    let Some((_, group_body)) = consume_call_expression(body, group.open_paren) else {
        return Vec::new();
    };
    extract_group_fields(&group_body, imports, builder_aliases)
        .into_iter()
        .map(|field| field.name)
        .collect()
}

pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<ReactiveFormKind>(line)
}

pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<ReactiveFormKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/reactive_forms.rs"]
mod tests;
