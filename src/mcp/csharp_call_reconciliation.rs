//! C# exact-call reconciliation after declaration-authority changes.
//!
//! Invocation evidence remains compilation-local. The current admitted C#
//! source set is recompiled twice: once to reconstitute declaration authority,
//! then again to resolve every caller against that complete authority. All
//! source hashes are revalidated before one index replacement transaction.

use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::workspace::index::SemanticFidelity;
use std::path::Path;

struct Projection {
    file: String,
    edges: Vec<crate::layers::meta::semantic::SemanticEdge>,
    source_hash: String,
    dependencies: Vec<crate::layers::meta::semantic::SemanticDependency>,
    fidelity: SemanticFidelity,
}

fn compilation_fidelity(fidelity: SemanticFidelity) -> Fidelity {
    match fidelity {
        SemanticFidelity::Low => Fidelity::Low,
        SemanticFidelity::Medium => Fidelity::Medium,
        SemanticFidelity::High => Fidelity::High,
    }
}

impl McpState {
    #[cfg(test)]
    pub(crate) fn reset_test_csharp_reconciliation_count(&self) {
        self.csharp_call_authority.reset_reconciliation_count();
    }

    #[cfg(test)]
    pub(crate) fn test_csharp_reconciliation_count(&self) -> usize {
        self.csharp_call_authority.reconciliation_count()
    }

    pub(crate) fn reconcile_csharp_after_authority_change(
        &self,
        changed_file: &str,
    ) -> Result<(), String> {
        let is_csharp = Path::new(changed_file)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("cs"));
        if is_csharp {
            self.mark_csharp_authority_dirty();
        }
        Ok(())
    }

    pub(crate) fn reconcile_csharp_exact_calls(&self) -> Result<(), String> {
        #[cfg(test)]
        self.csharp_call_authority.record_reconciliation();

        let mut files: Vec<_> = self
            .workspace_index_read()
            .file_map()
            .keys()
            .filter(|file| is_admitted_csharp_source(self, file))
            .cloned()
            .collect();
        files.sort();
        files.dedup();
        if files.is_empty() {
            return Ok(());
        }

        let mut authority = Vec::new();
        for file in &files {
            let (compiled, _, _, _) =
                crate::mcp::tool_helpers::compile_file_ir_candidate(file, Fidelity::High, self)
                    .map_err(|error| error.to_string())?;
            authority.extend(crate::ir::csharp_exact_calls::declaration_authority(
                &compiled.instructions,
            ));
        }

        let authority_files = files.clone();
        let mut projections = Vec::with_capacity(files.len());
        for file in files {
            // Reconciliation may strengthen exact-call evidence, but it must
            // preserve the semantic fidelity of the publication it reconciles.
            let fidelity = self
                .workspace_index_read()
                .semantic_projection_fidelity(&file)
                .unwrap_or(SemanticFidelity::High);
            let (_, edges, _, source_hash) =
                crate::mcp::tool_helpers::compile_file_ir_candidate_with_csharp_authority(
                    &file,
                    compilation_fidelity(fidelity),
                    self,
                    &authority,
                )
                .map_err(|error| error.to_string())?;
            projections.push(Projection {
                dependencies: authority_dependencies(&file, &authority_files, &edges),
                file,
                edges,
                source_hash,
                fidelity,
            });
        }

        for projection in &projections {
            let source = self
                .read_source(&projection.file)
                .map_err(|error| error.to_string())?;
            let current_hash = self.cache_read().compute_hash(source.as_bytes());
            if current_hash != projection.source_hash {
                return Err(format!(
                    "C# exact-call reconciliation source changed during compilation: {}",
                    projection.file
                ));
            }
        }

        {
            let mut index = self.workspace_index_lock();
            for projection in &projections {
                index.replace_semantic_projection_with_dependencies(
                    &projection.file,
                    projection.edges.clone(),
                    projection.fidelity,
                    projection.source_hash.clone(),
                    projection.dependencies.clone(),
                );
            }
        }
        for projection in projections {
            if let Some(alias) = self.alias_for_path(&projection.file) {
                self.remember_semantic_projection(
                    &alias,
                    projection.edges,
                    projection.dependencies,
                );
            }
        }
        Ok(())
    }
}

fn authority_dependencies(
    owner: &str,
    authority_files: &[String],
    edges: &[crate::layers::meta::semantic::SemanticEdge],
) -> Vec<crate::layers::meta::semantic::SemanticDependency> {
    if !edges
        .iter()
        .any(|edge| edge.relation == crate::layers::meta::semantic::SemanticRelation::Calls)
    {
        return Vec::new();
    }
    let mut files: Vec<_> = authority_files
        .iter()
        .map(String::as_str)
        .filter(|file| *file != owner)
        .map(str::to_owned)
        .collect();
    files.sort();
    files.dedup();
    files
        .into_iter()
        .map(crate::layers::meta::semantic::SemanticDependency::current)
        .collect()
}

fn is_admitted_csharp_source(state: &McpState, file: &str) -> bool {
    let path = Path::new(file);
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("cs"))
        && !state.config.is_excluded(file)
}

#[cfg(test)]
#[path = "../tests/mcp/csharp_call_reconciliation.rs"]
mod tests;
