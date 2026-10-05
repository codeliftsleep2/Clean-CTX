use super::HydrationRequirement;
use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::workspace::index::SemanticFidelity;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CandidatePublication {
    Published,
    Current,
    Failed,
}

pub(super) fn select_candidates(
    state: &McpState,
    candidates: Vec<String>,
    requirement: HydrationRequirement,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let resolved: Vec<_> = candidates
        .into_iter()
        .map(|path| state.semantic_owner_path(&path))
        .filter(|path| current_source_candidate_is_eligible(state, path))
        .filter(|path| seen.insert(path.clone()))
        .collect();
    let indexed = state.workspace_index_read();
    let mut selected: Vec<_> = resolved
        .into_iter()
        .filter(|path| match requirement {
            HydrationRequirement::LegacyEdit => !indexed.file_map().contains_key(path),
            HydrationRequirement::Semantic(_) => true,
        })
        .collect();
    drop(indexed);
    selected.sort();
    selected
}

fn current_source_candidate_is_eligible(state: &McpState, path: &str) -> bool {
    let path = std::path::Path::new(path);
    path.is_file()
        && !state.config.is_excluded(&path.to_string_lossy())
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(crate::compression::language::language_for_extension)
            .is_some()
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn compile_candidate(
    state: &McpState,
    resolved_path: &str,
    workspace_root: Option<&str>,
) -> bool {
    compile_candidate_for(
        state,
        resolved_path,
        workspace_root,
        HydrationRequirement::LegacyEdit,
    )
    .is_ok_and(|outcome| outcome == CandidatePublication::Published)
}

pub(super) fn compile_candidate_for(
    state: &McpState,
    resolved_path: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
) -> Result<CandidatePublication, String> {
    let validated = match super::super::super::tool_helpers::resolve_file_path_checked(
        resolved_path,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(path) => path,
        Err(_) => return Ok(CandidatePublication::Failed),
    };
    let canonical = state.semantic_owner_path(&validated);
    let publication = state.begin_semantic_publication(&canonical);
    let (fidelity, coverage) = match requirement {
        HydrationRequirement::LegacyEdit => (Fidelity::Edit, None),
        HydrationRequirement::Semantic(coverage) => {
            let source = state
                .read_source(&validated)
                .map_err(|error| error.to_string())?;
            let source_hash = state.cache_read().compute_hash(source.as_bytes());
            if state
                .workspace_index_read()
                .has_current_semantic_projection(&canonical, coverage, &source_hash)
            {
                return Ok(CandidatePublication::Current);
            }
            let fidelity = match coverage {
                SemanticFidelity::Low => Fidelity::Low,
                SemanticFidelity::Medium => Fidelity::Medium,
                SemanticFidelity::High => Fidelity::High,
            };
            (fidelity, Some(coverage))
        }
    };

    #[cfg(all(test, feature = "rust"))]
    if super::take_test_publication_failure(&canonical) {
        return Ok(CandidatePublication::Failed);
    }

    state.preflight_semantic_publication(&validated)?;
    match super::super::super::tool_helpers::compile_file_ir_focused(
        &validated, fidelity, state, None,
    ) {
        Ok((_, semantic_edges, compiled_hash)) => {
            let expected_hash = compiled_hash.clone();
            publication
                .commit(
                    || {
                        state
                            .read_source(&validated)
                            .map(|current| {
                                state.cache_read().compute_hash(current.as_bytes()) == expected_hash
                            })
                            .map_err(|error| error.to_string())
                    },
                    || {
                        let mut index = state.workspace_index_lock();
                        if let Some(coverage) = coverage {
                            index.replace_semantic_projection(
                                &canonical,
                                semantic_edges,
                                coverage,
                                compiled_hash,
                            );
                        } else if !semantic_edges.is_empty() {
                            index.remove_file(&canonical);
                            index.add_edges(&canonical, semantic_edges);
                        }
                        Ok::<_, String>(CandidatePublication::Published)
                    },
                )
                .map(|published| published.unwrap_or(CandidatePublication::Failed))
        }
        Err(_) => Ok(CandidatePublication::Failed),
    }
}
