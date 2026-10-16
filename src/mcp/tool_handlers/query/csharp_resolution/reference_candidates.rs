//! Mechanical request-local prefilter over C# reference facts.

use super::{BaseReference, Catalog};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct ReferenceCandidateIndex {
    by_file_owner: HashMap<String, HashMap<String, Vec<usize>>>,
}

impl ReferenceCandidateIndex {
    pub(super) fn build(references: &[BaseReference]) -> Self {
        let mut index = Self::default();
        for (position, reference) in references.iter().enumerate() {
            index
                .by_file_owner
                .entry(reference.file.clone())
                .or_default()
                .entry(reference.owner.clone())
                .or_default()
                .push(position);
        }
        #[cfg(test)]
        {
            let bucket_sizes = index
                .by_file_owner
                .values()
                .flat_map(|owners| owners.values().map(Vec::len));
            let mut keys = 0;
            let mut indexed = 0;
            let mut maximum = 0;
            for size in bucket_sizes {
                keys += 1;
                indexed += size;
                maximum = maximum.max(size);
            }
            crate::mcp::tool_handlers::query::work_metrics::record_reference_candidate_index(
                keys, indexed, maximum,
            );
        }
        index
    }

    fn positions(&self, file: &str, owner: &str) -> &[usize] {
        self.by_file_owner
            .get(file)
            .and_then(|owners| owners.get(owner))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

impl Catalog {
    pub(super) fn references_for_owner<'a>(
        &'a self,
        file: &str,
        owner: &str,
    ) -> impl Iterator<Item = &'a BaseReference> + 'a {
        self.reference_candidates
            .positions(file, owner)
            .iter()
            .map(|position| &self.references[*position])
    }
}

pub(in crate::mcp::tool_handlers::query) fn terminal_lookup_name(written: &str) -> Option<String> {
    let source = format!("class __CleanCtxProbe : {written} {{}}");
    let language = crate::compression::language::safe_csharp_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(&source, None)?;
    super::find_first_base_shape(tree.root_node(), source.as_bytes()).map(|shape| shape.name)
}
