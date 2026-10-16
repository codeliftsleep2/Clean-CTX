//! Ordered Angular template selector occurrence authority.

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) struct SourceSpan {
    pub(crate) start_byte: usize,
    pub(crate) end_byte: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) enum TemplateControlKind {
    If,
    ElseIf,
    Else,
    For,
    Empty,
    Switch,
    Case,
    Default,
    Defer,
    Placeholder,
    Loading,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(crate) enum TemplateControlSyntax {
    Legacy,
    Modern,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct TemplateControlFrame {
    pub(crate) kind: TemplateControlKind,
    pub(crate) syntax: TemplateControlSyntax,
    pub(crate) written_header: String,
    pub(crate) span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct SelectorOccurrence {
    pub(crate) written_selector: String,
    pub(crate) span: SourceSpan,
    pub(crate) control_ancestry: Vec<TemplateControlFrame>,
}

#[derive(Clone)]
struct ModernRegion {
    frame: TemplateControlFrame,
    body_start: usize,
    body_end: usize,
}

pub(crate) fn extract(source: &str) -> Vec<SelectorOccurrence> {
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language = tree_sitter_html::LANGUAGE.into();
    if parser.set_language(&language).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source.as_bytes(), None) else {
        return Vec::new();
    };
    let modern = modern_regions(source);
    let mut occurrences = Vec::new();
    walk(
        tree.root_node(),
        source,
        &modern,
        &mut Vec::new(),
        &mut occurrences,
    );
    occurrences.sort_by_key(|occurrence| occurrence.span.start_byte);
    occurrences
}

fn walk(
    node: tree_sitter::Node,
    source: &str,
    modern: &[ModernRegion],
    legacy: &mut Vec<TemplateControlFrame>,
    occurrences: &mut Vec<SelectorOccurrence>,
) {
    match node.kind() {
        "element" => walk_element(node, source, modern, legacy, occurrences),
        "self_closing_tag" => capture_tag(node, source, modern, legacy, occurrences),
        _ => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                walk(child, source, modern, legacy, occurrences);
            }
        }
    }
}

fn walk_element(
    node: tree_sitter::Node,
    source: &str,
    modern: &[ModernRegion],
    legacy: &mut Vec<TemplateControlFrame>,
    occurrences: &mut Vec<SelectorOccurrence>,
) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.named_children(&mut cursor).collect();
    if let Some(tag) = children
        .iter()
        .copied()
        .find(|child| matches!(child.kind(), "start_tag" | "self_closing_tag"))
    {
        let added = push_legacy_frames(tag, node, source, legacy);
        capture_tag(tag, source, modern, legacy, occurrences);
        for child in children {
            if !matches!(child.kind(), "start_tag" | "end_tag" | "self_closing_tag") {
                walk(child, source, modern, legacy, occurrences);
            }
        }
        legacy.truncate(legacy.len().saturating_sub(added));
    }
}

fn capture_tag(
    tag: tree_sitter::Node,
    source: &str,
    modern: &[ModernRegion],
    legacy: &[TemplateControlFrame],
    occurrences: &mut Vec<SelectorOccurrence>,
) {
    let Some(name) = tag_name(tag, source) else {
        return;
    };
    if !name.contains('-') {
        return;
    }
    let span = SourceSpan {
        start_byte: tag.start_byte(),
        end_byte: tag.end_byte(),
    };
    let mut control_ancestry = legacy.to_vec();
    control_ancestry.extend(
        modern
            .iter()
            .filter(|region| {
                span.start_byte >= region.body_start && span.end_byte <= region.body_end
            })
            .map(|region| region.frame.clone()),
    );
    control_ancestry.sort_by(|left, right| {
        left.span
            .start_byte
            .cmp(&right.span.start_byte)
            .then_with(|| right.span.end_byte.cmp(&left.span.end_byte))
    });
    occurrences.push(SelectorOccurrence {
        written_selector: name,
        span,
        control_ancestry,
    });
}

fn push_legacy_frames(
    tag: tree_sitter::Node,
    element: tree_sitter::Node,
    source: &str,
    legacy: &mut Vec<TemplateControlFrame>,
) -> usize {
    let before = legacy.len();
    let mut cursor = tag.walk();
    for attribute in tag
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "attribute")
    {
        let Some((name, value)) = attribute_parts(attribute, source) else {
            continue;
        };
        let kind = match name.as_str() {
            "*ngIf" => Some(TemplateControlKind::If),
            "*ngFor" => Some(TemplateControlKind::For),
            "[ngSwitch]" => Some(TemplateControlKind::Switch),
            "*ngSwitchCase" => Some(TemplateControlKind::Case),
            "*ngSwitchDefault" => Some(TemplateControlKind::Default),
            _ => None,
        };
        if let Some(kind) = kind {
            legacy.push(TemplateControlFrame {
                kind,
                syntax: TemplateControlSyntax::Legacy,
                written_header: value
                    .map_or_else(|| name.clone(), |value| format!("{name}({value})")),
                span: SourceSpan {
                    start_byte: element.start_byte(),
                    end_byte: element.end_byte(),
                },
            });
        }
    }
    legacy.len() - before
}

fn tag_name(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == "tag_name")
        .and_then(|child| child.utf8_text(source.as_bytes()).ok())
        .map(str::to_string)
}

fn attribute_parts(attribute: tree_sitter::Node, source: &str) -> Option<(String, Option<String>)> {
    let mut name = None;
    let mut value = None;
    let mut cursor = attribute.walk();
    for child in attribute.named_children(&mut cursor) {
        match child.kind() {
            "attribute_name" => {
                name = child.utf8_text(source.as_bytes()).ok().map(str::to_string);
            }
            "quoted_attribute_value" | "attribute_value" => {
                value = child
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(|raw| strip_quotes(raw.trim()).to_string());
            }
            _ => {}
        }
    }
    name.map(|name| (name, value))
}

fn strip_quotes(raw: &str) -> &str {
    let bytes = raw.as_bytes();
    if bytes.len() >= 2 && matches!(bytes[0], b'"' | b'\'') && bytes.last() == Some(&bytes[0]) {
        &raw[1..raw.len() - 1]
    } else {
        raw
    }
}

fn modern_regions(source: &str) -> Vec<ModernRegion> {
    let mut regions = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find('@') {
        let start = cursor + relative;
        let Some((kind, token_len)) = modern_kind(&source[start..]) else {
            cursor = start + 1;
            continue;
        };
        if !control_boundary(source, start, token_len) {
            cursor = start + 1;
            continue;
        }
        let Some(open) = find_open_brace(source, start + token_len) else {
            cursor = start + token_len;
            continue;
        };
        let Some(close) = matching_brace(source, open) else {
            cursor = start + token_len;
            continue;
        };
        regions.push(ModernRegion {
            frame: TemplateControlFrame {
                kind,
                syntax: TemplateControlSyntax::Modern,
                written_header: collapse(&source[start..=open]),
                span: SourceSpan {
                    start_byte: start,
                    end_byte: close + 1,
                },
            },
            body_start: open + 1,
            body_end: close,
        });
        cursor = start + token_len;
    }
    regions
}

fn modern_kind(text: &str) -> Option<(TemplateControlKind, usize)> {
    const TOKENS: &[(&str, TemplateControlKind)] = &[
        ("@else if", TemplateControlKind::ElseIf),
        ("@placeholder", TemplateControlKind::Placeholder),
        ("@loading", TemplateControlKind::Loading),
        ("@default", TemplateControlKind::Default),
        ("@switch", TemplateControlKind::Switch),
        ("@defer", TemplateControlKind::Defer),
        ("@empty", TemplateControlKind::Empty),
        ("@error", TemplateControlKind::Error),
        ("@case", TemplateControlKind::Case),
        ("@else", TemplateControlKind::Else),
        ("@for", TemplateControlKind::For),
        ("@if", TemplateControlKind::If),
    ];
    TOKENS
        .iter()
        .find_map(|(token, kind)| text.starts_with(token).then_some((*kind, token.len())))
}

fn control_boundary(source: &str, start: usize, token_len: usize) -> bool {
    let bytes = source.as_bytes();
    let before = start == 0
        || (!bytes[start - 1].is_ascii_alphanumeric() && !matches!(bytes[start - 1], b'_' | b'$'));
    let after = start + token_len;
    before
        && (after >= bytes.len()
            || (!bytes[after].is_ascii_alphanumeric() && !matches!(bytes[after], b'_' | b'$')))
}
fn find_open_brace(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote = None;
    let mut escaped = false;
    let mut parens = 0usize;
    for (index, byte) in bytes.iter().copied().enumerate().skip(start) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(active) = quote {
            if byte == active {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"') || byte == 96 {
            quote = Some(byte);
            continue;
        }
        match byte {
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b'{' if parens == 0 => return Some(index),
            b';' if parens == 0 => return None,
            _ => {}
        }
    }
    None
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate().skip(open) {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(active) = quote {
            if byte == active {
                quote = None;
            }
            continue;
        }
        if matches!(byte, b'\'' | b'"') || byte == 96 {
            quote = Some(byte);
            continue;
        }
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn collapse(raw: &str) -> String {
    raw.split_whitespace().collect::<Vec<_>>().join(" ")
}
