//! Offset-preserving bridge for the pinned grammar's missing type-star modifier.
//! Original snapshots remain the interpreted/certified source. The bridge owns
//! the modifier's meaning; the parser owns the remaining export structure.
use std::collections::BTreeSet;

pub(super) fn parse(
    parser: &mut tree_sitter::Parser,
    source: &str,
) -> Option<(tree_sitter::Tree, BTreeSet<usize>)> {
    let original = parser.parse(source, None)?;
    if !original.root_node().has_error() {
        return Some((original, BTreeSet::new()));
    }
    let mut pending = vec![original.root_node()];
    let mut leaves = Vec::new();
    while let Some(node) = pending.pop() {
        if node.child_count() == 0 {
            if node.kind() != "comment" {
                leaves.push(node);
            }
        } else {
            let mut cursor = node.walk();
            pending.extend(node.children(&mut cursor));
        }
    }
    leaves.sort_by_key(|node| node.start_byte());
    let mut modifiers = BTreeSet::new();
    let mut parser_input = source.as_bytes().to_vec();
    for tokens in leaves.windows(3) {
        let [export, modifier, star] = tokens else {
            continue;
        };
        if export.kind() != "export"
            || modifier.utf8_text(source.as_bytes()).ok()? != "type"
            || star.kind() != "*"
            || !top_level(*export)
        {
            continue;
        }
        // Only this grammar-owned ASCII keyword is elided. Every other byte,
        // line and source offset remains unchanged. The role is retained below.
        parser_input[modifier.start_byte()..modifier.end_byte()].fill(b' ');
        modifiers.insert(export.start_byte());
    }
    if modifiers.is_empty() {
        return None;
    }
    let tree = parser.parse(&parser_input, None)?;
    if tree.root_node().has_error() {
        return None;
    }
    let mut proven = BTreeSet::new();
    let mut cursor = tree.root_node().walk();
    for node in tree.root_node().named_children(&mut cursor) {
        if modifiers.contains(&node.start_byte()) {
            if node.kind() != "export_statement" || node.child_by_field_name("source").is_none() {
                return None;
            }
            proven.insert(node.start_byte());
        }
    }
    drop(cursor);
    if proven != modifiers {
        return None;
    }
    Some((tree, modifiers))
}

fn top_level(node: tree_sitter::Node<'_>) -> bool {
    let mut parent = node.parent();
    while let Some(node) = parent {
        if !matches!(node.kind(), "export_statement" | "ERROR" | "program") {
            return false;
        }
        parent = node.parent();
    }
    true
}
