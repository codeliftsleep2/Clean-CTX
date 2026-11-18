//! Private same-file TypeScript direct type-reference to named-import evidence.

// Prereq3b intentionally freezes private compilation-local evidence before
// the separately authorized Store-qualification consumer exists.
#![allow(dead_code)]

use super::invocation_evidence::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeImportBinding {
    source_id: String,
    source_hash: String,
    type_reference_span: SourceSpan,
    type_head_span: SourceSpan,
    import_statement_span: SourceSpan,
    local_binding_span: SourceSpan,
    imported_symbol_span: SourceSpan,
    imported_symbol_name: String,
    module_specifier_span: SourceSpan,
    module_specifier: String,
}

impl TypeImportBinding {
    pub(crate) fn source_id(&self) -> &str {
        &self.source_id
    }

    pub(crate) fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub(crate) fn type_reference_span(&self) -> SourceSpan {
        self.type_reference_span
    }

    pub(crate) fn type_head_span(&self) -> SourceSpan {
        self.type_head_span
    }

    pub(crate) fn import_statement_span(&self) -> SourceSpan {
        self.import_statement_span
    }

    pub(crate) fn local_binding_span(&self) -> SourceSpan {
        self.local_binding_span
    }

    pub(crate) fn imported_symbol_span(&self) -> SourceSpan {
        self.imported_symbol_span
    }

    pub(crate) fn imported_symbol_name(&self) -> &str {
        &self.imported_symbol_name
    }

    pub(crate) fn module_specifier_span(&self) -> SourceSpan {
        self.module_specifier_span
    }

    pub(crate) fn module_specifier(&self) -> &str {
        &self.module_specifier
    }
}

/// Bind an exact supported direct type-reference occurrence to its same-file,
/// direct, unaliased named import. Unsupported or ambiguous forms fail closed.
pub(crate) fn bind_type_reference_to_named_import(
    source_id: &str,
    source: &str,
    type_reference_span: SourceSpan,
) -> Option<TypeImportBinding> {
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    if tree.root_node().has_error() {
        return None;
    }

    let reference = tree.root_node().named_descendant_for_byte_range(
        type_reference_span.start_byte(),
        type_reference_span.end_byte(),
    )?;
    if reference.start_byte() != type_reference_span.start_byte()
        || reference.end_byte() != type_reference_span.end_byte()
    {
        return None;
    }
    let head = match reference.kind() {
        "type_identifier" => reference,
        "generic_type" => reference.child_by_field_name("name")?,
        _ => return None,
    };
    if head.kind() != "type_identifier" {
        return None;
    }
    let name = head.utf8_text(source.as_bytes()).ok()?;
    if has_type_space_collision(tree.root_node(), name, source.as_bytes()) {
        return None;
    }

    let mut imports = Vec::new();
    collect_matching_imports(tree.root_node(), name, source.as_bytes(), &mut imports);
    let [(statement, binding, module)] = imports.as_slice() else {
        return None;
    };
    let module_value = module.utf8_text(source.as_bytes()).ok()?;
    Some(TypeImportBinding {
        source_id: source_id.to_owned(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        type_reference_span,
        type_head_span: span(head)?,
        import_statement_span: span(*statement)?,
        local_binding_span: span(*binding)?,
        imported_symbol_span: span(*binding)?,
        imported_symbol_name: name.to_owned(),
        module_specifier_span: span(*module)?,
        module_specifier: module_value.to_owned(),
    })
}

fn span(node: tree_sitter::Node<'_>) -> Option<SourceSpan> {
    SourceSpan::new(node.start_byte(), node.end_byte()).ok()
}

fn collect_matching_imports<'tree>(
    node: tree_sitter::Node<'tree>,
    name: &str,
    source: &[u8],
    output: &mut Vec<(
        tree_sitter::Node<'tree>,
        tree_sitter::Node<'tree>,
        tree_sitter::Node<'tree>,
    )>,
) {
    if node.kind() == "import_statement" {
        collect_from_import(node, name, source, output);
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_matching_imports(child, name, source, output);
    }
}

fn collect_from_import<'tree>(
    statement: tree_sitter::Node<'tree>,
    name: &str,
    source: &[u8],
    output: &mut Vec<(
        tree_sitter::Node<'tree>,
        tree_sitter::Node<'tree>,
        tree_sitter::Node<'tree>,
    )>,
) {
    if has_direct_child(statement, "type") {
        return;
    }
    let Some(module) = statement
        .child_by_field_name("source")
        .filter(|node| node.kind() == "string")
        .and_then(single_string_fragment)
    else {
        return;
    };
    let Some(clause) = statement
        .named_children(&mut statement.walk())
        .find(|node| node.kind() == "import_clause")
    else {
        return;
    };
    let Some(named) = clause
        .named_children(&mut clause.walk())
        .find(|node| node.kind() == "named_imports")
    else {
        return;
    };
    let mut cursor = named.walk();
    for specifier in named
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "import_specifier")
    {
        if specifier.child_by_field_name("alias").is_some() || has_direct_child(specifier, "type") {
            continue;
        }
        let Some(binding) = specifier
            .child_by_field_name("name")
            .filter(|node| node.kind() == "identifier")
        else {
            continue;
        };
        if binding.utf8_text(source).ok() == Some(name) {
            output.push((statement, binding, module));
        }
    }
}

fn single_string_fragment(string: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    let fragments: Vec<_> = string
        .named_children(&mut string.walk())
        .filter(|node| node.kind() == "string_fragment")
        .collect();
    let [fragment] = fragments.as_slice() else {
        return None;
    };
    Some(*fragment)
}

fn has_direct_child(node: tree_sitter::Node<'_>, kind: &str) -> bool {
    node.children(&mut node.walk())
        .any(|child| child.kind() == kind)
}

fn has_type_space_collision(node: tree_sitter::Node<'_>, name: &str, source: &[u8]) -> bool {
    if matches!(
        node.kind(),
        "type_alias_declaration"
            | "interface_declaration"
            | "class_declaration"
            | "abstract_class_declaration"
            | "enum_declaration"
            | "type_parameter"
    ) && node
        .child_by_field_name("name")
        .and_then(|identifier| identifier.utf8_text(source).ok())
        == Some(name)
    {
        return true;
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| has_type_space_collision(child, name, source))
}

#[cfg(test)]
#[path = "../tests/ir/typescript_type_import_binding.rs"]
mod tests;
