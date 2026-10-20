//! Private same-file TypeScript value-identifier binding evidence.
//!
//! This module deliberately owns no framework classification and performs no
//! import, module, member, or type-space resolution.

// Some provenance accessors remain reserved for compilation-local consumers
// and the complete falsification matrix.
#![allow(dead_code)]

use super::invocation_evidence::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueDeclarationKind {
    LexicalConst,
    LexicalLet,
    Parameter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceBinding {
    source_id: String,
    source_hash: String,
    reference_span: SourceSpan,
    declaration_span: SourceSpan,
    declaration_kind: ValueDeclarationKind,
}

impl ReferenceBinding {
    pub(crate) fn source_id(&self) -> &str {
        &self.source_id
    }

    pub(crate) fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub(crate) fn reference_span(&self) -> SourceSpan {
        self.reference_span
    }

    pub(crate) fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }

    pub(crate) fn declaration_kind(&self) -> ValueDeclarationKind {
        self.declaration_kind
    }
}

/// Resolve one exact direct value-identifier occurrence to its same-file
/// declaration. Unsupported or ambiguous syntax fails closed with `None`.
pub(crate) fn bind_direct_value_identifier(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
) -> Option<ReferenceBinding> {
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
    if !matches!(
        reference.kind(),
        "identifier" | "shorthand_property_identifier"
    ) || reference.start_byte() != reference_span.start_byte()
        || reference.end_byte() != reference_span.end_byte()
        || is_declaration_name(reference)
    {
        return None;
    }

    let name = reference.utf8_text(source.as_bytes()).ok()?;
    let mut scope = nearest_lexical_scope(reference)?;
    loop {
        let mut candidates = Vec::new();
        collect_scope_declarations(scope, scope, name, source.as_bytes(), &mut candidates);
        collect_function_parameters(scope, name, source.as_bytes(), &mut candidates);

        match candidates.as_slice() {
            [(declaration_span, declaration_kind)] => {
                return Some(ReferenceBinding {
                    source_id: source_id.to_owned(),
                    source_hash: crate::cache::content_hash(source.as_bytes()),
                    reference_span,
                    declaration_span: *declaration_span,
                    declaration_kind: *declaration_kind,
                });
            }
            [] => scope = nearest_lexical_scope(scope.parent()?)?,
            _ => return None,
        }
    }
}

fn is_declaration_name(node: tree_sitter::Node<'_>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    matches!(
        parent.kind(),
        "variable_declarator" | "required_parameter" | "optional_parameter"
    ) && (parent.child_by_field_name("name") == Some(node)
        || parent.child_by_field_name("pattern") == Some(node))
}

fn nearest_lexical_scope(mut node: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    loop {
        if matches!(node.kind(), "program" | "statement_block") {
            return Some(node);
        }
        node = node.parent()?;
    }
}

fn collect_scope_declarations<'tree>(
    scope: tree_sitter::Node<'tree>,
    node: tree_sitter::Node<'tree>,
    name: &str,
    source: &[u8],
    output: &mut Vec<(SourceSpan, ValueDeclarationKind)>,
) {
    if node != scope
        && (matches!(node.kind(), "program" | "statement_block") || is_function_like(node))
    {
        return;
    }

    if node.kind() == "lexical_declaration" {
        let declaration_kind = match node.child(0).map(|child| child.kind()) {
            Some("const") => Some(ValueDeclarationKind::LexicalConst),
            Some("let") => Some(ValueDeclarationKind::LexicalLet),
            _ => None,
        };
        if let Some(declaration_kind) = declaration_kind {
            let mut cursor = node.walk();
            for declarator in node
                .named_children(&mut cursor)
                .filter(|child| child.kind() == "variable_declarator")
            {
                let Some(identifier) = declarator.child_by_field_name("name") else {
                    continue;
                };
                if identifier.kind() == "identifier"
                    && identifier.utf8_text(source).ok() == Some(name)
                {
                    if let Ok(span) =
                        SourceSpan::new(identifier.start_byte(), identifier.end_byte())
                    {
                        output.push((span, declaration_kind));
                    }
                }
            }
        }
        return;
    }

    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_scope_declarations(scope, child, name, source, output);
    }
}

fn collect_function_parameters(
    scope: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
    output: &mut Vec<(SourceSpan, ValueDeclarationKind)>,
) {
    let Some(function) = scope.parent().filter(|node| is_function_like(*node)) else {
        return;
    };
    let Some(parameters) = function.child_by_field_name("parameters") else {
        return;
    };
    let mut cursor = parameters.walk();
    for parameter in parameters.named_children(&mut cursor) {
        let identifier = match parameter.kind() {
            "identifier" => Some(parameter),
            "required_parameter" | "optional_parameter" => parameter.child_by_field_name("pattern"),
            _ => None,
        };
        let Some(identifier) = identifier.filter(|node| node.kind() == "identifier") else {
            continue;
        };
        if identifier.utf8_text(source).ok() == Some(name) {
            if let Ok(span) = SourceSpan::new(identifier.start_byte(), identifier.end_byte()) {
                output.push((span, ValueDeclarationKind::Parameter));
            }
        }
    }
}

fn is_function_like(node: tree_sitter::Node<'_>) -> bool {
    matches!(
        node.kind(),
        "function_declaration"
            | "function_expression"
            | "generator_function_declaration"
            | "generator_function"
            | "arrow_function"
            | "method_definition"
    )
}

#[cfg(test)]
#[path = "../tests/ir/typescript_identifier_binding.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/ir/typescript_identifier_binding_fixtures.rs"]
mod fixture_tests;
