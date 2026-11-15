use super::model::{Declaration, Diagnostic, RELATIONS, Relationship};
use std::collections::BTreeMap;

/// Locate a field only inside its exact source-owned declaration fence. Fences
/// nested in illustrative examples are inert, matching declaration parsing.
pub(super) fn field_location(bytes: &[u8], id: &str, field: &str) -> Option<(usize, usize)> {
    let text = std::str::from_utf8(bytes).ok()?;
    let mut fence: Option<(char, usize, bool)> = None;
    let mut selected_id = None;
    let mut location = None;
    for (offset, line) in text.lines().enumerate() {
        let marker = line.chars().next().filter(|c| *c == '`' || *c == '~');
        let count = marker.map_or(0, |c| line.chars().take_while(|x| *x == c).count());
        if let Some((ch, length, selected)) = fence {
            if marker == Some(ch) && count >= length && line[count..].trim().is_empty() {
                if selected && selected_id == Some(id) {
                    return location;
                }
                fence = None;
                selected_id = None;
                location = None;
            } else if selected && let Some((key, value)) = line.split_once(':') {
                if key == "node" {
                    selected_id = Some(value.trim());
                }
                if key == field {
                    location = Some((offset + 1, 1));
                }
            }
        } else if count >= 3 {
            fence = Some((marker?, count, line[count..].trim() == "architecture-graph"));
        }
    }
    None
}

/// GLS flat scalar/block-list subset, not YAML and not a second metadata format.
fn block(path: &str, text: &str, first_line: usize) -> Result<Declaration, Box<Diagnostic>> {
    let error = |message: &str| {
        Box::new(Diagnostic::new(
            "E_MALFORMED_DECLARATION",
            path,
            None,
            message,
        ))
    };
    let mut scalars = BTreeMap::<String, String>::new();
    let mut edges = BTreeMap::<String, Vec<String>>::new();
    let mut list: Option<String> = None;
    for (offset, raw) in text.lines().enumerate() {
        let error = |message: &str| {
            let (field, observed) = raw.split_once(':').unwrap_or(("", raw));
            Box::new(
                Diagnostic::new(
                    "E_MALFORMED_DECLARATION",
                    path,
                    scalars.get("node").map(String::as_str),
                    message,
                )
                .at_field(first_line + offset, 1, field.trim(), observed.trim()),
            )
        };
        if raw.trim().is_empty() || raw.trim_start().starts_with('#') {
            continue;
        }
        if let Some(item) = raw.strip_prefix("  - ") {
            let key = list
                .as_ref()
                .ok_or_else(|| error("List item without a relationship field"))?;
            let item = item.trim();
            if item.is_empty() || item.contains(['[', ']', '{', '}', ',', '\"', '\'']) {
                return Err(error("Inline/nested list syntax is not supported"));
            }
            edges
                .get_mut(key)
                .expect("selected edge field")
                .push(item.into());
            continue;
        }
        if raw.starts_with([' ', '\t']) {
            return Err(error(
                "Unsupported indentation; use exactly two spaces then '- '",
            ));
        }
        let (key, tail) = raw
            .split_once(':')
            .ok_or_else(|| error("Expected key: scalar or relationship: block list"))?;
        if key.is_empty()
            || !key.bytes().all(|c| c.is_ascii_lowercase() || c == b'_')
            || (!tail.is_empty() && !tail.starts_with(' '))
        {
            return Err(error("Invalid scalar/list field syntax"));
        }
        if scalars.contains_key(key) || edges.contains_key(key) {
            return Err(error("Duplicate declaration field"));
        }
        let value = tail.trim();
        let relation = key.to_ascii_uppercase();
        if RELATIONS.contains(&relation.as_str()) || key == "knowledge_content" {
            let mut values = Vec::new();
            if !value.is_empty() {
                if value.starts_with('[') || value.ends_with(',') {
                    return Err(error("Use block lists, not inline lists"));
                }
                values.push(value.into());
            }
            edges.insert(key.into(), values);
            list = if value.is_empty() {
                Some(key.into())
            } else {
                None
            };
        } else {
            if ![
                "node",
                "kind",
                "status",
                "title",
                "track",
                "artifact",
                "verdict",
                "commit",
                "note",
                "knowledge_scope",
            ]
            .contains(&key)
            {
                return Err(error("Unknown GLS declaration field"));
            }
            if value.is_empty() {
                return Err(error("Scalar field requires a value"));
            }
            scalars.insert(key.into(), value.into());
            list = None;
        }
    }
    for required in ["node", "kind", "status", "title", "track"] {
        if !scalars.contains_key(required) {
            return Err(error(&format!("Missing required field: {required}")));
        }
    }
    let knowledge = match (
        scalars.remove("knowledge_scope"),
        edges.remove("knowledge_content"),
    ) {
        (None, None) => None,
        (Some(scope), Some(entries)) => {
            Some(super::content::spec(scope, entries).map_err(|e| error(e.0))?)
        }
        _ => {
            return Err(error(
                "knowledge_scope and knowledge_content must be declared together",
            ));
        }
    };
    Ok(Declaration {
        id: scalars.remove("node").expect("required"),
        kind: scalars.remove("kind").expect("required"),
        status: scalars.remove("status").expect("required"),
        title: scalars.remove("title").expect("required"),
        track: scalars.remove("track").expect("required"),
        artifact: scalars.remove("artifact"),
        verdict: scalars.remove("verdict"),
        commit: scalars.remove("commit"),
        note: scalars.remove("note"),
        knowledge,
        relationships: edges
            .into_iter()
            .flat_map(|(kind, targets)| {
                targets.into_iter().map(move |target| Relationship {
                    kind: kind.to_ascii_uppercase(),
                    target,
                })
            })
            .collect(),
    })
}

/// Multiple declarations per artifact are supported; nested illustrative fences are inert.
pub fn declarations(path: &str, bytes: &[u8]) -> Result<Vec<Declaration>, Box<Diagnostic>> {
    let error = |message| {
        Box::new(Diagnostic::new(
            "E_MALFORMED_DECLARATION",
            path,
            None,
            message,
        ))
    };
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(error("Save strict UTF-8 without BOM"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| error("Invalid UTF-8"))?;
    let mut fence: Option<(char, usize, bool)> = None;
    let mut content = String::new();
    let mut found = Vec::new();
    let mut first_line = 1;
    for (offset, line) in text.lines().enumerate() {
        let marker = line.chars().next().filter(|c| *c == '`' || *c == '~');
        let count = marker.map_or(0, |c| line.chars().take_while(|x| *x == c).count());
        if let Some((c, n, selected)) = fence {
            if marker == Some(c) && count >= n && line[count..].trim().is_empty() {
                if selected {
                    found.push(block(path, &content, first_line)?);
                }
                fence = None;
                content.clear();
            } else if selected {
                content.push_str(line);
                content.push('\n');
            }
        } else if count >= 3 {
            first_line = offset + 2;
            fence = Some((
                marker.expect("marker"),
                count,
                line[count..].trim() == "architecture-graph",
            ));
        }
    }
    if fence.is_some_and(|(_, _, selected)| selected) {
        return Err(error("Close the architecture-graph fence"));
    }
    Ok(found)
}
