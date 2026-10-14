//! Exact TypeScript value-reference to direct named-import evidence.

use super::invocation_evidence::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueImportBinding {
    pub source_id: String,
    pub source_hash: String,
    pub reference_span: SourceSpan,
    pub imported_name: String,
    pub module_specifier: String,
}

/// Bind one exact identifier occurrence to one unaliased, value-space named
/// import from a direct-relative module. Framework meaning is deliberately
/// outside this mechanical boundary.
pub(crate) fn bind_direct_named_value_import(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<ValueImportBinding> {
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    if tree.root_node().has_error() {
        return None;
    }
    let reference = tree
        .root_node()
        .named_descendant_for_byte_range(reference_span.start_byte(), reference_span.end_byte())?;
    if reference.kind() != "identifier"
        || reference.start_byte() != reference_span.start_byte()
        || reference.end_byte() != reference_span.end_byte()
        || crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
            source_id,
            source,
            reference_span,
        )
        .is_some()
    {
        return None;
    }
    let name = reference.utf8_text(source.as_bytes()).ok()?;
    let mut matches = Vec::new();
    collect_imports(tree.root_node(), name, source.as_bytes(), &mut matches);
    let [module] = matches.as_slice() else {
        return None;
    };
    Some(ValueImportBinding {
        source_id: source_id.to_owned(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        reference_span,
        imported_name: name.to_owned(),
        module_specifier: (*module).to_owned(),
    })
}

fn collect_imports(
    node: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
    output: &mut Vec<String>,
) {
    if node.kind() == "import_statement" {
        if node
            .children(&mut node.walk())
            .any(|child| child.kind() == "type")
        {
            return;
        }
        let Some(module) = node
            .child_by_field_name("source")
            .and_then(|string| string.named_child(0))
            .and_then(|fragment| fragment.utf8_text(source).ok())
            .filter(|module| module.starts_with("./") || module.starts_with("../"))
        else {
            return;
        };
        collect_specifiers(node, name, module, source, output);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_imports(child, name, source, output);
    }
}

fn collect_specifiers(
    node: tree_sitter::Node<'_>,
    name: &str,
    module: &str,
    source: &[u8],
    output: &mut Vec<String>,
) {
    if node.kind() == "import_specifier" {
        if specifier_is_supported(node, name, source) {
            output.push(module.to_owned());
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_specifiers(child, name, module, source, output);
    }
}

fn specifier_is_supported(node: tree_sitter::Node<'_>, name: &str, source: &[u8]) -> bool {
    node.child_by_field_name("alias").is_none()
        && !node
            .children(&mut node.walk())
            .any(|child| child.kind() == "type")
        && node
            .child_by_field_name("name")
            .and_then(|binding| binding.utf8_text(source).ok())
            == Some(name)
}

#[cfg(test)]
#[path = "../tests/ir/typescript_value_import_binding.rs"]
mod tests;
