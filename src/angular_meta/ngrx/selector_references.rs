//! Exact same-file Selector reference-to-declaration composition.

use super::{NgRxShape, selector_declarations::SelectorDeclarationEvidence};
use crate::ir::invocation_evidence::{ArgumentQualifier, InvocationEvidenceView, WrittenCallee};

/// Retain only select occurrences whose one direct identifier argument binds
/// to the exact declaration independently classified as an NgRx Selector.
pub(crate) fn admit(
    mut shape: NgRxShape,
    evidence: &InvocationEvidenceView<'_>,
    declarations: &[SelectorDeclarationEvidence],
    source_id: &str,
    source: &str,
) -> NgRxShape {
    shape.select_sites.retain_mut(|site| {
        let Some(call) = evidence.occurrences().iter().find(|call| {
            call.span().start_byte() == site.call_start
                && matches!(call.callee(), WrittenCallee::Member { terminal, .. }
                    if evidence.slice(*terminal).ok() == Some("select"))
        }) else {
            return false;
        };
        let [argument] = call.arguments() else {
            return false;
        };
        if argument.qualifier() != &ArgumentQualifier::Plain {
            return false;
        }
        let Some(binding) = crate::ir::typescript_identifier_binding::bind_direct_value_identifier(
            source_id,
            source,
            argument.span(),
        ) else {
            // Unsupported/unbound forms retain the established occurrence
            // product. They still acquire no declaration authority.
            return true;
        };
        let Some(declaration) = declarations.iter().find(|declaration| {
            declaration.file == binding.source_id()
                && declaration.source_hash == binding.source_hash()
                && declaration.name_span == binding.declaration_span()
        }) else {
            return false;
        };
        site.selector_name.clone_from(&declaration.name);
        true
    });
    shape
}
