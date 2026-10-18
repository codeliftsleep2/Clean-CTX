//! Artifact-backed C# exact-call projection and deterministic commit.
//!
//! Current source-bound artifacts supply declaration and invocation evidence.
//! Missing artifacts are compiled once, then every caller is projected against
//! the complete declaration catalog without reparsing or rereading retained
//! source snapshots.

use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::mcp::csharp_calls::authority::{
    authority_fingerprint, authority_manifest, merge_declarations,
};
use crate::mcp::semantic_compilation::artifact::SemanticCompilationArtifact;
use crate::mcp::semantic_compilation::executor::CompilationJob;
use crate::workspace::index::SemanticFidelity;
use std::path::Path;
use std::sync::Arc;

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

fn semantic_fidelity(fidelity: Fidelity) -> SemanticFidelity {
    match fidelity {
        Fidelity::Low => SemanticFidelity::Low,
        Fidelity::Medium => SemanticFidelity::Medium,
        Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => SemanticFidelity::High,
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

    pub(crate) fn mark_csharp_call_authority_dirty_for_file(
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

    pub(crate) fn project_csharp_exact_calls_for_generation(
        &self,
        generation: u64,
    ) -> Result<String, String> {
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
        let artifacts = self.current_csharp_artifacts(&files)?;
        let manifest = authority_manifest(
            &artifacts,
            self.config.additional_roots.iter().cloned(),
            self.config.exclude_patterns.iter().cloned(),
        );
        let fingerprint = authority_fingerprint(&manifest);
        let authority = merge_declarations(&artifacts);

        let mut projections = Vec::with_capacity(files.len());
        for artifact in artifacts {
            let file = artifact.canonical_owner().to_string();
            // Reconciliation may strengthen exact-call evidence, but it must
            // preserve the semantic fidelity of the publication it reconciles.
            let fidelity = self
                .workspace_index_read()
                .semantic_projection_fidelity(&file)
                .unwrap_or_else(|| semantic_fidelity(artifact.fidelity()));
            let edges = artifact.project_exact_csharp_calls(&authority)?;
            projections.push(Projection {
                dependencies: artifact.semantic_dependencies_owned(),
                file,
                edges,
                source_hash: artifact.source_hash().to_string(),
                fidelity,
            });
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
            index.commit_csharp_exact_call_coverage(fingerprint.0.clone(), generation);
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
        Ok(fingerprint.0)
    }

    fn current_csharp_artifacts(
        &self,
        files: &[String],
    ) -> Result<Vec<Arc<SemanticCompilationArtifact>>, String> {
        let mut artifacts = Vec::with_capacity(files.len());
        let mut missing = Vec::new();
        for file in files {
            let source = self.read_source(file).map_err(|error| error.to_string())?;
            let source_hash = crate::cache::content_hash(source.as_bytes());
            if let Some(artifact) = self.csharp_call_authority.artifact(file)
                && artifact.source_hash() == source_hash
            {
                artifacts.push(artifact);
                continue;
            }
            let fidelity = self
                .workspace_index_read()
                .semantic_projection_fidelity(file)
                .unwrap_or(SemanticFidelity::High);
            let path_alias = self.get_or_create_alias(file.clone());
            missing.push(CompilationJob {
                file_path: file.clone(),
                source,
                fidelity: compilation_fidelity(fidelity),
                previous_version: self.file_version(&path_alias).unwrap_or(0),
                path_alias,
                canonical_owner: file.clone(),
                skip_set: self.get_skip_set(file),
            });
        }
        for result in self.semantic_compiler().compile(&self.config, missing) {
            let artifact = Arc::new(result.result.map_err(|error| error.to_string())?);
            let current = self
                .read_source(&result.file_path)
                .map_err(|error| error.to_string())?;
            if crate::cache::content_hash(current.as_bytes()) != artifact.source_hash() {
                return Err(format!(
                    "C# artifact source changed during compilation: {}",
                    result.file_path
                ));
            }
            self.csharp_call_authority
                .retain_artifact(Arc::clone(&artifact));
            artifacts.push(artifact);
        }
        artifacts.sort_by(|left, right| left.canonical_owner().cmp(right.canonical_owner()));
        Ok(artifacts)
    }
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
#[path = "../../tests/mcp/csharp_call_reconciliation.rs"]
mod tests;
