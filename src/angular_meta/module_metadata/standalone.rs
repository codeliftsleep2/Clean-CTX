//! Angular eligibility from demanded metadata and the defining API package.
use super::canonical::object;
use crate::angular_meta::{class_role_authority as roles, decorators::ClassKind};
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{DeclarationId, Resolution, TypeScriptAuthority};
use std::collections::BTreeSet;

pub(super) fn eligible(authority: &TypeScriptAuthority, id: &DeclarationId) -> Option<bool> {
    let kind = roles::declaration_role(authority, id)?;
    if !matches!(
        kind,
        ClassKind::Component | ClassKind::Directive | ClassKind::Pipe
    ) {
        return None;
    }
    let facet = authority
        .declaration_facets
        .iter()
        .find(|facet| facet.declaration == *id)?;
    let decorators: Vec<_> = facet
        .decorators
        .iter()
        .filter(|decorator| {
            matches!(&decorator.callee.value, ExpressionValue::Reference(callee)
            if roles::decorator_role(authority, callee) == Some(kind))
        })
        .collect();
    let [decorator] = decorators.as_slice() else {
        return None;
    };
    let [argument] = decorator.arguments.as_slice() else {
        return None;
    };
    let fields = object(authority, argument, &mut BTreeSet::new())?;
    // An unexpanded spread/computed field cannot prove absence or final value.
    if fields.iter().any(|(name, _)| name.is_empty()) {
        return None;
    }
    let flags: Vec<_> = fields
        .iter()
        .filter(|(name, _)| name == "standalone")
        .collect();
    match flags.as_slice() {
        [(_, value)] => boolean(authority, value, &mut BTreeSet::new()),
        [] => {
            let ExpressionValue::Reference(Resolution::Resolved(api)) = &decorator.callee.value
            else {
                return None;
            };
            let versions: BTreeSet<_> = authority
                .package_exports
                .iter()
                .filter(|package| {
                    package.declaration == *api
                        && roles::public_api(authority, api, "@angular/core", &api.name)
                })
                .filter_map(|package| package.version.as_deref())
                .collect();
            if versions.len() != 1 {
                return None;
            }
            stable_major(versions.first().copied()?).map(|major| major >= 19)
        }
        _ => None,
    }
}

fn boolean(
    authority: &TypeScriptAuthority,
    value: &ExpressionFacet,
    active: &mut BTreeSet<DeclarationId>,
) -> Option<bool> {
    match &value.value {
        ExpressionValue::Boolean(value) => Some(*value),
        ExpressionValue::Reference(Resolution::Resolved(id)) if active.insert(id.clone()) => {
            let initializer = authority
                .declaration_facets
                .iter()
                .find(|facet| facet.declaration == *id)?
                .initializer
                .as_ref()?;
            boolean(authority, initializer, active)
        }
        _ => None,
    }
}

fn stable_major(version: &str) -> Option<u32> {
    let parts: Vec<_> = version.split('.').collect();
    let [major, minor, patch] = parts.as_slice() else {
        return None;
    };
    let mut numbers = Vec::new();
    for part in [major, minor, patch] {
        let number = part.parse::<u32>().ok()?;
        if number.to_string() != *part {
            return None;
        }
        numbers.push(number);
    }
    Some(numbers[0])
}
