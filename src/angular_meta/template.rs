// src/angular_meta/template.rs
//
// Angular-syntax template extractor — Tier 2 + 2.5 of the Meta-Layer.
//
// Uses `tree-sitter-html` to parse Angular templates and extract
// structural shape: tags, bindings, structural directives, and
// modern control-flow blocks (@if, @for, @switch, @defer, @let).
// Raw HTML content is NEVER included — only the structural summary.
//
// The output is a single-line shape summary suitable for a `Φtpl:`
// marker in the workspace manifest.

#[cfg(feature = "angular")]
use std::sync::OnceLock;
#[cfg(feature = "angular")]
use tree_sitter::{Language, Parser};

/// Default maximum nesting depth for tag extraction.
#[cfg(feature = "angular")]
const DEFAULT_DEPTH: usize = 4;

/// Cached tree-sitter `Language` (F-ANG-18). `tree_sitter_html::language()`
/// is a pure function that returns a `static` pointer, but caching the
/// result in a `OnceLock` lets us:
/// 1. Avoid the function call overhead per `extract_template_shape`.
/// 2. Make the call site look like a `static LANG: Language`.
///
/// We keep a fresh `Parser` per call (parsers hold mutable state and
/// are not `Sync`), but the `Language` is immutable and thread-safe.
#[cfg(feature = "angular")]
fn html_language() -> &'static Language {
    static LANG: OnceLock<Language> = OnceLock::new();
    LANG.get_or_init(|| tree_sitter_html::LANGUAGE.into())
}

/// Structural shape of an Angular template, suitable for a one-line
/// summary in the workspace manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TemplateShape {
    /// Unique HTML element tag names (e.g. `div`, `app-user-card`).
    pub tags: Vec<String>,
    /// Property bindings: `[prop]="expr"` → `"prop"`.
    pub prop_bindings: Vec<String>,
    /// Event bindings: `(event)="handler"` → `"event"`.
    pub event_bindings: Vec<String>,
    /// Two-way bindings: `[(model)]="value"` → `"model"`.
    pub two_way_bindings: Vec<String>,
    /// Structural directives: `*ngIf`, `*ngFor`, `*ngSwitch`, etc.
    pub structural_directives: Vec<String>,
    /// Interpolation expressions count (e.g. `{{ count }}`).
    pub interpolation_count: usize,
    /// Custom-element tags (tags containing a hyphen, following the
    /// Angular custom-element convention).
    pub custom_elements: Vec<String>,
    // --- Phase 2.5: Modern Angular syntax ---
    /// Modern control-flow blocks: `@if`, `@for`, `@switch`, `@else`,
    /// `@case`, `@default`, `@empty`.
    pub control_flow_blocks: Vec<String>,
    /// `@let` declarations (Angular 18+): e.g. `@let user = ...`
    pub let_declarations: Vec<String>,
    /// `@defer` block triggers (Angular 17+): e.g. `on viewport`,
    /// `on idle`, `on interaction`, `on immediate`.
    pub defer_blocks: Vec<String>,
    /// F-FULL-13: marker for parser failure on corrupt input.
    pub parse_failed: bool,
}

impl TemplateShape {
    /// Format as a single-line shape summary. Example output:
    ///
    /// ```text
    /// Φtpl:div,app-user-card @if @for [ngIf] [(ngModel)] {{count}} (click) [style.color]
    /// ```
    pub fn to_marker_line(&self) -> String {
        // F-FULL-13: distinguish parser failure from empty template
        if self.parse_failed {
            return "Φtpl:PARSE_ERROR".to_string();
        }

        let mut parts: Vec<String> = Vec::new();

        // Tags (join with commas, limit to 8 for readability).
        let tag_list: Vec<&str> = self.tags.iter().take(8).map(|s| s.as_str()).collect();
        if !tag_list.is_empty() {
            parts.push(tag_list.join(","));
        }

        // Modern control-flow blocks: @if, @for, @switch, @else, @case, @default, @empty.
        for block in &self.control_flow_blocks {
            parts.push(format!("@{}", block));
        }

        // @defer blocks (separate category — performance/lazy-loading).
        for defer in &self.defer_blocks {
            parts.push(format!("@defer({})", defer));
        }

        // @let declarations.
        if !self.let_declarations.is_empty() {
            parts.push("@let".to_string());
        }

        // Structural directives (legacy).
        for dir in &self.structural_directives {
            parts.push(format!("[{}]", dir));
        }

        // Two-way bindings (most informative binding type).
        for b in &self.two_way_bindings {
            parts.push(format!("[({})]", b));
        }

        // Interpolation count.
        if self.interpolation_count > 0 {
            parts.push(format!("{{{}}}x{}", "{}", self.interpolation_count));
        }

        // Event bindings (limit to 4).
        for b in self.event_bindings.iter().take(4) {
            parts.push(format!("({})", b));
        }

        // Property bindings (limit to 4).
        for b in self.prop_bindings.iter().take(4) {
            parts.push(format!("[{}]", b));
        }

        // Custom elements.
        for el in &self.custom_elements {
            parts.push(format!("<{}>", el));
        }

        if parts.is_empty() {
            "Φtpl:empty".to_string()
        } else {
            format!("Φtpl:{}", parts.join(" "))
        }
    }
}

/// Extract the structural shape of an Angular template from its
/// raw HTML content.
///
/// Uses tree-sitter-html for parsing. The `depth` parameter
/// controls how many levels of nesting to extract tags from
/// (default: 4). Setting depth to 0 extracts only the root element.
///
/// This function is only available when the `angular` feature is enabled.
/// When disabled, it returns an empty `TemplateShape`.
#[cfg(feature = "angular")]
pub fn extract_template_shape(html: &str) -> TemplateShape {
    extract_template_shape_with_depth(html, DEFAULT_DEPTH)
}

/// Extract template shape with a custom depth limit.
///
/// This function is only available when the `angular` feature is enabled.
/// When disabled, it returns an empty `TemplateShape` with `parse_failed` set to false.
#[cfg(feature = "angular")]
pub fn extract_template_shape_with_depth(html: &str, depth: usize) -> TemplateShape {
    let mut shape = TemplateShape::default();

    if html.trim().is_empty() {
        return shape;
    }

    let mut parser = Parser::new();
    parser.set_language(html_language()).ok();
    let tree = match parser.parse(html.as_bytes(), None) {
        Some(t) => t,
        None => {
            // F-FULL-13: surface parser failure to the caller
            shape.parse_failed = true;
            return shape;
        }
    };

    let root = tree.root_node();
    walk_node(root, html, depth, 0, &mut shape);

    // Deduplicate.
    shape.tags.sort();
    shape.tags.dedup();
    shape.custom_elements.sort();
    shape.custom_elements.dedup();
    shape.prop_bindings.sort();
    shape.prop_bindings.dedup();
    shape.event_bindings.sort();
    shape.event_bindings.dedup();
    shape.two_way_bindings.sort();
    shape.two_way_bindings.dedup();
    shape.structural_directives.sort();
    shape.structural_directives.dedup();
    shape.control_flow_blocks.sort();
    shape.control_flow_blocks.dedup();
    shape.defer_blocks.sort();
    shape.defer_blocks.dedup();

    shape
}

/// Recursively walk the tree-sitter AST to extract Angular-specific
/// structural information.
///
/// tree-sitter-html 0.20.x node structure:
/// ```text
/// fragment
///   element
///     start_tag
///       tag_name
///       attribute
///         attribute_name  (*ngIf, [title], (click), [(ngModel)])
///     text / element (children)
///     end_tag
///   self_closing_tag   (<app-avatar />)
///     tag_name
///     attribute
///       attribute_name
/// ```
#[cfg(feature = "angular")]
fn walk_node(
    node: tree_sitter::Node,
    source: &str,
    max_depth: usize,
    current_depth: usize,
    shape: &mut TemplateShape,
) {
    let kind = node.kind();

    match kind {
        // Standard open+close elements: <div>...</div>
        "element" => {
            process_element_node(node, source, max_depth, current_depth, shape);
        }
        // Self-closing tags: <app-avatar />, <input />, <br />
        "self_closing_tag" => {
            process_self_closing_tag_node(node, source, shape);
            // No recursion into children (self-closing tags have none).
        }
        "text" => {
            // Count interpolations: `{{ expr }}`.
            // Also detect modern Angular control-flow syntax that
            // tree-sitter-html cannot parse as HTML.
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                let count = text.matches("{{").count();
                shape.interpolation_count += count;

                // Scan for modern @-syntax control flow.
                extract_modern_syntax_from_text(text, shape);
            }
        }
        // Handle fragment (root node) and other container nodes
        // by recursing into their children.
        "fragment" | "document" => {
            if current_depth < max_depth {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    walk_node(child, source, max_depth, current_depth + 1, shape);
                }
            }
        }
        _ => {
            // Unknown node types: recurse into children.
            if current_depth < max_depth {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    walk_node(child, source, max_depth, current_depth + 1, shape);
                }
            }
        }
    }
}

/// Process an `element` node (has start_tag, children, end_tag).
///
/// tree-sitter-html 0.20.x represents XHTML self-closing elements
/// (`<app-heavy />`) as an `element` wrapping a `self_closing_tag`
/// child, rather than having a `start_tag` child. We must check for
/// both structures.
#[cfg(feature = "angular")]
fn process_element_node(
    node: tree_sitter::Node,
    source: &str,
    max_depth: usize,
    current_depth: usize,
    shape: &mut TemplateShape,
) {
    // Check if this element wraps a self_closing_tag (XHTML-style `<tag />`).
    // In that case, extract tag name and attributes from the self_closing_tag child.
    if let Some(self_closing) = find_child(node, "self_closing_tag") {
        process_self_closing_tag_node(self_closing, source, shape);
        // No further children to recurse into for XHTML self-closing elements.
        return;
    }

    // Extract tag name from the start_tag child.
    if let Some(tag_name) = extract_tag_name_from_element(node, source) {
        shape.tags.push(tag_name.clone());
        // Custom elements contain a hyphen.
        if tag_name.contains('-') {
            shape.custom_elements.push(tag_name);
        }
    }

    // Extract attributes from the start_tag child.
    if let Some(start_tag) = find_child(node, "start_tag") {
        extract_attributes(start_tag, source, shape);
    }

    // Recurse into child elements (not start_tag/end_tag).
    if current_depth < max_depth {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            let child_kind = child.kind();
            // Only recurse into element and text nodes, skip start_tag, end_tag.
            if child_kind == "element"
                || child_kind == "text"
                || child_kind == "self_closing_tag"
            {
                walk_node(child, source, max_depth, current_depth + 1, shape);
            }
        }
    }
}

/// Process a `self_closing_tag` node (e.g. `<app-avatar />`).
#[cfg(feature = "angular")]
fn process_self_closing_tag_node(
    node: tree_sitter::Node,
    source: &str,
    shape: &mut TemplateShape,
) {
    // Extract tag name directly from the self_closing_tag node.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let child_kind = child.kind();
        if child_kind == "tag_name" {
            if let Ok(tag) = child.utf8_text(source.as_bytes()) {
                shape.tags.push(tag.to_string());
                if tag.contains('-') {
                    shape.custom_elements.push(tag.to_string());
                }
            }
        } else if child_kind == "attribute" {
            // Extract attribute name.
            let mut inner_cursor = child.walk();
            for inner in child.children(&mut inner_cursor) {
                if inner.kind() == "attribute_name" {
                    if let Ok(attr) = inner.utf8_text(source.as_bytes()) {
                        capture_attribute(attr, shape);
                    }
                    break;
                }
            }
        }
    }
}

/// Capture an attribute name into the appropriate binding/directive category.
#[cfg(feature = "angular")]
fn capture_attribute(attr_name: &str, shape: &mut TemplateShape) {
    // Structural directives: *ngIf, *ngFor, *ngSwitch, etc.
    if let Some(directive) = attr_name.strip_prefix('*') {
        shape.structural_directives.push(directive.to_string());
        return;
    }

    // Two-way binding: [(name)]="..."
    if attr_name.starts_with("[(") && attr_name.ends_with(")]") {
        let inner = &attr_name[2..attr_name.len() - 2];
        shape.two_way_bindings.push(inner.to_string());
        return;
    }

    // Property binding: [name]="..."
    if attr_name.starts_with('[') && attr_name.ends_with(']') {
        let inner = &attr_name[1..attr_name.len() - 1];
        shape.prop_bindings.push(inner.to_string());
        return;
    }

    // Event binding: (name)="..."
    if attr_name.starts_with('(') && attr_name.ends_with(')') {
        let inner = &attr_name[1..attr_name.len() - 1];
        shape.event_bindings.push(inner.to_string());
    }
}

/// Check if `text` contains the given `@keyword` as a standalone token.
///
/// Uses word-boundary heuristics: the `@keyword` must be preceded by
/// start-of-text, whitespace, `{`, or `}`; and followed by whitespace,
/// `(`, `{`, `;`, or end-of-text.
#[cfg(feature = "angular")]
fn contains_at_keyword(text: &str, keyword: &str) -> bool {
    let needle = format!("@{}", keyword);
    let mut search_start = 0;
    while let Some(pos) = text[search_start..].find(&needle) {
        let absolute_pos = search_start + pos;
        // Check character before @ (if any)
        let before_ok = if absolute_pos == 0 {
            true
        } else {
            let ch = text.as_bytes()[absolute_pos - 1];
            ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r' || ch == b'{' || ch == b'}'
        };
        // Check character after keyword (if any)
        let after_pos = absolute_pos + needle.len();
        let after_ok = if after_pos >= text.len() {
            true
        } else {
            let ch = text.as_bytes()[after_pos];
            ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r'
                || ch == b'(' || ch == b'{' || ch == b';'
        };
        if before_ok && after_ok {
            return true;
        }
        // Move past this match and try again (avoid false positives
        // like `@formatter` matching `@for`).
        search_start = absolute_pos + 1;
    }
    false
}

/// Check if `text` contains `@defer (on <trigger>)` and return
/// the trigger name(s) found.
#[cfg(feature = "angular")]
fn extract_defer_triggers(text: &str) -> Vec<String> {
    let mut triggers = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = text[search_start..].find("@defer") {
        let absolute_pos = search_start + pos;
        let rest = &text[absolute_pos + "@defer".len()..];

        // Check if followed by whitespace then `(on ...)`
        let rest_trimmed = rest.trim_start();
        if let Some(on_rest) = rest_trimmed.strip_prefix("(on") {
            // Extract the trigger word: after "(on" and optional whitespace
            let after_on = on_rest.trim_start();
            if let Some(trigger_end) = after_on.find(|c: char| !c.is_alphanumeric() && c != '_') {
                let trigger = &after_on[..trigger_end];
                if !trigger.is_empty() {
                    triggers.push(trigger.to_string());
                }
            }
        }

        search_start = absolute_pos + 1;
    }
    triggers
}

/// Extract modern Angular syntax tokens (@if, @for, @switch, @defer,
/// @let, etc.) from a text node.
#[cfg(feature = "angular")]
fn extract_modern_syntax_from_text(text: &str, shape: &mut TemplateShape) {
    // --- Control-flow blocks (non-defer) ---

    if contains_at_keyword(text, "if") {
        shape.control_flow_blocks.push("if".to_string());
    }
    if contains_at_keyword(text, "else") {
        shape.control_flow_blocks.push("else".to_string());
    }
    if contains_at_keyword(text, "for") {
        shape.control_flow_blocks.push("for".to_string());
    }
    if contains_at_keyword(text, "empty") {
        shape.control_flow_blocks.push("empty".to_string());
    }
    if contains_at_keyword(text, "switch") {
        shape.control_flow_blocks.push("switch".to_string());
    }
    if contains_at_keyword(text, "case") {
        shape.control_flow_blocks.push("case".to_string());
    }
    if contains_at_keyword(text, "default") {
        shape.control_flow_blocks.push("default".to_string());
    }

    // --- @defer blocks (separate category) ---
    // Extract named triggers: @defer (on viewport), @defer (on idle), etc.
    let had_triggers = !extract_defer_triggers(text).is_empty();
    for trigger in extract_defer_triggers(text) {
        shape.defer_blocks.push(trigger);
    }
    // Detect bare @defer (no trigger) — only if no trigger was found in
    // this text node.
    if contains_at_keyword(text, "defer") && !contains_at_keyword(text, "deferred") && !had_triggers {
        shape.defer_blocks.push("default".to_string());
    }

    // @placeholder, @loading, @error (defer sub-blocks)
    for sub in &["placeholder", "loading", "error"] {
        if contains_at_keyword(text, sub) {
            shape.defer_blocks.push(sub.to_string());
        }
    }

    // --- @let declarations (separate category) ---
    if contains_at_keyword(text, "let") {
        shape.let_declarations.push("let".to_string());
    }
}

/// Find the first child of `node` with the given kind.
#[cfg(feature = "angular")]
fn find_child<'a>(node: tree_sitter::Node<'a>, kind: &str) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find(|&child| child.kind() == kind)
}

/// Extract the tag name from an `element` node by finding its
/// `start_tag` child and then the `tag_name` within it.
#[cfg(feature = "angular")]
fn extract_tag_name_from_element(
    node: tree_sitter::Node,
    source: &str,
) -> Option<String> {
    let start_tag = find_child(node, "start_tag")?;
    let mut cursor = start_tag.walk();
    for child in start_tag.children(&mut cursor) {
        if child.kind() == "tag_name" {
            return child
                .utf8_text(source.as_bytes())
                .ok()
                .map(|s| s.to_string());
        }
    }
    None
}

/// Extract Angular-specific attributes (bindings, directives) from
/// a `start_tag` node.
#[cfg(feature = "angular")]
fn extract_attributes(
    start_tag: tree_sitter::Node,
    source: &str,
    shape: &mut TemplateShape,
) {
    let mut cursor = start_tag.walk();
    for child in start_tag.children(&mut cursor) {
        if child.kind() != "attribute" {
            continue;
        }

        // Get the attribute_name child.
        let attr_name = {
            let mut inner_cursor = child.walk();
            let mut found = None;
            for inner in child.children(&mut inner_cursor) {
                if inner.kind() == "attribute_name" {
                    found = inner
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(|s| s.to_string());
                    break;
                }
            }
            match found {
                Some(n) => n,
                None => continue,
            }
        };

        capture_attribute(&attr_name, shape);
    }
}

#[cfg(all(test, feature = "angular"))]
#[path = "../tests/angular_meta/template.rs"]
mod tests;
