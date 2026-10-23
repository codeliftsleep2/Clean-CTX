//! Angular interpretation of demanded native declaration/composition facets.
use super::ModuleProjection;
use crate::angular_meta::{class_role_authority as roles, decorators::ClassKind};
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{DeclarationId, Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

pub(super) fn project(
    authority: &TypeScriptAuthority,
    start: usize,
    end: usize,
) -> ModuleProjection {
    let mut result = ModuleProjection::default();
    if !authority.capture_current
        || authority.disposition.is_some()
        || !authority.witnesses.iter().all(|witness| witness.current())
    {
        return result;
    }
    let owners: Vec<_> = authority
        .declaration_facets
        .iter()
        .filter(|facet| {
            facet.declaration.source == authority.source
                && facet.declaration.start >= start
                && facet.declaration.end == end
                && matches!(
                    roles::declaration_role(authority, &facet.declaration),
                    Some(ClassKind::Module | ClassKind::Component)
                )
        })
        .collect();
    let [owner] = owners.as_slice() else {
        return result;
    };
    let owner_kind = roles::declaration_role(authority, &owner.declaration);
    let component = owner_kind == Some(ClassKind::Component);
    if component && super::standalone::eligible(authority, &owner.declaration) != Some(true) {
        return result;
    }
    let decorators: Vec<_> = owner
        .decorators
        .iter()
        .filter(|decorator| {
            matches!(&decorator.callee.value, ExpressionValue::Reference(resolution)
            if roles::decorator_role(authority, resolution) == owner_kind)
        })
        .collect();
    let [decorator] = decorators.as_slice() else {
        return result;
    };
    let [argument] = decorator.arguments.as_slice() else {
        return result;
    };
    let Some(fields) = object(authority, argument, &mut BTreeSet::new()) else {
        return result;
    };
    for (field, relation) in [
        ("declarations", SemanticRelation::DeclaresInModule),
        ("imports", SemanticRelation::ImportsModule),
        ("exports", SemanticRelation::ExportsFromModule),
    ] {
        if component && field != "imports" {
            continue;
        }
        let values: Vec<_> = fields.iter().filter(|(name, _)| name == field).collect();
        let [(_, value)] = values.as_slice() else {
            continue;
        };
        let mut leaves = BTreeSet::new();
        collect(
            authority,
            value,
            &mut BTreeSet::new(),
            &mut leaves,
            &mut result.unresolved_spreads,
        );
        for id in leaves {
            let kind = roles::declaration_role(authority, &id).or_else(|| {
                roles::public_api(authority, &id, "@angular/router", "RouterModule")
                    .then_some(ClassKind::Module)
            });
            if component
                && kind != Some(ClassKind::Module)
                && super::standalone::eligible(authority, &id) != Some(true)
            {
                continue;
            }
            let kind = match kind {
                Some(ClassKind::Component) => "Component",
                Some(ClassKind::Directive) => "Directive",
                Some(ClassKind::Pipe) => "Pipe",
                Some(ClassKind::Module) => "Module",
                _ => continue,
            };
            let mut subject = EntityRef::new(
                "angular",
                if component { "Component" } else { "Module" },
                &owner.declaration.name,
            );
            subject.file = Some(owner.declaration.source.to_string_lossy().into_owned());
            subject.canonical = Some(endpoint(
                &owner.declaration,
                Some((decorator.start, decorator.end)),
            ));
            let mut object = EntityRef::new("angular", kind, &id.name);
            object.file = Some(id.source.to_string_lossy().into_owned());
            object.canonical = Some(endpoint(&id, None));
            result.edges.push(SemanticEdge {
                relation,
                subject,
                object,
                layer: "angular",
                call_evidence: None,
            });
        }
    }
    result
}

pub(super) fn object<'a>(
    authority: &'a TypeScriptAuthority,
    expression: &'a ExpressionFacet,
    active: &mut BTreeSet<DeclarationId>,
) -> Option<&'a [(String, ExpressionFacet)]> {
    match &expression.value {
        ExpressionValue::Object(fields) => Some(fields),
        ExpressionValue::Reference(Resolution::Resolved(id)) if active.insert(id.clone()) => {
            let facet = authority
                .declaration_facets
                .iter()
                .find(|facet| facet.declaration == *id)?;
            object(authority, facet.initializer.as_ref()?, active)
        }
        _ => None,
    }
}

fn collect(
    authority: &TypeScriptAuthority,
    expression: &ExpressionFacet,
    active: &mut BTreeSet<DeclarationId>,
    leaves: &mut BTreeSet<DeclarationId>,
    unresolved: &mut Vec<String>,
) {
    match &expression.value {
        ExpressionValue::Array(items) => {
            for item in items {
                collect(authority, item, active, leaves, unresolved);
            }
        }
        ExpressionValue::Spread(value) => collect(authority, value, active, leaves, unresolved),
        ExpressionValue::Reference(Resolution::Resolved(id)) => {
            if !active.insert(id.clone()) {
                unresolved.push(format!("Cyclic composition: {}", id.name));
                return;
            }
            if let Some(initializer) = authority
                .declaration_facets
                .iter()
                .find(|facet| facet.declaration == *id)
                .and_then(|facet| facet.initializer.as_ref())
            {
                collect(authority, initializer, active, leaves, unresolved);
            } else {
                leaves.insert(id.clone());
            }
            active.remove(id);
        }
        // Preserve the established router module factory forms, using actual
        // RouterModule package/export identity rather than written import text.
        ExpressionValue::Call(function, _) => {
            if let ExpressionValue::Member(object, member) = &function.value
                && matches!(member.as_str(), "forRoot" | "forChild")
                && let ExpressionValue::Reference(Resolution::Resolved(id)) = &object.value
                && roles::public_api(authority, id, "@angular/router", "RouterModule")
            {
                leaves.insert(id.clone());
            }
        }
        _ => unresolved.push(format!(
            "Unproven composition site {}..{}",
            expression.start, expression.end
        )),
    }
}

fn endpoint(
    id: &DeclarationId,
    site: Option<(usize, usize)>,
) -> crate::layers::meta::canonical_endpoint::CanonicalEndpoint {
    crate::layers::meta::canonical_endpoint::CanonicalEndpoint::Declaration {
        declaration: id.into(),
        site,
    }
}
