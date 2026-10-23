//! Exact constructor decorator syntax; Angular alone interprets API roles.
use super::declaration_facets::{DecoratorFacet, ExpressionFacet, ExpressionValue};
use super::{Resolution, Resolver};
use crate::ir::invocation_evidence::SourceSpan;
use std::path::Path;

pub(super) fn is_constructor_decorator_site(mut node: tree_sitter::Node<'_>, source: &str) -> bool {
    loop {
        if node.kind() == "decorator" {
            return node
                .parent()
                .filter(|p| matches!(p.kind(), "required_parameter" | "optional_parameter"))
                .and_then(|p| p.parent())
                .filter(|p| p.kind() == "formal_parameters")
                .and_then(|p| p.parent())
                .filter(|p| p.kind() == "method_definition")
                .and_then(|p| p.child_by_field_name("name"))
                .is_some_and(|name| name.utf8_text(source.as_bytes()).ok() == Some("constructor"));
        }
        if matches!(node.kind(), "formal_parameters" | "class_body" | "program") {
            return false;
        }
        let Some(parent) = node.parent() else {
            return false;
        };
        node = parent;
    }
}

pub(super) fn capture(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    parameter: tree_sitter::Node<'_>,
) -> Vec<DecoratorFacet> {
    let mut cursor = parameter.walk();
    parameter
        .named_children(&mut cursor)
        .filter(|node| node.kind() == "decorator")
        .map(|decorator| {
            let mut cursor = decorator.walk();
            let expression = decorator
                .named_children(&mut cursor)
                .find(|n| n.kind() != "comment");
            let call = expression.filter(|node| node.kind() == "call_expression");
            let callee = call
                .and_then(|call| call.child_by_field_name("function"))
                .or(expression);
            let callee = callee
                .map(|node| reference(resolver, file, source, node))
                .unwrap_or(ExpressionFacet {
                    start: decorator.start_byte(),
                    end: decorator.end_byte(),
                    value: ExpressionValue::Unsupported("missing decorator callee".into()),
                });
            let arguments = call
                .and_then(|call| call.child_by_field_name("arguments"))
                .map(|args| {
                    let mut cursor = args.walk();
                    args.named_children(&mut cursor)
                        .filter(|node| node.kind() != "comment")
                        .map(|node| reference(resolver, file, source, node))
                        .collect()
                })
                .unwrap_or_default();
            DecoratorFacet {
                start: decorator.start_byte(),
                end: decorator.end_byte(),
                callee,
                arguments,
            }
        })
        .collect()
}

fn reference(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    node: tree_sitter::Node<'_>,
) -> ExpressionFacet {
    let value = if matches!(node.kind(), "identifier" | "member_expression") {
        let resolution = SourceSpan::new(node.start_byte(), node.end_byte())
            .ok()
            .and_then(|span| resolver.value_reference(file, source, span))
            .unwrap_or_else(|| {
                Resolution::Unresolved("Constructor decorator reference unsupported".into())
            });
        ExpressionValue::Reference(resolution)
    } else if node.kind() == "string" && !node.has_error() {
        node.utf8_text(source.as_bytes())
            .ok()
            .and_then(super::string_literal::decode)
            .map(ExpressionValue::StringValue)
            .unwrap_or_else(|| ExpressionValue::Unsupported("unproved string literal".into()))
    } else {
        ExpressionValue::Unsupported(node.kind().into())
    };
    ExpressionFacet {
        start: node.start_byte(),
        end: node.end_byte(),
        value,
    }
}
