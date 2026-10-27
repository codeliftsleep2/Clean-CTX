//! Declared inline membership on a proved bootstrap invocation; no effective providers.
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::{
    ExpressionFacet, ExpressionValue, InvocationFacet,
};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(super) fn project_call(
    authority: &TypeScriptAuthority,
    call: &InvocationFacet,
    args: &[ExpressionFacet],
    subject: EntityRef,
) -> Vec<SemanticEdge> {
    if args.len() < 2
        || args[..2]
            .iter()
            .any(|arg| matches!(arg.value, ExpressionValue::Spread(_)))
    {
        return Vec::new();
    }
    let object = &args[1];
    let ExpressionValue::Object(fields) = &object.value else {
        return Vec::new();
    };
    if !authority.admits_demanded_structure(&call.source, &call.source_hash, object)
        || fields.iter().any(|(key, _)| key.is_empty())
    {
        return Vec::new();
    }
    let mut providers = fields.iter().filter(|(key, _)| key == "providers");
    let Some((_, value)) = providers.next() else {
        return Vec::new();
    };
    if providers.next().is_some()
        || !(object.start <= value.start && value.start < value.end && value.end <= object.end)
    {
        return Vec::new();
    }
    super::provider_imported::project_entries_at(
        authority,
        (&call.source, &call.source_hash),
        subject,
        value,
        false,
        SemanticRelation::RegistersProvider,
        true,
    )
}
