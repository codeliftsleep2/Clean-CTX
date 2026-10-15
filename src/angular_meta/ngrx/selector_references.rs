//! Exact same-file Selector reference-to-declaration composition.

use super::{NgRxShape, selector_declarations::SelectorDeclarationEvidence};
use crate::angular_meta::inject_evidence::ClassInitializerEvidence;
use crate::ir::invocation_evidence::{
    ArgumentQualifier, InvocationEvidenceView, ParentSite, SourceSpan, WrittenCallee,
};

/// Project direct `store.select(selector)` and `store.pipe(select(selector))`
/// invocations whose receiver is an exact NgRx Store binding and whose direct
/// identifier argument is an exact Selector.
pub(crate) fn admit(
    mut shape: NgRxShape,
    evidence: &InvocationEvidenceView<'_>,
    declarations: &[SelectorDeclarationEvidence],
    classes: &[ClassInitializerEvidence],
    source_id: &str,
    source: &str,
) -> NgRxShape {
    let source_hash = crate::cache::content_hash(source.as_bytes());
    shape.select_sites.clear();
    for call in evidence.occurrences() {
        let Some(receiver) = store_receiver(call, evidence, source_id, source) else {
            continue;
        };
        let [argument] = call.arguments() else {
            continue;
        };
        if argument.qualifier() != &ArgumentQualifier::Plain {
            continue;
        }
        let (type_reference, service_owner) = if let Some(member) =
            crate::ir::typescript_this_property_binding::bind_this_property(
                source_id, source, receiver,
            ) {
            if member.source_id() != source_id || member.source_hash() != source_hash {
                continue;
            }
            let Some(type_reference) = member.type_reference_span() else {
                continue;
            };
            (
                type_reference,
                super::service_owners::authoritative_service_owner(call, classes, source)
                    .map(|owner| owner.service),
            )
        } else {
            let Some(authority) = initializer_store_authority(
                call,
                receiver,
                classes,
                shape.component_name.as_deref(),
                source,
            ) else {
                continue;
            };
            (authority.type_reference, authority.service_owner)
        };
        let Some(import) =
            crate::ir::typescript_type_import_binding::bind_type_reference_to_named_import(
                source_id,
                source,
                type_reference,
            )
        else {
            continue;
        };
        if import.source_id() != source_id
            || import.source_hash() != source_hash
            || import.imported_symbol_name() != "Store"
            || import.module_specifier() != "@ngrx/store"
        {
            continue;
        }
        let (selector_name, module_specifier) = if let Some(binding) =
            crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
                source_id,
                source,
                argument.span(),
            ) {
            let Some(declaration) = declarations.iter().find(|declaration| {
                declaration.file == binding.source_id()
                    && declaration.source_hash == binding.source_hash()
                    && declaration.name_span == binding.declaration_span()
            }) else {
                continue;
            };
            (declaration.name.clone(), None)
        } else {
            let Some(import) =
                crate::ir::typescript_value_import_binding::bind_direct_named_value_import(
                    source_id,
                    source,
                    argument.span(),
                )
            else {
                continue;
            };
            if import.source_id != source_id || import.source_hash != source_hash {
                continue;
            }
            (import.imported_name, Some(import.module_specifier))
        };
        shape.select_sites.push(super::SelectSite {
            selector_name,
            service_owner,
            module_specifier,
        });
    }
    shape
}

pub(crate) struct InitializerStoreAuthority {
    pub(crate) type_reference: SourceSpan,
    pub(crate) service_owner: Option<String>,
}

pub(crate) fn initializer_store_authority(
    call: &crate::ir::invocation_evidence::InvocationOccurrence,
    receiver_span: SourceSpan,
    classes: &[ClassInitializerEvidence],
    component_owner: Option<&str>,
    source: &str,
) -> Option<InitializerStoreAuthority> {
    use crate::ir::invocation_evidence::DeclarationContextKind;

    let context = call.declaration_context()?;
    if !matches!(
        context.kind(),
        DeclarationContextKind::FieldInitializer | DeclarationContextKind::PropertyInitializer
    ) {
        return None;
    }
    let language = crate::compression::language::safe_typescript_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(source, None)?;
    if tree.root_node().has_error() {
        return None;
    }
    let receiver = tree
        .root_node()
        .named_descendant_for_byte_range(receiver_span.start_byte(), receiver_span.end_byte())?;
    if receiver.kind() != "member_expression"
        || receiver.start_byte() != receiver_span.start_byte()
        || receiver.end_byte() != receiver_span.end_byte()
    {
        return None;
    }
    let object = receiver.child_by_field_name("object")?;
    let property = receiver.child_by_field_name("property")?;
    if object.kind() != "this"
        || property.kind() != "property_identifier"
        || receiver
            .children(&mut receiver.walk())
            .any(|child| matches!(child.kind(), "optional_chain" | "?."))
    {
        return None;
    }
    let declaration = context.declaration();
    let field = tree
        .root_node()
        .named_descendant_for_byte_range(declaration.start_byte(), declaration.end_byte())?;
    if field.kind() != "public_field_definition"
        || field.start_byte() != declaration.start_byte()
        || field.end_byte() != declaration.end_byte()
        || field
            .children(&mut field.walk())
            .any(|child| child.kind() == "static")
    {
        return None;
    }
    let value = field.child_by_field_name("value")?;
    if receiver.start_byte() < value.start_byte() || receiver.end_byte() > value.end_byte() {
        return None;
    }
    let mut ancestor = receiver.parent()?;
    while ancestor != field {
        if matches!(
            ancestor.kind(),
            "function_declaration"
                | "function_expression"
                | "generator_function_declaration"
                | "generator_function"
        ) {
            return None;
        }
        ancestor = ancestor.parent()?;
    }
    let body = field.parent().filter(|node| node.kind() == "class_body")?;
    let class = body
        .parent()
        .filter(|node| node.kind() == "class_declaration")?;
    let class_span = SourceSpan::new(class.start_byte(), class.end_byte()).ok()?;
    let class_evidence = classes.iter().find(|item| item.class_span == class_span)?;
    let (_, kind, _, _, _) =
        crate::angular_meta::decorators::extract_graph_entries(&class_evidence.raw_class)?;
    let service_owner = match kind {
        crate::angular_meta::decorators::ClassKind::Service => {
            Some(class_evidence.consumer.clone())
        }
        crate::angular_meta::decorators::ClassKind::Component
            if component_owner == Some(class_evidence.consumer.as_str()) =>
        {
            None
        }
        _ => return None,
    };
    let property_name = property.utf8_text(source.as_bytes()).ok()?;
    let candidates = parameter_properties(class, property_name, source.as_bytes());
    let [(_declaration_name, type_reference)] = candidates.as_slice() else {
        return None;
    };
    Some(InitializerStoreAuthority {
        type_reference: (*type_reference)?,
        service_owner,
    })
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
    let mut cursor = body.walk();
    for constructor in body
        .named_children(&mut cursor)
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
            if !parameter
                .children(&mut parameter.walk())
                .any(|child| matches!(child.kind(), "accessibility_modifier" | "readonly"))
            {
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

fn store_receiver(
    call: &crate::ir::invocation_evidence::InvocationOccurrence,
    evidence: &InvocationEvidenceView<'_>,
    source_id: &str,
    source: &str,
) -> Option<SourceSpan> {
    match call.callee() {
        WrittenCallee::Member { receiver, terminal }
            if evidence.slice(*terminal).ok() == Some("select") =>
        {
            Some(*receiver)
        }
        WrittenCallee::Free { terminal } if evidence.slice(*terminal).ok() == Some("select") => {
            if !exact_ngrx_select_import(source_id, source, *terminal) {
                return None;
            }
            let parent = call.parent()?;
            if parent.site() != (ParentSite::Argument { index: 0 }) {
                return None;
            }
            let outer = evidence
                .occurrences()
                .iter()
                .find(|candidate| candidate.span() == parent.invocation())?;
            let WrittenCallee::Member { receiver, terminal } = outer.callee() else {
                return None;
            };
            (evidence.slice(*terminal).ok() == Some("pipe")).then_some(*receiver)
        }
        _ => None,
    }
}

fn exact_ngrx_select_import(source_id: &str, source: &str, reference_span: SourceSpan) -> bool {
    if crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
        source_id,
        source,
        reference_span,
    )
    .is_some()
    {
        return false;
    }
    let Some(language) = crate::compression::language::safe_typescript_language() else {
        return false;
    };
    let mut parser = tree_sitter::Parser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    let Some(tree) = parser.parse(source, None) else {
        return false;
    };
    let mut matches = 0;
    collect_ngrx_select_imports(tree.root_node(), source.as_bytes(), &mut matches);
    matches == 1
}

fn collect_ngrx_select_imports(node: tree_sitter::Node<'_>, source: &[u8], matches: &mut usize) {
    if node.kind() == "import_statement"
        && node
            .child_by_field_name("source")
            .and_then(|string| string.named_child(0))
            .and_then(|fragment| fragment.utf8_text(source).ok())
            == Some("@ngrx/store")
        && has_unaliased_select_specifier(node, source)
    {
        *matches += 1;
        return;
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_ngrx_select_imports(child, source, matches);
    }
}

fn has_unaliased_select_specifier(node: tree_sitter::Node<'_>, source: &[u8]) -> bool {
    if node.kind() == "import_specifier" {
        return node.child_by_field_name("alias").is_none()
            && node
                .child_by_field_name("name")
                .and_then(|name| name.utf8_text(source).ok())
                == Some("select");
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .any(|child| has_unaliased_select_specifier(child, source))
}
