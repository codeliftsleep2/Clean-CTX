//! Acquisition obligations derived from the canonical class inputs and resolver.
use super::*;
use crate::ir::csharp_symbols::CSharpSymbolComposition;
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Default)]
pub(crate) struct AncestryObligations {
    pub(crate) discovery_terms: BTreeSet<String>,
    pub(crate) owners: BTreeSet<String>,
    pub(crate) reasons: BTreeSet<&'static str>,
    pub(crate) requirements: Vec<CSharpAncestryObligation>,
}

/// Original canonical identity/scope and the current resolver disposition.
/// Acquisition attaches the existing production partition; names extracted
/// from this value are only candidate hints.
pub(crate) struct CSharpAncestryObligation {
    owner: CSharpSymbolKey,
    file: String,
    span: CSharpSourceSpan,
    written_base: String,
    scope: Scope,
    disposition: CSharpResolution<CSharpTypeIdentity>,
}

impl CSharpAncestryObligation {
    pub(crate) fn identity_material(&self) -> String {
        format!(
            "{:?}",
            (
                &self.owner,
                &self.file,
                self.span,
                &self.written_base,
                &self.scope,
                &self.disposition
            )
        )
    }
}

/// The file selects owners, never a bare type name. Every subsequent hop is
/// selected by the existing resolver with its original scope and symbol key.
pub(crate) fn ancestry_obligations(
    units: &[CSharpCompilationUnit],
    seed: &str,
    known_external: &[CSharpTypeIdentity],
) -> AncestryObligations {
    let index = CSharpSourceSymbolIndex::build(
        units
            .iter()
            .flat_map(|unit| unit.classes.iter().map(|input| input.part.clone())),
    );
    collect_obligations(units, seed, known_external, &index)
}

pub(crate) fn ancestry_obligations_for_files(
    units: &[CSharpCompilationUnit],
    files: &[&str],
    external: &[CSharpTypeIdentity],
) -> BTreeMap<String, AncestryObligations> {
    let index = CSharpSourceSymbolIndex::build(
        units
            .iter()
            .flat_map(|unit| unit.classes.iter().map(|input| input.part.clone())),
    );
    files
        .iter()
        .map(|file| {
            (
                (*file).to_string(),
                collect_obligations(units, file, external, &index),
            )
        })
        .collect()
}

fn collect_obligations(
    units: &[CSharpCompilationUnit],
    seed: &str,
    known_external: &[CSharpTypeIdentity],
    index: &CSharpSourceSymbolIndex,
) -> AncestryObligations {
    let mut result = AncestryObligations::default();
    let seeds: BTreeSet<_> = units
        .iter()
        .flat_map(|unit| &unit.classes)
        .filter(|input| input.part.file == seed)
        .map(|input| input.part.key.clone())
        .collect();
    for key in seeds {
        visit(
            &key,
            units,
            index,
            known_external,
            &mut HashSet::new(),
            &mut result,
        );
    }
    result
}

fn visit(
    key: &CSharpSymbolKey,
    units: &[CSharpCompilationUnit],
    index: &CSharpSourceSymbolIndex,
    external: &[CSharpTypeIdentity],
    active: &mut HashSet<CSharpSymbolKey>,
    result: &mut AncestryObligations,
) {
    if !active.insert(key.clone()) {
        result.reasons.insert("dependency_cycle");
        return;
    }
    // Establish the complete declaration family, including duplicate non-partial
    // declarations. Otherwise a later same-name admission could retract truth.
    result.discovery_terms.insert(key.name.clone());
    let Some(CSharpSymbolComposition::Resolved(symbol)) = index.get(key) else {
        result.reasons.insert("dependency_ambiguous");
        active.remove(key);
        return;
    };
    for part in &symbol.parts {
        result.owners.insert(part.file.clone());
    }
    for input in units
        .iter()
        .flat_map(|unit| &unit.classes)
        .filter(|input| input.part.key == *key)
    {
        for attribute in input
            .attributes
            .iter()
            .chain(input.methods.iter().flat_map(|method| &method.attributes))
        {
            hint(&attribute.written, &mut result.discovery_terms);
            hint(
                &format!("{}Attribute", attribute.written),
                &mut result.discovery_terms,
            );
            for alias in &input.scope.aliases.directives {
                hint(&alias.target, &mut result.discovery_terms);
            }
            let resolved = resolve_attribute(attribute, input, external, index);
            let potential = resolve_attribute(
                attribute,
                input,
                external,
                &CSharpSourceSymbolIndex::default(),
            );
            if matches!(potential.resolution, CSharpResolution::Resolved { .. }) {
                match &resolved.resolution {
                    CSharpResolution::Ambiguous(_) => {
                        result.reasons.insert("dependency_ambiguous");
                    }
                    CSharpResolution::Unresolved(_) => {
                        result.reasons.insert("dependency_unresolved");
                    }
                    CSharpResolution::Unsupported(_) => {
                        result.reasons.insert("dependency_unsupported");
                    }
                    CSharpResolution::Resolved { .. } => {}
                }
            }
            result.requirements.push(CSharpAncestryObligation {
                owner: key.clone(),
                file: input.part.file.clone(),
                span: attribute.span,
                written_base: attribute.written.clone(),
                scope: input.scope.clone(),
                disposition: resolved.resolution,
            });
        }
    }
    if let Some(written) = &symbol.base_reference {
        let input = units
            .iter()
            .flat_map(|unit| &unit.classes)
            .find(|input| input.part.key == *key && input.part.base_references.contains(written));
        if let Some(input) = input {
            // These are conservative discovery hints, never resolution evidence.
            hint(written, &mut result.discovery_terms);
            for alias in &input.scope.aliases.directives {
                hint(&alias.target, &mut result.discovery_terms);
            }
            let disposition = resolve_direct_base(written, &input.scope, external, index);
            result.requirements.push(CSharpAncestryObligation {
                owner: key.clone(),
                file: input.part.file.clone(),
                span: input.part.span,
                written_base: written.clone(),
                scope: input.scope.clone(),
                disposition: disposition.clone(),
            });
            match disposition {
                CSharpResolution::Resolved { identity, .. }
                    if identity.origin
                        == super::super::CSharpTypeIdentityOrigin::AdmittedSource =>
                {
                    let next = CSharpSymbolKey {
                        namespace_path: identity.namespace_path,
                        containing_type_path: identity.containing_type_path,
                        name: identity.terminal_name,
                        generic_arity: identity.generic_arity,
                    };
                    visit(&next, units, index, external, active, result);
                }
                CSharpResolution::Resolved { .. } => {}
                CSharpResolution::Ambiguous(_) => {
                    result.reasons.insert("dependency_ambiguous");
                }
                CSharpResolution::Unsupported(_) => {
                    result.reasons.insert("dependency_unsupported");
                }
                CSharpResolution::Unresolved(_) => {
                    result.reasons.insert("dependency_unresolved");
                }
            }
        } else {
            result.reasons.insert("dependency_unresolved");
        }
    }
    active.remove(key);
}

fn hint(written: &str, terms: &mut BTreeSet<String>) {
    let head = written.split('<').next().unwrap_or(written).trim();
    let terminal = head
        .rsplit(['.', ':'])
        .next()
        .unwrap_or(head)
        .trim_start_matches('@');
    if !terminal.is_empty() {
        terms.insert(terminal.to_string());
    }
}
