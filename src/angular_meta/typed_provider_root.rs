//! Shared exact native annotation/const-initializer proof for typed provider roots.
use crate::ir::typescript_resolution::declaration_facets::{DeclarationFacet, ExpressionValue};
use crate::ir::typescript_resolution::{Namespace, Resolution, TypeScriptAuthority};

pub(super) fn admits(
    authority: &TypeScriptAuthority,
    facet: &DeclarationFacet,
    package: &str,
    export: &str,
) -> bool {
    let id = &facet.declaration;
    if id.source != authority.source
        || id.source_hash != authority.source_hash
        || id.kind != "variable_declarator"
        || authority
            .declaration_facets
            .iter()
            .filter(|f| f.declaration == *id)
            .count()
            != 1
        || !authority.declarations.contains(id)
    {
        return false;
    }
    let (Some(annotation), Some(initializer)) = (&facet.type_annotation, &facet.initializer) else {
        return false;
    };
    if !(id.start <= annotation.start
        && annotation.start < annotation.end
        && annotation.end <= initializer.start
        && initializer.start < initializer.end
        && initializer.end <= id.end)
    {
        return false;
    }
    let ExpressionValue::Reference(Resolution::Resolved(target)) = &annotation.value else {
        return false;
    };
    let mut sites = authority
        .type_reference_sites
        .iter()
        .filter(|(s, e, _)| *s == annotation.start && *e == annotation.end);
    let Some((_, _, Resolution::Resolved(actual))) = sites.next() else {
        return false;
    };
    actual == target
        && sites.next().is_none()
        && super::class_role_authority::public_entry_api(
            authority,
            target,
            package,
            package,
            export,
            Namespace::Type,
        )
        && authority.admits_demanded_structure(
            &authority.source,
            &authority.source_hash,
            initializer,
        )
}
