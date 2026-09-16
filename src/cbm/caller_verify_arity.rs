//! C# arity evidence extraction and request-local parse reuse.
//!
//! This module is the source-side half of caller verification: it turns a
//! candidate/target C# file into the two shapes the verifier compares —
//! declared parameter shapes and observed call-site argument counts — and it
//! owns the memo that makes one verification batch parse each file at most
//! once. The verdicts, the retained candidate identity, and the response
//! metadata live in [`crate::cbm::caller_verify`].
//!
//! Nothing here caches anything across requests: the memo is created for one
//! verification operation and dropped with it, so verification truth can never
//! go stale behind a cached parse. Which bytes are current stays
//! [`crate::mcp::McpState::read_source`]'s decision.

use std::collections::HashMap;
use std::sync::Arc;
use tree_sitter::{Node, Parser, Tree};

/// Declared parameter shape of one method declaration.
///
/// `declared` counts every parameter (the extension receiver included);
/// `minimum_explicit`/`maximum_explicit` describe the argument counts a call
/// site may legally pass once the receiver, optional parameters, and `params`
/// are accounted for. `flexible` marks a shape whose accepted arity is not a
/// single exact number, so it can never establish a unique overload by itself.
#[derive(Debug, Clone, Copy)]
pub(super) struct ArityShape {
    pub(super) declared: usize,
    pub(super) minimum_explicit: usize,
    pub(super) maximum_explicit: Option<usize>,
    pub(super) flexible: bool,
}

impl ArityShape {
    pub(super) fn accepts(self, observed: usize) -> bool {
        observed >= self.minimum_explicit
            && self
                .maximum_explicit
                .is_none_or(|maximum| observed <= maximum)
    }

    pub(super) fn exact_explicit(self) -> Option<usize> {
        match self.maximum_explicit {
            Some(maximum) if maximum == self.minimum_explicit && !self.flexible => Some(maximum),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Declaration {
    pub(super) arity: ArityShape,
}

/// Request-local reuse of parsed C# verification representations.
///
/// One verification batch (one response) can verify several symbols whose
/// target or candidate source is the same canonical file — a broad
/// `search_graph` answer commonly holds several overloads declared in one
/// file, and every one of them reads the same candidate sources. The memo
/// parses each keyed path once for the lifetime of that batch.
///
/// It is deliberately not a cache: it is created for one verification
/// operation, never persisted, never shared across requests, and it does not
/// decide *which* bytes are current (that remains
/// [`crate::mcp::McpState::read_source`]'s mtime/size contract), so a cached
/// parse can never outlive the source it came from.
#[derive(Default)]
pub(crate) struct ParseMemo {
    trees: HashMap<String, Option<Arc<Tree>>>,
}

impl ParseMemo {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Parse representations this batch produced — one per keyed path, since a
    /// key is parsed at most once.
    ///
    /// Structural evidence for the duplicate-work regression: a count, not a
    /// heuristic, and therefore deterministic.
    #[cfg(test)]
    pub(crate) fn parses(&self) -> usize {
        self.trees.len()
    }

    /// The parse for `key`, produced at most once per batch.
    ///
    /// A source that fails to parse is memoized as `None` as well: a broken
    /// file is not re-parsed for every symbol that references it.
    pub(crate) fn tree(&mut self, key: &str, source: &str) -> Option<Arc<Tree>> {
        if let Some(memoized) = self.trees.get(key) {
            return memoized.clone();
        }
        let parsed = parse_tree(source).map(Arc::new);
        self.trees.insert(key.to_string(), parsed.clone());
        parsed
    }
}

/// Parse a C# source that is syntactically complete.
///
/// An error-recovering tree is reported as `None` so a file with syntax errors
/// is never treated as evidence.
fn parse_tree(source: &str) -> Option<Tree> {
    let mut parser = Parser::new();
    let language = crate::compression::language::safe_csharp_language()?;
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    (!tree.root_node().has_error()).then_some(tree)
}

/// Method declarations named `method_name` in an already-parsed source.
pub(super) fn declarations_of(
    tree: &Tree,
    source: &str,
    method_name: &str,
    type_name: Option<&str>,
) -> Vec<Declaration> {
    let mut nodes = Vec::new();
    collect_kind(tree.root_node(), "method_declaration", &mut nodes);
    nodes
        .into_iter()
        .filter(|node| node_text(node.child_by_field_name("name"), source) == Some(method_name))
        .filter(|node| {
            type_name.is_none_or(|name| enclosing_type_name(*node, source) == Some(name))
        })
        .filter_map(|node| declaration_arity(node, source).map(|arity| Declaration { arity }))
        .collect()
}

/// Explicit argument counts of every invocation of `method_name` in an
/// already-parsed source.
pub(super) fn invocation_arities_of(tree: &Tree, source: &str, method_name: &str) -> Vec<usize> {
    let mut nodes = Vec::new();
    collect_kind(tree.root_node(), "invocation_expression", &mut nodes);
    nodes
        .into_iter()
        .filter(|node| invocation_name(*node, source) == Some(method_name))
        .filter_map(|node| node.child_by_field_name("arguments"))
        .map(|arguments| arguments.named_child_count())
        .collect()
}

fn declaration_arity(node: Node<'_>, source: &str) -> Option<ArityShape> {
    let parameters = node.child_by_field_name("parameters")?;
    let mut cursor = parameters.walk();
    let parameter_nodes: Vec<Node<'_>> = parameters
        .named_children(&mut cursor)
        .filter(|parameter| parameter.kind() == "parameter")
        .collect();
    let parameter_list_text = parameters.utf8_text(source.as_bytes()).ok()?;
    let has_params = has_word(parameter_list_text, "params");
    let declared = parameter_nodes.len() + usize::from(has_params);
    let texts: Vec<&str> = parameter_nodes
        .iter()
        .filter_map(|parameter| parameter.utf8_text(source.as_bytes()).ok())
        .collect();
    if texts.len() != parameter_nodes.len() {
        return None;
    }
    let extension_receiver = texts.first().is_some_and(|text| has_word(text, "this"));
    let receiver_adjustment = usize::from(extension_receiver);
    let optional = texts.iter().filter(|text| text.contains('=')).count();
    let minimum_explicit = declared
        .saturating_sub(receiver_adjustment)
        .saturating_sub(optional)
        .saturating_sub(usize::from(has_params));
    let maximum_explicit = (!has_params).then_some(declared.saturating_sub(receiver_adjustment));
    Some(ArityShape {
        declared,
        minimum_explicit,
        maximum_explicit,
        flexible: optional > 0 || has_params,
    })
}

fn invocation_name<'a>(node: Node<'_>, source: &'a str) -> Option<&'a str> {
    let function = node.child_by_field_name("function")?;
    if function.kind() == "member_access_expression" {
        node_text(function.child_by_field_name("name"), source)
    } else {
        let text = function.utf8_text(source.as_bytes()).ok()?;
        Some(text.rsplit('.').next().unwrap_or(text))
    }
}

fn enclosing_type_name<'a>(mut node: Node<'_>, source: &'a str) -> Option<&'a str> {
    while let Some(parent) = node.parent() {
        if matches!(
            parent.kind(),
            "class_declaration" | "struct_declaration" | "record_declaration"
        ) {
            return node_text(parent.child_by_field_name("name"), source);
        }
        node = parent;
    }
    None
}

fn collect_kind<'tree>(node: Node<'tree>, kind: &str, output: &mut Vec<Node<'tree>>) {
    if node.kind() == kind {
        output.push(node);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_kind(child, kind, output);
    }
}

fn node_text<'a>(node: Option<Node<'_>>, source: &'a str) -> Option<&'a str> {
    node?.utf8_text(source.as_bytes()).ok()
}

fn has_word(text: &str, needle: &str) -> bool {
    text.split(|character: char| !character.is_alphanumeric() && character != '_')
        .any(|word| word == needle)
}

pub(super) fn arities_overlap(left: ArityShape, right: ArityShape) -> bool {
    let lower = left.minimum_explicit.max(right.minimum_explicit);
    match (left.maximum_explicit, right.maximum_explicit) {
        (Some(left_max), Some(right_max)) => lower <= left_max.min(right_max),
        (Some(left_max), None) => lower <= left_max,
        (None, Some(right_max)) => lower <= right_max,
        (None, None) => true,
    }
}
