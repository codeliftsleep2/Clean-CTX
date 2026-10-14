//! Exact same-file Selector reference-to-declaration composition.

use super::{NgRxShape, selector_declarations::SelectorDeclarationEvidence};
use crate::angular_meta::inject_evidence::ClassInitializerEvidence;
use crate::ir::invocation_evidence::{ArgumentQualifier, InvocationEvidenceView, WrittenCallee};

/// Project only direct member `select` invocations whose receiver is an exact
/// NgRx Store binding and whose direct identifier argument is an exact Selector.
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
        let WrittenCallee::Member { receiver, terminal } = call.callee() else {
            continue;
        };
        if evidence.slice(*terminal).ok() != Some("select") {
            continue;
        }
        let [argument] = call.arguments() else {
            continue;
        };
        if argument.qualifier() != &ArgumentQualifier::Plain {
            continue;
        }
        let Some(member) = crate::ir::typescript_this_property_binding::bind_this_property(
            source_id, source, *receiver,
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
        let Some(binding) = crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
            source_id,
            source,
            argument.span(),
        ) else {
            continue;
        };
        let Some(declaration) = declarations.iter().find(|declaration| {
            declaration.file == binding.source_id()
                && declaration.source_hash == binding.source_hash()
                && declaration.name_span == binding.declaration_span()
        }) else {
            continue;
        };
        shape.select_sites.push(super::SelectSite {
            selector_name: declaration.name.clone(),
            service_owner: super::service_owners::authoritative_service_owner(
                call, classes, source,
            )
            .map(|owner| owner.service),
        });
    }
    shape
}
