//! Canonical `focusMethods` resolution.
//!
//! Selectors are resolved against typed owners and canonical method IDs before
//! body filtering. Display names are input syntax, never rendering identity.

use super::compiler::CompiledIR;
use super::hierarchical::{HierarchicalIR, MethodNode};
use super::opcodes::CoreOp;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusResolutionError {
    OwnerNotFound(String),
    AmbiguousOwner(String),
    MethodNotFound(String),
    AmbiguousBareMethod(String),
}

impl fmt::Display for FocusResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OwnerNotFound(owner) => write!(f, "focus owner not found: {owner}"),
            Self::AmbiguousOwner(owner) => write!(f, "focus owner is ambiguous: {owner}"),
            Self::MethodNotFound(selector) => write!(f, "focus method not found: {selector}"),
            Self::AmbiguousBareMethod(method) => write!(
                f,
                "bare focus method is owned by multiple types; qualify it: {method}"
            ),
        }
    }
}

impl std::error::Error for FocusResolutionError {}

#[derive(Clone, Copy)]
struct Owner<'a> {
    name: &'a str,
    methods: &'a [MethodNode],
}

/// Resolve bare or `Owner.method` selectors to canonical method IDs.
///
/// A bare selector is valid only when one typed owner contains that method.
/// Every overload under the resolved owner is selected because the documented
/// selector grammar has no signature discriminator.
pub fn resolve_focus_method_ids(
    hierarchy: &HierarchicalIR,
    selectors: &HashSet<String>,
) -> Result<HashSet<String>, FocusResolutionError> {
    let owners = hierarchy
        .classes
        .iter()
        .map(|owner| Owner {
            name: &owner.name,
            methods: &owner.methods,
        })
        .chain(hierarchy.interfaces.iter().map(|owner| Owner {
            name: &owner.name,
            methods: &owner.methods,
        }))
        .collect::<Vec<_>>();
    let mut resolved = HashSet::new();

    for selector in selectors {
        if let Some((owner_name, method_name)) = selector.rsplit_once('.') {
            let matching_owners = owners
                .iter()
                .filter(|owner| owner.name == owner_name)
                .collect::<Vec<_>>();
            let owner = match matching_owners.as_slice() {
                [] => return Err(FocusResolutionError::OwnerNotFound(owner_name.to_string())),
                [owner] => owner,
                _ => return Err(FocusResolutionError::AmbiguousOwner(owner_name.to_string())),
            };
            let matches = owner
                .methods
                .iter()
                .filter(|method| method.name == method_name)
                .map(|method| method.id.clone())
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return Err(FocusResolutionError::MethodNotFound(selector.clone()));
            }
            resolved.extend(matches);
            continue;
        }

        let matching_owners = owners
            .iter()
            .filter(|owner| {
                owner
                    .methods
                    .iter()
                    .any(|method| method.name.as_str() == selector.as_str())
            })
            .collect::<Vec<_>>();
        let owner = match matching_owners.as_slice() {
            [] => return Err(FocusResolutionError::MethodNotFound(selector.clone())),
            [owner] => owner,
            _ => return Err(FocusResolutionError::AmbiguousBareMethod(selector.clone())),
        };
        resolved.extend(
            owner
                .methods
                .iter()
                .filter(|method| method.name.as_str() == selector.as_str())
                .map(|method| method.id.clone()),
        );
    }

    Ok(resolved)
}

/// Remove exact bodies outside the already-resolved canonical target set.
pub fn retain_focused_bodies(ir: &mut CompiledIR, method_ids: &HashSet<String>) {
    ir.instructions.retain(|op| match op {
        CoreOp::Body(method_id, ..) => method_ids.contains(method_id),
        _ => true,
    });
}

#[cfg(test)]
#[path = "../tests/ir/focus.rs"]
mod tests;
