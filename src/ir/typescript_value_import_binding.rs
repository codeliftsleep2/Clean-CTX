//! Exact TypeScript value-reference to direct named-import evidence.

use super::invocation_evidence::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValueImportBinding {
    pub source_id: String,
    pub source_hash: String,
    pub reference_span: SourceSpan,
    pub imported_name: String,
    pub local_name: String,
    pub module_specifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NamespaceValueImportBinding {
    pub source_id: String,
    pub source_hash: String,
    pub reference_span: SourceSpan,
    pub namespace_name: String,
    pub imported_name: String,
    pub module_specifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NamedObjectMemberImportBinding {
    pub source_id: String,
    pub source_hash: String,
    pub reference_span: SourceSpan,
    pub imported_object: String,
    pub local_object: String,
    pub member_name: String,
    pub module_specifier: String,
}

/// Bind `LocalAlias.member` through one direct-relative named value import.
/// This is declaration mechanics only; no framework role is implied.
pub(crate) fn bind_direct_named_object_member_import(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<NamedObjectMemberImportBinding> {
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
    if reference.kind() != "member_expression"
        || reference.start_byte() != reference_span.start_byte()
        || reference.end_byte() != reference_span.end_byte()
    {
        return None;
    }
    let object = reference.child_by_field_name("object")?;
    let member = reference.child_by_field_name("property")?;
    if object.kind() != "identifier" || member.kind() != "property_identifier" {
        return None;
    }
    let object_span = SourceSpan::new(object.start_byte(), object.end_byte()).ok()?;
    let binding = bind_direct_named_value_import_allow_alias(source_id, source, object_span)?;
    Some(NamedObjectMemberImportBinding {
        source_id: source_id.to_owned(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        reference_span,
        imported_object: binding.imported_name,
        local_object: binding.local_name,
        member_name: member.utf8_text(source.as_bytes()).ok()?.to_owned(),
        module_specifier: binding.module_specifier,
    })
}

/// Bind one exact identifier occurrence to one value-space named
/// import from a direct-relative module. Framework meaning is deliberately
/// outside this mechanical boundary. An `as` alias is normalized into separate
/// imported and local names; the local spelling never becomes declaration
/// identity.
pub(crate) fn bind_direct_named_value_import(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<ValueImportBinding> {
    bind_named_value_import_with_scope(source_id, source, reference_span, true, false)
}

pub(crate) fn bind_direct_named_value_import_allow_alias(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<ValueImportBinding> {
    bind_named_value_import_with_scope(source_id, source, reference_span, true, true)
}

/// Bind one exact identifier occurrence to one value-space named
/// import. Framework consumers interpret the returned module specifier.
pub(crate) fn bind_named_value_import(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<ValueImportBinding> {
    bind_named_value_import_with_scope(source_id, source, reference_span, false, false)
}

pub(crate) fn bind_named_value_import_allow_alias(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<ValueImportBinding> {
    bind_named_value_import_with_scope(source_id, source, reference_span, false, true)
}

fn bind_named_value_import_with_scope(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
    relative_only: bool,
    allow_alias: bool,
) -> Option<ValueImportBinding> {
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    bind_named_value_import_in_tree(
        source_id,
        source,
        reference_span,
        relative_only,
        allow_alias,
        tree.root_node(),
    )
}

pub(crate) fn bind_named_value_import_in_tree(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
    relative_only: bool,
    allow_alias: bool,
    root: tree_sitter::Node<'_>,
) -> Option<ValueImportBinding> {
    if root.has_error() {
        return None;
    }
    let reference = root
        .named_descendant_for_byte_range(reference_span.start_byte(), reference_span.end_byte())?;
    if !matches!(
        reference.kind(),
        "identifier" | "shorthand_property_identifier"
    ) || reference.start_byte() != reference_span.start_byte()
        || reference.end_byte() != reference_span.end_byte()
        || !matches!(
            crate::ir::typescript_identifier_binding::lookup_value_identifier_in_tree(
                source_id,
                source,
                reference_span,
                root,
            ),
            Ok(None)
        )
    {
        return None;
    }
    let name = reference.utf8_text(source.as_bytes()).ok()?;
    let mut matches = Vec::new();
    collect_imports(root, name, source.as_bytes(), relative_only, &mut matches);
    let [(module, imported_name)] = matches.as_slice() else {
        return None;
    };
    if !allow_alias && imported_name != name {
        return None;
    }
    Some(ValueImportBinding {
        source_id: source_id.to_owned(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        reference_span,
        imported_name: imported_name.clone(),
        local_name: name.to_owned(),
        module_specifier: module.clone(),
    })
}

/// Bind one exact `Namespace.member` occurrence through one value-space
/// namespace import from a direct-relative module. Export and framework
/// authority remain outside this mechanical boundary.
pub(crate) fn bind_direct_namespace_value_import(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<NamespaceValueImportBinding> {
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
    if reference.kind() != "member_expression"
        || reference.start_byte() != reference_span.start_byte()
        || reference.end_byte() != reference_span.end_byte()
    {
        return None;
    }
    let namespace = reference.child_by_field_name("object")?;
    let member = reference.child_by_field_name("property")?;
    if namespace.kind() != "identifier"
        || member.kind() != "property_identifier"
        || !matches!(
            crate::ir::typescript_identifier_binding::lookup_value_identifier_in_tree(
                source_id,
                source,
                SourceSpan::new(namespace.start_byte(), namespace.end_byte()).ok()?,
                tree.root_node(),
            ),
            Ok(None)
        )
    {
        return None;
    }
    let namespace_name = namespace.utf8_text(source.as_bytes()).ok()?;
    let imported_name = member.utf8_text(source.as_bytes()).ok()?;
    let mut matches = Vec::new();
    collect_namespace_imports(
        tree.root_node(),
        namespace_name,
        source.as_bytes(),
        &mut matches,
    );
    let [module] = matches.as_slice() else {
        return None;
    };
    Some(NamespaceValueImportBinding {
        source_id: source_id.to_owned(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        reference_span,
        namespace_name: namespace_name.to_owned(),
        imported_name: imported_name.to_owned(),
        module_specifier: (*module).to_owned(),
    })
}

fn collect_namespace_imports(
    node: tree_sitter::Node<'_>,
    namespace_name: &str,
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
        if has_namespace_import(node, namespace_name, source) {
            output.push(module.to_owned());
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_namespace_imports(child, namespace_name, source, output);
    }
}

fn has_namespace_import(node: tree_sitter::Node<'_>, name: &str, source: &[u8]) -> bool {
    if node.kind() == "namespace_import"
        && node
            .named_child(0)
            .and_then(|binding| binding.utf8_text(source).ok())
            == Some(name)
    {
        return true;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| has_namespace_import(child, name, source))
}

fn collect_imports(
    node: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
    relative_only: bool,
    output: &mut Vec<(String, String)>,
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
            .filter(|module| {
                !relative_only || module.starts_with("./") || module.starts_with("../")
            })
        else {
            return;
        };
        collect_specifiers(node, name, module, source, output);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_imports(child, name, source, relative_only, output);
    }
}

fn collect_specifiers(
    node: tree_sitter::Node<'_>,
    name: &str,
    module: &str,
    source: &[u8],
    output: &mut Vec<(String, String)>,
) {
    if node.kind() == "import_specifier" {
        if let Some(imported) = supported_specifier(node, name, source) {
            output.push((module.to_owned(), imported));
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_specifiers(child, name, module, source, output);
    }
}

fn supported_specifier(
    node: tree_sitter::Node<'_>,
    local_name: &str,
    source: &[u8],
) -> Option<String> {
    if node
        .children(&mut node.walk())
        .any(|child| child.kind() == "type")
    {
        return None;
    }
    let imported = node.child_by_field_name("name")?.utf8_text(source).ok()?;
    let local = node
        .child_by_field_name("alias")
        .or_else(|| node.child_by_field_name("name"))?
        .utf8_text(source)
        .ok()?;
    (local == local_name).then(|| imported.to_owned())
}

#[cfg(test)]
#[path = "../tests/ir/typescript_value_import_binding.rs"]
mod tests;
