//! Admitted-source C# symbol identity and partial declaration composition.

use super::csharp_resolution::{CSharpDeclarationKind, CSharpSourceSpan};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CSharpSymbolKey {
    pub namespace_path: Vec<String>,
    pub containing_type_path: Vec<String>,
    pub name: String,
    pub generic_arity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpDeclarationPart {
    pub key: CSharpSymbolKey,
    pub kind: CSharpDeclarationKind,
    pub is_partial: bool,
    pub file: String,
    pub span: CSharpSourceSpan,
    pub base_references: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpSourceSymbol {
    pub key: CSharpSymbolKey,
    pub kind: CSharpDeclarationKind,
    pub parts: Vec<CSharpDeclarationPart>,
    pub base_reference: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSharpSymbolConflict {
    NonPartialDuplicate,
    KindMismatch,
    ConflictingBaseTypes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CSharpSymbolComposition {
    Resolved(CSharpSourceSymbol),
    Ambiguous(CSharpSymbolConflict),
}

#[derive(Debug, Default)]
pub struct CSharpSourceSymbolIndex {
    symbols: HashMap<CSharpSymbolKey, CSharpSymbolComposition>,
    qualified_candidates: HashMap<(Vec<String>, usize), Vec<CSharpSymbolKey>>,
    known_namespaces: HashSet<Vec<String>>,
}

impl CSharpSourceSymbolIndex {
    /// Compose physical declarations into source symbols.
    pub fn build(parts: impl IntoIterator<Item = CSharpDeclarationPart>) -> Self {
        let mut grouped: HashMap<CSharpSymbolKey, Vec<CSharpDeclarationPart>> = HashMap::new();
        for part in parts {
            grouped.entry(part.key.clone()).or_default().push(part);
        }

        let symbols: HashMap<CSharpSymbolKey, CSharpSymbolComposition> = grouped
            .into_iter()
            .map(|(key, mut parts)| {
                parts.sort_by(|left, right| {
                    (&left.file, left.span.start_byte, left.span.end_byte).cmp(&(
                        &right.file,
                        right.span.start_byte,
                        right.span.end_byte,
                    ))
                });
                let composition = compose_symbol(key.clone(), parts);
                (key, composition)
            })
            .collect();
        let mut qualified_candidates: HashMap<(Vec<String>, usize), Vec<CSharpSymbolKey>> =
            HashMap::new();
        let mut known_namespaces = HashSet::new();
        for key in symbols.keys() {
            known_namespaces.insert(key.namespace_path.clone());
            let mut path = key.namespace_path.clone();
            path.extend(key.containing_type_path.iter().cloned());
            path.push(key.name.clone());
            qualified_candidates
                .entry((path, key.generic_arity))
                .or_default()
                .push(key.clone());
        }
        for candidates in qualified_candidates.values_mut() {
            candidates.sort();
        }
        Self {
            symbols,
            qualified_candidates,
            known_namespaces,
        }
    }

    pub fn get(&self, key: &CSharpSymbolKey) -> Option<&CSharpSymbolComposition> {
        self.symbols.get(key)
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub(crate) fn matching_qualified_path(
        &self,
        path: &[String],
        generic_arity: usize,
    ) -> Vec<&CSharpSymbolComposition> {
        self.qualified_candidates
            .get(&(path.to_vec(), generic_arity))
            .into_iter()
            .flatten()
            .filter_map(|key| self.symbols.get(key))
            .collect()
    }

    pub(crate) fn knows_namespace(&self, namespace_path: &[String]) -> bool {
        self.known_namespaces.contains(namespace_path)
    }
}

fn compose_symbol(
    key: CSharpSymbolKey,
    parts: Vec<CSharpDeclarationPart>,
) -> CSharpSymbolComposition {
    let Some(first) = parts.first() else {
        unreachable!("a grouped symbol always contains at least one declaration part");
    };
    let kind = first.kind;
    if parts.iter().any(|part| part.kind != kind) {
        return CSharpSymbolComposition::Ambiguous(CSharpSymbolConflict::KindMismatch);
    }
    if parts.len() > 1 && parts.iter().any(|part| !part.is_partial) {
        return CSharpSymbolComposition::Ambiguous(CSharpSymbolConflict::NonPartialDuplicate);
    }

    let mut bases = parts
        .iter()
        .flat_map(|part| part.base_references.iter())
        .map(|base| base.trim())
        .filter(|base| !base.is_empty())
        .collect::<Vec<_>>();
    bases.sort_unstable();
    bases.dedup();
    if bases.len() > 1 {
        return CSharpSymbolComposition::Ambiguous(CSharpSymbolConflict::ConflictingBaseTypes);
    }
    let base_reference = bases.first().map(|base| (*base).to_string());

    CSharpSymbolComposition::Resolved(CSharpSourceSymbol {
        key,
        kind,
        parts,
        base_reference,
    })
}

#[cfg(test)]
#[path = "../tests/ir/csharp_symbol_index_r1.rs"]
mod tests;
