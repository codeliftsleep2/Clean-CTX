//! Lexically scoped C# namespace and type alias resolution.

use super::{
    CSharpResolution, CSharpResolutionBasis, CSharpResolutionEvidence, CSharpResolutionReason,
    CSharpSourceSpan, CSharpTypeIdentity, resolve_explicit_type_reference,
};
use crate::ir::csharp_symbols::CSharpSourceSymbolIndex;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpAliasDirective {
    pub alias: String,
    pub target: String,
    pub scope_depth: usize,
    pub file: String,
    pub span: CSharpSourceSpan,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CSharpAliasContext {
    pub directives: Vec<CSharpAliasDirective>,
}

pub fn resolve_alias_type_reference(
    written: &str,
    aliases: &CSharpAliasContext,
    known_external: &[CSharpTypeIdentity],
    source_symbols: &CSharpSourceSymbolIndex,
) -> CSharpResolution<CSharpTypeIdentity> {
    let basis = if alias_suffix(written).is_some() {
        CSharpResolutionBasis::NamespaceAlias
    } else {
        CSharpResolutionBasis::TypeAlias
    };
    resolve_alias(
        written.trim(),
        aliases,
        known_external,
        source_symbols,
        basis,
        &mut HashSet::new(),
    )
}

fn resolve_alias(
    written: &str,
    aliases: &CSharpAliasContext,
    known_external: &[CSharpTypeIdentity],
    source_symbols: &CSharpSourceSymbolIndex,
    basis: CSharpResolutionBasis,
    visited: &mut HashSet<String>,
) -> CSharpResolution<CSharpTypeIdentity> {
    let (head, suffix) = alias_head_and_suffix(written);
    let matching = aliases
        .directives
        .iter()
        .filter(|directive| directive.alias == head)
        .collect::<Vec<_>>();
    let Some(max_depth) = matching.iter().map(|directive| directive.scope_depth).max() else {
        return resolve_explicit_type_reference(written, known_external, source_symbols);
    };
    let selected = matching
        .into_iter()
        .filter(|directive| directive.scope_depth == max_depth)
        .collect::<Vec<_>>();
    if selected.len() != 1 {
        return CSharpResolution::Ambiguous(CSharpResolutionReason::DuplicateAlias);
    }
    let directive = selected[0];
    if !visited.insert(directive.alias.clone()) {
        return CSharpResolution::Unsupported(CSharpResolutionReason::AliasCycle);
    }

    let substituted = format!("{}{}", directive.target.trim(), suffix.unwrap_or_default());
    let mut result = resolve_alias(
        &substituted,
        aliases,
        known_external,
        source_symbols,
        basis,
        visited,
    );
    visited.remove(&directive.alias);
    if let CSharpResolution::Resolved {
        basis: resolved_basis,
        evidence,
        ..
    } = &mut result
    {
        *resolved_basis = basis;
        evidence.insert(
            0,
            CSharpResolutionEvidence {
                file: directive.file.clone(),
                span: directive.span,
            },
        );
    }
    result
}

fn alias_head_and_suffix(written: &str) -> (&str, Option<&str>) {
    match alias_suffix(written) {
        Some(index) => (&written[..index], Some(&written[index..])),
        None => (written, None),
    }
}

fn alias_suffix(written: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, character) in written.char_indices() {
        match character {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            '.' if depth == 0 => return Some(index),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
#[path = "../../tests/ir/csharp_alias_resolution_r4.rs"]
mod tests;
