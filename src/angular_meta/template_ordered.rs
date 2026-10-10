//! Source-ordered Angular template presentation for Medium and High fidelity.
//!
//! tree-sitter HTML owns element boundaries and nesting. Angular 17+ block
//! syntax remains text in that grammar, so text events preserve every written
//! block token and brace in source order. Angular 15–16 structural directives
//! additionally emit normalized behavioral headers before their owning element.

use crate::compression::Fidelity;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplateEvent {
    Content(String),
    Comment(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct OrderedTemplate {
    events: Vec<TemplateEvent>,
    custom_elements: Vec<String>,
    interpolation_count: usize,
    parse_failed: bool,
    control_consumed_until: usize,
}

impl OrderedTemplate {
    pub(crate) fn render(&self, fidelity: Fidelity) -> Vec<String> {
        if self.parse_failed {
            return vec!["Φtpl:PARSE_ERROR".to_string()];
        }
        let mut lines = vec!["Φtpl:".to_string()];
        for event in &self.events {
            match event {
                TemplateEvent::Content(line) => lines.push(line.clone()),
                TemplateEvent::Comment(line)
                    if matches!(fidelity, Fidelity::High | Fidelity::Edit | Fidelity::Verbatim) =>
                {
                    lines.push(line.clone());
                }
                TemplateEvent::Comment(_) => {}
            }
        }
        if matches!(fidelity, Fidelity::High | Fidelity::Edit | Fidelity::Verbatim)
            && self.interpolation_count > 0
        {
            lines.push(format!("{{{{}}}}x{}", self.interpolation_count));
        }
        if lines.len() == 1 {
            vec!["Φtpl:empty".to_string()]
        } else {
            lines
        }
    }

    pub(crate) fn prime_ng_markers(&self) -> Vec<String> {
        let mut markers: Vec<_> = self
            .custom_elements
            .iter()
            .filter(|tag| tag.starts_with("p-"))
            .map(|tag| format!("Φ{}:", tag))
            .collect();
        markers.sort();
        markers.dedup();
        markers
    }
}

#[cfg(feature = "angular")]
pub(crate) fn extract(html: &str) -> OrderedTemplate {
    if html.trim().is_empty() {
        return OrderedTemplate::default();
    }
    let mut parser = tree_sitter::Parser::new();
    let language: tree_sitter::Language = tree_sitter_html::LANGUAGE.into();
    if parser.set_language(&language).is_err() {
        return OrderedTemplate {
            parse_failed: true,
            ..OrderedTemplate::default()
        };
    }
    let Some(tree) = parser.parse(html.as_bytes(), None) else {
        return OrderedTemplate {
            parse_failed: true,
            ..OrderedTemplate::default()
        };
    };
    let mut template = OrderedTemplate::default();
    walk(tree.root_node(), html, &mut template);
    template
}

#[cfg(not(feature = "angular"))]
pub(crate) fn extract(_html: &str) -> OrderedTemplate {
    OrderedTemplate::default()
}

#[cfg(feature = "angular")]
fn walk(node: tree_sitter::Node, source: &str, template: &mut OrderedTemplate) {
    match node.kind() {
        "element" => walk_element(node, source, template),
        "self_closing_tag" => push_tag(node, source, template),
        "text" => {
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                template.interpolation_count += text.matches("{{").count();
                push_text_node(node, source, template);
            }
        }
        "comment" => {
            if let Ok(comment) = node.utf8_text(source.as_bytes()) {
                let comment = collapse_multiline(comment);
                if !comment.is_empty() {
                    template.events.push(TemplateEvent::Comment(comment));
                }
            }
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                walk(child, source, template);
            }
        }
    }
}

#[cfg(feature = "angular")]
fn walk_element(node: tree_sitter::Node, source: &str, template: &mut OrderedTemplate) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.named_children(&mut cursor).collect();
    if let Some(self_closing) = children
        .iter()
        .copied()
        .find(|child| child.kind() == "self_closing_tag")
    {
        push_tag(self_closing, source, template);
        return;
    }

    for child in children {
        match child.kind() {
            "start_tag" => push_tag(child, source, template),
            "end_tag" => push_raw_content(child, source, template),
            _ => walk(child, source, template),
        }
    }
}

#[cfg(feature = "angular")]
fn push_tag(node: tree_sitter::Node, source: &str, template: &mut OrderedTemplate) {
    push_legacy_headers(node, source, template);
    if let Some(tag) = tag_name(node, source)
        && tag.contains('-')
    {
        template.custom_elements.push(tag);
    }
    push_raw_content(node, source, template);
}

#[cfg(feature = "angular")]
fn push_legacy_headers(
    tag: tree_sitter::Node,
    source: &str,
    template: &mut OrderedTemplate,
) {
    let mut cursor = tag.walk();
    for attribute in tag
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "attribute")
    {
        let Some((name, value)) = attribute_parts(attribute, source) else {
            continue;
        };
        match (name.as_str(), value.as_deref()) {
            ("*ngIf", Some(condition)) => {
                template.events.push(TemplateEvent::Content(format!(
                    "@if({condition})"
                )));
                template.events.push(TemplateEvent::Content(format!(
                    "*ngIf({condition})"
                )));
            }
            ("*ngFor", Some(expression)) => {
                if let Some((variable, iterable)) = legacy_for_parts(expression) {
                    template.events.push(TemplateEvent::Content(format!(
                        "@for({variable} of {iterable})"
                    )));
                }
                template.events.push(TemplateEvent::Content(format!(
                    "*ngFor({expression})"
                )));
            }
            ("*ngSwitchCase", Some(expression)) => {
                template.events.push(TemplateEvent::Content(format!(
                    "@case({expression})"
                )));
            }
            ("*ngSwitchDefault", _) => template
                .events
                .push(TemplateEvent::Content("@default".to_string())),
            _ => {}
        }
    }
}

#[cfg(feature = "angular")]
fn attribute_parts(
    attribute: tree_sitter::Node,
    source: &str,
) -> Option<(String, Option<String>)> {
    let mut name = None;
    let mut value = None;
    let mut cursor = attribute.walk();
    for child in attribute.named_children(&mut cursor) {
        match child.kind() {
            "attribute_name" => {
                name = child
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(str::to_string);
            }
            "quoted_attribute_value" | "attribute_value" => {
                value = child.utf8_text(source.as_bytes()).ok().map(|raw| {
                    raw.trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string()
                });
            }
            _ => {}
        }
    }
    name.map(|name| (name, value))
}

fn legacy_for_parts(expression: &str) -> Option<(&str, &str)> {
    let first = expression.split(';').next()?.trim();
    let split = first.find(" of ")?;
    let variable = first[..split].trim().trim_start_matches("let ").trim();
    let iterable = first[split + 4..].trim();
    (!variable.is_empty() && !iterable.is_empty()).then_some((variable, iterable))
}

#[cfg(feature = "angular")]
fn tag_name(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    let result = node
        .named_children(&mut cursor)
        .find(|child| child.kind() == "tag_name")
        .and_then(|child| child.utf8_text(source.as_bytes()).ok())
        .map(str::to_string);
    result
}

#[cfg(feature = "angular")]
fn push_raw_content(
    node: tree_sitter::Node,
    source: &str,
    template: &mut OrderedTemplate,
) {
    if let Ok(raw) = node.utf8_text(source.as_bytes()) {
        let raw = collapse_multiline(raw);
        if !raw.is_empty() {
            template.events.push(TemplateEvent::Content(raw));
        }
    }
}

#[cfg(feature = "angular")]
fn push_text_node(
    node: tree_sitter::Node,
    source: &str,
    template: &mut OrderedTemplate,
) {
    let mut cursor = node.start_byte().max(template.control_consumed_until);
    let node_end = node.end_byte();
    if cursor >= node_end {
        return;
    }
    while cursor < node_end {
        let Some(control_start) = find_control_start(source, cursor, node_end) else {
            push_text(&source[cursor..node_end], template);
            break;
        };
        push_text(&source[cursor..control_start], template);
        let Some((header, control_end)) = control_header(source, control_start) else {
            push_text(&source[control_start..control_start + 1], template);
            cursor = control_start + 1;
            continue;
        };
        template
            .events
            .push(TemplateEvent::Content(normalize_modern_headers(&header)));
        template.control_consumed_until = control_end;
        cursor = control_end;
    }
}

fn find_control_start(source: &str, start: usize, end: usize) -> Option<usize> {
    const TOKENS: &[&str] = &[
        "@else if", "@placeholder", "@loading", "@default", "@switch",
        "@defer", "@empty", "@error", "@case", "@else", "@for", "@let", "@if",
    ];
    let mut best = None;
    for token in TOKENS {
        let mut search = start;
        while search < end {
            let Some(relative) = source[search..end].find(token) else {
                break;
            };
            let position = search + relative;
            if control_boundary(source, position, token.len()) {
                best = Some(best.map_or(position, |current: usize| current.min(position)));
                break;
            }
            search = position + 1;
        }
    }
    best
}

fn control_boundary(source: &str, position: usize, token_len: usize) -> bool {
    let bytes = source.as_bytes();
    let before = position == 0
        || matches!(bytes[position - 1], b' ' | b'\t' | b'\n' | b'\r' | b'{' | b'}');
    let after = position + token_len;
    before
        && (after >= bytes.len()
            || matches!(bytes[after], b' ' | b'\t' | b'\n' | b'\r' | b'(' | b'{' | b';'))
}

fn control_header(source: &str, start: usize) -> Option<(String, usize)> {
    let rest = &source[start..];
    if rest.starts_with("@let") {
        let end = rest.find(';').map(|index| index + 1).unwrap_or(rest.len());
        return Some((rest[..end].trim().to_string(), start + end));
    }
    let brace = rest.find('{')?;
    let open = rest[..brace].find('(');
    let mut end = brace + 1;
    if let Some(open) = open {
        let close = matching_paren(&rest[open..])?;
        let close = open + close;
        let suffix = &rest[close + 1..];
        if let Some(relative_brace) = suffix.find('{') {
            end = close + 1 + relative_brace + 1;
        } else {
            end = close + 1;
        }
    }
    Some((rest[..end].trim().to_string(), start + end))
}
fn push_text(text: &str, template: &mut OrderedTemplate) {
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        template
            .events
            .push(TemplateEvent::Content(normalize_modern_headers(line)));
    }
}

fn collapse_multiline(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_modern_headers(line: &str) -> String {
    let mut normalized = line.to_string();
    for keyword in ["if", "else if", "switch", "case", "defer"] {
        normalized = normalized.replace(
            &format!("@{keyword} ("),
            &format!("@{keyword}("),
        );
    }
    normalize_for_headers(&normalized)
}

fn normalize_for_headers(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(index) = rest.find("@for (") {
        output.push_str(&rest[..index]);
        let header = &rest[index + 5..];
        let Some(close) = matching_paren(header) else {
            output.push_str(&rest[index..]);
            return output;
        };
        let inner = &header[1..close];
        let mut parts = inner.split(';').map(str::trim);
        let first = parts.next().unwrap_or_default();
        output.push_str("@for(");
        output.push_str(first);
        output.push(')');
        for part in parts.filter(|part| !part.is_empty()) {
            output.push(' ');
            output.push_str(part);
        }
        rest = &header[close + 1..];
    }
    output.push_str(rest);
    output
}

fn matching_paren(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.first().copied() != Some(b'(') {
        return None;
    }
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;
    for (index, byte) in bytes.iter().copied().enumerate() {
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
        if matches!(byte, b'\'' | b'"' | b'\x60') {
            quote = Some(byte);
            continue;
        }
        match byte {
            b'(' => depth += 1,
            b')' => {
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
