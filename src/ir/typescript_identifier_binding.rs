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
    FunctionVar,
    CatchParameter,
    Parameter,
    PatternConst,
    PatternLet,
    PatternVar,
    PatternParameter,
    PatternCatch,
    LoopLet,
    LoopConst,
    LoopVar,
    ArrowParameter,
    FunctionSelf,
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
    // Preserve this bounded adapter's const/let/parameter contract. The
    // canonical tree-taking binder retains broader declaration evidence;
    // legacy consumers must not silently gain a new declaration role.
    bind_value_identifier_in_tree(source_id, source, reference_span, tree.root_node()).filter(
        |binding| {
            matches!(
                binding.declaration_kind(),
                ValueDeclarationKind::LexicalConst
                    | ValueDeclarationKind::LexicalLet
                    | ValueDeclarationKind::Parameter
            )
        },
    )
}

pub(crate) fn bind_value_identifier_in_tree(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
    root: tree_sitter::Node<'_>,
) -> Option<ReferenceBinding> {
    lookup_value_identifier_in_tree(source_id, source, reference_span, root)
        .ok()
        .flatten()
}

pub(crate) fn lookup_value_identifier_in_tree(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
    root: tree_sitter::Node<'_>,
) -> Result<Option<ReferenceBinding>, ()> {
    lookup_value_site(source_id, source, reference_span, root, false, false)
}

/// Query the value binding at a type-name site without promoting its type
/// interpretation. Used only by demanded constructor eligibility acquisition.
#[cfg(feature = "typescript")]
pub(crate) fn lookup_value_at_type_site_in_tree(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
    root: tree_sitter::Node<'_>,
) -> Result<Option<ReferenceBinding>, ()> {
    lookup_value_site(source_id, source, reference_span, root, true, false)
}

/// Constructor decorators evaluate outside the constructor parameter scope.
#[cfg(feature = "typescript")]
pub(crate) fn lookup_constructor_decorator_value_in_tree(
    source_id: &str,
    source: &str,
    span: SourceSpan,
    root: tree_sitter::Node<'_>,
) -> Result<Option<ReferenceBinding>, ()> {
    lookup_value_site(source_id, source, span, root, false, true)
}

fn lookup_value_site(
    source_id: &str,
    source: &str,
    reference_span: SourceSpan,
    root: tree_sitter::Node<'_>,
    allow_type_site: bool,
    skip_constructor: bool,
) -> Result<Option<ReferenceBinding>, ()> {
    if root.has_error() {
        return Err(());
    }
    let reference = root
        .named_descendant_for_byte_range(reference_span.start_byte(), reference_span.end_byte())
        .ok_or(())?;
    if !(matches!(
        reference.kind(),
        "identifier" | "shorthand_property_identifier"
    ) || (allow_type_site && reference.kind() == "type_identifier"))
        || reference.start_byte() != reference_span.start_byte()
        || reference.end_byte() != reference_span.end_byte()
        || is_declaration_name(reference)
    {
        return Err(());
    }

    let name = reference.utf8_text(source.as_bytes()).map_err(|_| ())?;
    let mut scope = value_scope(reference, skip_constructor, source).ok_or(())?;
    loop {
        let mut candidates = Vec::new();
        collect_scope_declarations(scope, scope, name, source.as_bytes(), &mut candidates);
        collect_function_parameters(scope, name, source.as_bytes(), &mut candidates);
        if scope.kind() == "catch_clause"
            && let Some(parameter) = scope.child_by_field_name("parameter")
        {
            push_pattern(
                parameter,
                name,
                source.as_bytes(),
                ValueDeclarationKind::CatchParameter,
                &mut candidates,
            );
        }
        if scope.kind() == "program" || is_function_like(scope) {
            collect_function_vars(scope, scope, name, source.as_bytes(), &mut candidates);
        }
        if candidates.is_empty()
            && matches!(scope.kind(), "function_expression" | "generator_function")
            && let Some(identifier) = scope.child_by_field_name("name")
        {
            push_pattern(
                identifier,
                name,
                source.as_bytes(),
                ValueDeclarationKind::FunctionSelf,
                &mut candidates,
            );
        }

        match candidates.as_slice() {
            [(declaration_span, declaration_kind)] => {
                return Ok(Some(ReferenceBinding {
                    source_id: source_id.to_owned(),
                    source_hash: crate::cache::content_hash(source.as_bytes()),
                    reference_span,
                    declaration_span: *declaration_span,
                    declaration_kind: *declaration_kind,
                }));
            }
            [] => match scope
                .parent()
                .and_then(|parent| value_scope(parent, skip_constructor, source))
            {
                Some(parent) => scope = parent,
                None => return Ok(None),
            },
            _ => return Err(()),
        }
    }
}

fn collect_function_vars<'tree>(
    scope: tree_sitter::Node<'tree>,
    node: tree_sitter::Node<'tree>,
    name: &str,
    source: &[u8],
    output: &mut Vec<(SourceSpan, ValueDeclarationKind)>,
) {
    if node != scope
        && (is_function_like(node)
            || matches!(node.kind(), "class_body" | "internal_module" | "module"))
    {
        return;
    }
    if node.kind() == "variable_declaration" {
        let mut cursor = node.walk();
        for declarator in node.named_children(&mut cursor) {
            if let Some(pattern) = declarator.child_by_field_name("name") {
                push_pattern(
                    pattern,
                    name,
                    source,
                    ValueDeclarationKind::FunctionVar,
                    output,
                );
            }
        }
        return;
    }
    if node.kind() == "for_in_statement"
        && node
            .child_by_field_name("kind")
            .is_some_and(|kind| kind.kind() == "var")
        && let Some(pattern) = node.child_by_field_name("left")
    {
        push_pattern(pattern, name, source, ValueDeclarationKind::LoopVar, output);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_function_vars(scope, child, name, source, output);
    }
}

fn is_declaration_name(node: tree_sitter::Node<'_>) -> bool {
    super::typescript_binding_pattern::is_binding_name(node)
}

fn nearest_lexical_scope(mut node: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    loop {
        if is_lexical_scope(node) {
            return Some(node);
        }
        node = node.parent()?;
    }
}

fn is_lexical_scope(node: tree_sitter::Node<'_>) -> bool {
    matches!(
        node.kind(),
        "program" | "statement_block" | "for_statement" | "for_in_statement" | "catch_clause"
    ) || is_function_like(node)
}

fn collect_scope_declarations<'tree>(
    scope: tree_sitter::Node<'tree>,
    node: tree_sitter::Node<'tree>,
    name: &str,
    source: &[u8],
    output: &mut Vec<(SourceSpan, ValueDeclarationKind)>,
) {
    if node != scope && is_lexical_scope(node) {
        return;
    }
    if node.kind() == "for_in_statement" {
        let kind = match node.child_by_field_name("kind").map(|kind| kind.kind()) {
            Some("let") => Some(ValueDeclarationKind::LoopLet),
            Some("const") => Some(ValueDeclarationKind::LoopConst),
            _ => None,
        };
        if let (Some(kind), Some(pattern)) = (kind, node.child_by_field_name("left")) {
            push_pattern(pattern, name, source, kind, output);
        }
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
                push_pattern(identifier, name, source, declaration_kind, output);
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
    let function = if is_function_like(scope) {
        Some(scope)
    } else {
        scope.parent().filter(|node| is_function_like(*node))
    };
    let Some(function) = function else {
        return;
    };
    let Some(parameters) = function.child_by_field_name("parameters") else {
        if function.kind() == "arrow_function"
            && let Some(parameter) = function.child_by_field_name("parameter")
        {
            push_pattern(
                parameter,
                name,
                source,
                ValueDeclarationKind::ArrowParameter,
                output,
            );
        }
        return;
    };
    let mut cursor = parameters.walk();
    for parameter in parameters.named_children(&mut cursor) {
        let identifier = match parameter.kind() {
            "identifier" => Some(parameter),
            "required_parameter" | "optional_parameter" => parameter.child_by_field_name("pattern"),
            _ => None,
        };
        if let Some(pattern) = identifier {
            push_pattern(
                pattern,
                name,
                source,
                ValueDeclarationKind::Parameter,
                output,
            );
        }
    }
}

fn push_pattern(
    pattern: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
    kind: ValueDeclarationKind,
    output: &mut Vec<(SourceSpan, ValueDeclarationKind)>,
) {
    let kind = if pattern.kind() == "identifier" {
        kind
    } else {
        match kind {
            ValueDeclarationKind::LexicalConst => ValueDeclarationKind::PatternConst,
            ValueDeclarationKind::LexicalLet => ValueDeclarationKind::PatternLet,
            ValueDeclarationKind::FunctionVar => ValueDeclarationKind::PatternVar,
            ValueDeclarationKind::Parameter => ValueDeclarationKind::PatternParameter,
            ValueDeclarationKind::CatchParameter => ValueDeclarationKind::PatternCatch,
            other => other,
        }
    };
    if let Some(names) = super::typescript_binding_pattern::names(pattern) {
        for identifier in names {
            if identifier.utf8_text(source).ok() == Some(name)
                && let Ok(span) = SourceSpan::new(identifier.start_byte(), identifier.end_byte())
            {
                output.push((span, kind));
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

fn value_scope<'tree>(
    mut node: tree_sitter::Node<'tree>,
    skip_constructor: bool,
    source: &str,
) -> Option<tree_sitter::Node<'tree>> {
    loop {
        let scope = nearest_lexical_scope(node)?;
        if skip_constructor
            && scope.kind() == "method_definition"
            && scope
                .child_by_field_name("name")
                .is_some_and(|name| name.utf8_text(source.as_bytes()).ok() == Some("constructor"))
        {
            node = scope.parent()?;
        } else {
            return Some(scope);
        }
    }
}
