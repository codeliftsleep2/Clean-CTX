//! Field-level Spring annotation parsing.

use super::AnnotationKind;
use crate::meta_util::consume_call_expression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnnotatedField {
    pub(crate) name: String,
    pub(crate) type_name: String,
    pub(crate) kind: AnnotationKind,
}

pub(crate) fn collect_field_annotations(body: &str) -> Vec<AnnotatedField> {
    let mut fields = Vec::new();
    let bytes = body.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] != b'@' {
            cursor += 1;
            continue;
        }
        cursor += 1;
        let name_start = cursor;
        while cursor < bytes.len()
            && (bytes[cursor].is_ascii_alphanumeric() || matches!(bytes[cursor], b'_' | b'$'))
        {
            cursor += 1;
        }
        let kind = match &body[name_start..cursor] {
            "Autowired" => AnnotationKind::Autowired,
            "Value" => AnnotationKind::Value,
            _ => continue,
        };

        cursor = skip_whitespace(bytes, cursor);
        if cursor < bytes.len() && bytes[cursor] == b'(' {
            if let Some((consumed, _)) = consume_call_expression(body, cursor) {
                cursor += consumed;
            } else {
                continue;
            }
        }
        cursor = skip_whitespace(bytes, cursor);

        let declaration_start = cursor;
        while cursor < bytes.len() && !matches!(bytes[cursor], b';' | b'=' | b'\n' | b'\r') {
            cursor += 1;
        }
        let declaration = body[declaration_start..cursor].trim();
        let tokens: Vec<&str> = declaration.split_whitespace().collect();
        if tokens.len() < 2 {
            continue;
        }
        let name = tokens[tokens.len() - 1].trim_end_matches("[]");
        let type_name = tokens[tokens.len() - 2];
        if !name.is_empty() && !type_name.is_empty() {
            fields.push(AnnotatedField {
                name: name.to_string(),
                type_name: type_name.to_string(),
                kind,
            });
        }
    }

    fields
}

fn skip_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }
    cursor
}
