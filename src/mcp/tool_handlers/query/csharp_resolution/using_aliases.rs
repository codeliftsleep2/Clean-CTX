//! Request-local capture of written C# using-alias evidence.

use tree_sitter::Node;

pub(super) fn collect_direct(
    node: Node<'_>,
    source: &[u8],
    file: &str,
    scope_depth: usize,
) -> Vec<crate::ir::csharp_resolution::CSharpAliasDirective> {
    let mut aliases = Vec::new();
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_one(child, source, file, scope_depth, &mut aliases);
    }
    aliases
}

fn collect_one(
    node: Node<'_>,
    source: &[u8],
    file: &str,
    scope_depth: usize,
    aliases: &mut Vec<crate::ir::csharp_resolution::CSharpAliasDirective>,
) {
    if node.kind() == "using_directive"
        && let Ok(text) = node.utf8_text(source)
    {
        let text = text.trim();
        let rest = text
            .strip_prefix("global ")
            .unwrap_or(text)
            .strip_prefix("using ")
            .unwrap_or_default()
            .trim_end_matches(';');
        if let Some((alias, target)) = rest.split_once('=') {
            aliases.push(crate::ir::csharp_resolution::CSharpAliasDirective {
                alias: alias.trim().to_string(),
                target: target.trim().to_string(),
                scope_depth,
                file: file.to_string(),
                span: crate::ir::csharp_resolution::CSharpSourceSpan {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                },
            });
        }
    }
}
