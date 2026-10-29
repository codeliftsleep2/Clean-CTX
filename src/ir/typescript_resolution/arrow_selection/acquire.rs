//! Structural acquisition using existing lexical binding, module and export resolvers.
use super::{ArrowSelection, Span};
use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_resolution::demanded_occurrences::{DemandedOccurrence, DemandedValue};
use crate::ir::typescript_resolution::{Namespace, Resolution, Resolver};
use std::{collections::BTreeSet, path::Path};
fn span(node: tree_sitter::Node<'_>) -> Span {
    (node.start_byte(), node.end_byte())
}
fn ordinary(node: tree_sitter::Node<'_>) -> bool {
    let mut c = node.walk();
    !node
        .children(&mut c)
        .any(|n| matches!(n.kind(), "async" | "optional_chain"))
        && !node.has_error()
}
fn args(node: tree_sitter::Node<'_>) -> Vec<tree_sitter::Node<'_>> {
    let mut c = node.walk();
    node.named_children(&mut c)
        .filter(|n| n.kind() != "comment")
        .collect()
}
pub(in crate::ir::typescript_resolution) fn capture(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    outer: tree_sitter::Node<'_>,
) -> Option<(ArrowSelection, Vec<DemandedOccurrence>)> {
    if outer.kind() != "arrow_function" || !ordinary(outer) {
        return None;
    }
    let parameters = outer.child_by_field_name("parameters")?;
    if !args(parameters).is_empty() {
        return None;
    }
    let returned = outer.child_by_field_name("body")?;
    if returned.kind() != "call_expression" || !ordinary(returned) {
        return None;
    }
    let access = returned.child_by_field_name("function")?;
    if access.kind() != "member_expression" || !ordinary(access) {
        return None;
    }
    let then_property = access.child_by_field_name("property")?;
    if then_property.kind() != "property_identifier"
        || then_property.utf8_text(source.as_bytes()).ok()? != "then"
    {
        return None;
    }
    let import = access.child_by_field_name("object")?;
    if import.kind() != "call_expression" || !ordinary(import) {
        return None;
    }
    let keyword = import.child_by_field_name("function")?;
    if keyword.kind() != "import" {
        return None;
    }
    let arguments = args(import.child_by_field_name("arguments")?);
    if arguments.len() != 1 {
        return None;
    }
    let literal = arguments[0];
    if literal.kind() != "string" {
        return None;
    }
    let specifier = crate::ir::typescript_resolution::string_value::decode(
        literal.utf8_text(source.as_bytes()).ok()?,
    )?;
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return None;
    }
    let arguments = args(returned.child_by_field_name("arguments")?);
    if arguments.len() != 1 {
        return None;
    }
    let selector = arguments[0];
    if selector.kind() != "arrow_function" || !ordinary(selector) {
        return None;
    }
    let parameter = if let Some(n) = selector.child_by_field_name("parameter") {
        n
    } else {
        let nodes = args(selector.child_by_field_name("parameters")?);
        if nodes.len() != 1 {
            return None;
        }
        let n = nodes[0];
        if n.kind() == "required_parameter" {
            let nodes = args(n);
            if nodes.len() != 1 {
                return None;
            }
            nodes[0]
        } else {
            n
        }
    };
    if parameter.kind() != "identifier" {
        return None;
    }
    let member = selector.child_by_field_name("body")?;
    if member.kind() != "member_expression" || !ordinary(member) {
        return None;
    }
    let receiver = member.child_by_field_name("object")?;
    let property = member.child_by_field_name("property")?;
    if receiver.kind() != "identifier" || property.kind() != "property_identifier" {
        return None;
    }
    let root = resolver.source_tree(file)?;
    let binding = crate::ir::typescript_identifier_binding::lookup_value_identifier_in_tree(
        &file.to_string_lossy(),
        source,
        SourceSpan::new(receiver.start_byte(), receiver.end_byte()).ok()?,
        root.root_node(),
    )
    .ok()??;
    if binding.declaration_kind()
        != crate::ir::typescript_identifier_binding::ValueDeclarationKind::ArrowParameter
        || (
            binding.declaration_span().start_byte(),
            binding.declaration_span().end_byte(),
        ) != span(parameter)
    {
        return None;
    }
    let module = resolver
        .module_request(
            file,
            &specifier,
            crate::ir::typescript_resolution::module_resolution::ModuleRequestKind::Import,
        )
        .ok()?;
    let export = property.utf8_text(source.as_bytes()).ok()?.to_string();
    let (targets, complete) =
        resolver.export_set(&module, &export, Namespace::Value, &mut BTreeSet::new());
    if !complete || targets.len() != 1 {
        return None;
    }
    let target = targets.into_iter().next()?;
    let proof = ArrowSelection {
        source: file.to_path_buf(),
        source_hash: crate::cache::content_hash(source.as_bytes()),
        outer: span(outer),
        parameters: span(parameters),
        returned: span(returned),
        access: span(access),
        import: span(import),
        import_keyword: span(keyword),
        literal: span(literal),
        then_property: span(then_property),
        selector: span(selector),
        parameter: span(parameter),
        member: span(member),
        receiver: span(receiver),
        property: span(property),
        module,
        specifier,
        export,
        target,
    };
    let mut occurrences = Vec::new();
    let mut put = |range: Span, value| {
        occurrences.push(DemandedOccurrence {
            source: proof.source.clone(),
            source_hash: proof.source_hash.clone(),
            start: range.0,
            end: range.1,
            value,
        })
    };
    for (range, kind, children) in proof.structures() {
        put(
            range,
            DemandedValue::Structure {
                kind: kind.into(),
                children,
            },
        );
    }
    put(
        proof.parameters,
        DemandedValue::Literal {
            kind: "zero-parameters".into(),
            text: "()".into(),
        },
    );
    put(
        proof.then_property,
        DemandedValue::Literal {
            kind: "property_identifier".into(),
            text: "then".into(),
        },
    );
    put(
        proof.literal,
        DemandedValue::Literal {
            kind: "string".into(),
            text: literal.utf8_text(source.as_bytes()).ok()?.into(),
        },
    );
    put(
        proof.parameter,
        DemandedValue::Literal {
            kind: "selector-parameter".into(),
            text: parameter.utf8_text(source.as_bytes()).ok()?.into(),
        },
    );
    put(
        proof.import_keyword,
        DemandedValue::Literal {
            kind: "import".into(),
            text: "import".into(),
        },
    );
    put(
        proof.property,
        DemandedValue::Reference(Resolution::Resolved(proof.target.clone())),
    );
    Some((proof, occurrences))
}
