//! Private same-class TypeScript `this.<property>` binding evidence.

// Prereq3a intentionally freezes private compilation-local evidence before
// the separately authorized Prereq3b/Store-qualification consumer exists.
#![allow(dead_code)]

use super::invocation_evidence::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThisPropertyBinding {
    source_id: String,
    source_hash: String,
    occurrence_span: SourceSpan,
    property_span: SourceSpan,
    enclosing_class_span: SourceSpan,
    declaration_name_span: SourceSpan,
    type_reference_span: Option<SourceSpan>,
}

impl ThisPropertyBinding {
    pub(crate) fn source_id(&self) -> &str {
        &self.source_id
    }

    pub(crate) fn source_hash(&self) -> &str {
        &self.source_hash
    }

    pub(crate) fn occurrence_span(&self) -> SourceSpan {
        self.occurrence_span
    }

    pub(crate) fn property_span(&self) -> SourceSpan {
        self.property_span
    }

    pub(crate) fn enclosing_class_span(&self) -> SourceSpan {
        self.enclosing_class_span
    }

    pub(crate) fn declaration_name_span(&self) -> SourceSpan {
        self.declaration_name_span
    }

    pub(crate) fn type_reference_span(&self) -> Option<SourceSpan> {
        self.type_reference_span
    }
}

/// Bind one exact supported `this.<property>` occurrence to its same-class
/// constructor parameter-property declaration. Unsupported syntax fails closed.
pub(crate) fn bind_this_property(
    source_id: &str,
    source: &str,
    occurrence_span: SourceSpan,
) -> Option<ThisPropertyBinding> {
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    if tree.root_node().has_error() {
        return None;
    }

    let occurrence = tree.root_node().named_descendant_for_byte_range(
        occurrence_span.start_byte(),
        occurrence_span.end_byte(),
    )?;
    if occurrence.kind() != "member_expression"
        || occurrence.start_byte() != occurrence_span.start_byte()
        || occurrence.end_byte() != occurrence_span.end_byte()
    {
        return None;
    }
    let object = occurrence.child_by_field_name("object")?;
    let property = occurrence.child_by_field_name("property")?;
    if object.kind() != "this" || property.kind() != "property_identifier" {
        return None;
    }
    if occurrence
        .children(&mut occurrence.walk())
        .any(|child| matches!(child.kind(), "optional_chain" | "?."))
    {
        return None;
    }

    let method = enclosing_instance_method(occurrence, source.as_bytes())?;
    let class = enclosing_class(method)?;
    let name = property.utf8_text(source.as_bytes()).ok()?;
    let candidates = parameter_properties(class, name, source.as_bytes());
    let [(declaration, type_reference)] = candidates.as_slice() else {
        return None;
    };

    Some(ThisPropertyBinding {
        source_id: source_id.to_owned(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        occurrence_span,
        property_span: SourceSpan::new(property.start_byte(), property.end_byte()).ok()?,
        enclosing_class_span: SourceSpan::new(class.start_byte(), class.end_byte()).ok()?,
        declaration_name_span: *declaration,
        type_reference_span: *type_reference,
    })
}

fn enclosing_instance_method<'tree>(
    node: tree_sitter::Node<'tree>,
    source: &[u8],
) -> Option<tree_sitter::Node<'tree>> {
    let mut parent = node.parent()?;
    loop {
        if parent.kind() == "method_definition" {
            if parent
                .children(&mut parent.walk())
                .any(|child| child.kind() == "static")
            {
                return None;
            }
            let name = parent.child_by_field_name("name")?;
            return (name.utf8_text(source).ok()? != "constructor").then_some(parent);
        }
        if matches!(
            parent.kind(),
            "function_declaration"
                | "function_expression"
                | "generator_function_declaration"
                | "generator_function"
                | "arrow_function"
        ) {
            return None;
        }
        parent = parent.parent()?;
    }
}

fn enclosing_class(mut node: tree_sitter::Node<'_>) -> Option<tree_sitter::Node<'_>> {
    loop {
        if node.kind() == "class_declaration" {
            return Some(node);
        }
        node = node.parent()?;
    }
}

fn parameter_properties(
    class: tree_sitter::Node<'_>,
    name: &str,
    source: &[u8],
) -> Vec<(SourceSpan, Option<SourceSpan>)> {
    let Some(body) = class.child_by_field_name("body") else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut body_cursor = body.walk();
    for constructor in body
        .named_children(&mut body_cursor)
        .filter(|node| node.kind() == "method_definition")
        .filter(|node| {
            node.child_by_field_name("name")
                .and_then(|name| name.utf8_text(source).ok())
                == Some("constructor")
        })
    {
        let Some(parameters) = constructor.child_by_field_name("parameters") else {
            continue;
        };
        let mut parameter_cursor = parameters.walk();
        for parameter in parameters
            .named_children(&mut parameter_cursor)
            .filter(|node| matches!(node.kind(), "required_parameter" | "optional_parameter"))
        {
            if !is_parameter_property(parameter) {
                continue;
            }
            let Some(identifier) = parameter
                .child_by_field_name("pattern")
                .or_else(|| parameter.child_by_field_name("name"))
                .filter(|node| node.kind() == "identifier")
            else {
                continue;
            };
            if identifier.utf8_text(source).ok() != Some(name) {
                continue;
            }
            let Some(declaration) =
                SourceSpan::new(identifier.start_byte(), identifier.end_byte()).ok()
            else {
                continue;
            };
            let type_reference = parameter
                .child_by_field_name("type")
                .and_then(|annotation| annotation.named_child(0))
                .and_then(|ty| SourceSpan::new(ty.start_byte(), ty.end_byte()).ok());
            result.push((declaration, type_reference));
        }
    }
    result
}

fn is_parameter_property(parameter: tree_sitter::Node<'_>) -> bool {
    parameter
        .children(&mut parameter.walk())
        .any(|child| matches!(child.kind(), "accessibility_modifier" | "readonly"))
}

#[cfg(test)]
#[path = "../tests/ir/typescript_this_property_binding.rs"]
mod tests;
