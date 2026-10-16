//! Mechanical request-local prefilter over C# declaration path and arity.

use super::{BaseReference, Catalog, Declaration, candidate_paths};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct DeclarationCandidateIndex {
    by_path_arity: HashMap<Vec<String>, HashMap<usize, Vec<usize>>>,
}

impl DeclarationCandidateIndex {
    pub(super) fn build(declarations: &[Declaration]) -> Self {
        let mut index = Self::default();
        for (position, declaration) in declarations.iter().enumerate() {
            index
                .by_path_arity
                .entry(declaration.path.clone())
                .or_default()
                .entry(declaration.arity)
                .or_default()
                .push(position);
        }
        #[cfg(test)]
        {
            let bucket_sizes = index
                .by_path_arity
                .values()
                .flat_map(|arities| arities.values().map(Vec::len));
            let mut keys = 0;
            let mut indexed = 0;
            let mut maximum = 0;
            for size in bucket_sizes {
                keys += 1;
                indexed += size;
                maximum = maximum.max(size);
            }
            crate::mcp::tool_handlers::query::work_metrics::record_declaration_candidate_index(
                keys, indexed, maximum,
            );
        }
        index
    }

    fn positions(&self, path: &[String], arity: usize) -> &[usize] {
        self.by_path_arity
            .get(path)
            .and_then(|arities| arities.get(&arity))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

impl Catalog {
    pub(super) fn declarations_for_path<'a>(
        &'a self,
        path: &[String],
        arity: usize,
    ) -> impl Iterator<Item = &'a Declaration> + 'a {
        self.declaration_candidates
            .positions(path, arity)
            .iter()
            .map(|position| &self.declarations[*position])
    }

    pub(super) fn resolve_declaration(&self, reference: &BaseReference) -> Option<&Declaration> {
        let mut paths = candidate_paths(reference);
        if reference.shape.qualifier.is_empty() {
            for import in &reference.imports {
                let mut path = import.clone();
                path.push(reference.shape.name.clone());
                paths.push(path);
            }
        }
        paths.dedup();
        for path in paths {
            let mut matches = self
                .declarations_for_path(&path, reference.shape.arity)
                .inspect(|_| {
                    #[cfg(test)]
                    crate::mcp::tool_handlers::query::work_metrics::record_csharp_declaration_resolution_comparison();
                });
            let declaration = matches.next();
            if matches.next().is_some() {
                return None;
            }
            if declaration.is_some() {
                return declaration;
            }
        }
        None
    }
}

#[cfg(test)]
#[path = "../../../../tests/mcp/csharp_declaration_candidates.rs"]
mod tests;
