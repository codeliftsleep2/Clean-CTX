//! Bounded annotation site selection; no Angular meaning or acquisition here.
use super::{Obligation, Span};
pub(super) fn direct_import(
    unit: &crate::ir::typescript_resolution::syntax::Unit,
    name: &str,
    member: Option<&str>,
    module: &str,
    export: &str,
) -> bool {
    use crate::ir::typescript_resolution::syntax::Binding;
    match unit.locals.get(name).map(Vec::as_slice) {
        Some(
            [
                Binding::Import {
                    module: actual,
                    imported,
                    ..
                },
            ],
        ) => member.is_none() && actual == module && imported == export,
        Some([Binding::Namespace { module: actual, .. }]) => {
            actual == module && member == Some(export)
        }
        _ => false,
    }
}
pub(super) fn select(
    tree: &tree_sitter::Tree,
    object: Span,
    source: &str,
) -> Result<(String, Option<String>, Span), Obligation> {
    let node = tree
        .root_node()
        .named_descendant_for_byte_range(object.start, object.end)
        .ok_or(Obligation::UnsupportedSyntax)?;
    if node.kind() != "object" || node.start_byte() != object.start || node.end_byte() != object.end
    {
        return Err(Obligation::UnsupportedSyntax);
    }
    let array = node
        .parent()
        .filter(|n| n.kind() == "array")
        .ok_or(Obligation::UnsupportedExpression)?;
    let declarator = array
        .parent()
        .filter(|n| {
            n.kind() == "variable_declarator" && n.child_by_field_name("value") == Some(array)
        })
        .ok_or(Obligation::UnsupportedExpression)?;
    let lexical = declarator
        .parent()
        .filter(|n| n.kind() == "lexical_declaration")
        .ok_or(Obligation::UnsupportedExpression)?;
    if lexical
        .child_by_field_name("kind")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        != Some("const")
    {
        return Err(Obligation::UnsupportedExpression);
    }
    let mut parent = lexical.parent();
    while let Some(n) = parent {
        if !matches!(n.kind(), "export_statement" | "program") {
            return Err(Obligation::UnsupportedLexicalScope);
        }
        parent = n.parent();
    }
    let annotation = declarator
        .child_by_field_name("type")
        .ok_or(Obligation::UnsupportedExpression)?;
    let mut cursor = annotation.walk();
    let children: Vec<_> = annotation
        .named_children(&mut cursor)
        .filter(|n| n.kind() != "comment")
        .collect();
    if children.len() != 1 {
        return Err(Obligation::UnsupportedSyntax);
    }
    let ty = children[0];
    if !matches!(ty.kind(), "type_identifier" | "nested_type_identifier") {
        return Err(Obligation::UnsupportedExpression);
    }
    let text = ty
        .utf8_text(source.as_bytes())
        .map_err(|_| Obligation::UnsupportedSyntax)?;
    let parts: Vec<_> = text.split('.').collect();
    if parts.is_empty() || parts.len() > 2 || text.contains(char::is_whitespace) {
        return Err(Obligation::UnsupportedExpression);
    }
    Ok((
        parts[0].into(),
        parts.get(1).map(|s| (*s).into()),
        Span {
            start: ty.start_byte(),
            end: ty.end_byte(),
        },
    ))
}
