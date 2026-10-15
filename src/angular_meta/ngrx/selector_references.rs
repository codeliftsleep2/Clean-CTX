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
        let Some(member) = crate::ir::typescript_this_property_binding::bind_this_property(
            source_id, source, receiver,
        ) else {
            continue;
        };
        if member.source_id() != source_id || member.source_hash() != source_hash {
            continue;
        }
        let Some(type_reference) = member.type_reference_span() else {
            continue;
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
            service_owner: super::service_owners::authoritative_service_owner(
                call, classes, source,
            )
            .map(|owner| owner.service),
            module_specifier,
        });
    }
    shape
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
