use super::HydrationRequirement;
use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::mcp::semantic_compilation::executor::{CompilationBatchResult, CompilationJob};
use crate::mcp::state::publication::SemanticPublicationTicket;
use crate::workspace::index::SemanticFidelity;
use std::collections::HashSet;
#[cfg(feature = "csharp")]
use std::sync::Arc;

#[cfg(all(test, feature = "rust"))]
static TEST_BATCH_SUPERSESSION: std::sync::Mutex<Option<(std::thread::ThreadId, String, String)>> =
    std::sync::Mutex::new(None);

#[cfg(all(test, feature = "rust"))]
pub(crate) fn supersede_test_batch_once(path: String, source: String) {
    *TEST_BATCH_SUPERSESSION
        .lock()
        .expect("batch supersession lock") = Some((std::thread::current().id(), path, source));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::mcp::tool_handlers) enum CandidatePublication {
    Published,
    PublishedCSharp,
    Current,
    PublicationFailed,
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
    let mut selected: Vec<_> = resolved
        .into_iter()
        .filter(|path| match requirement {
            HydrationRequirement::LegacyEdit => !has_current_legacy_edit_projection(state, path),
            HydrationRequirement::Semantic(_) => true,
        })
        .collect();
    selected.sort();
    selected
}

fn has_current_legacy_edit_projection(state: &McpState, path: &str) -> bool {
    let Ok(source) = state.read_source(path) else {
        return false;
    };
    let source_hash = state.cache_read().compute_hash(source.as_bytes());
    state
        .workspace_index_read()
        .has_current_semantic_projection(path, SemanticFidelity::High, &source_hash)
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
    match compile_candidate_for(
        state,
        resolved_path,
        workspace_root,
        HydrationRequirement::LegacyEdit,
    ) {
        Ok(CandidatePublication::Published) => true,
        Ok(CandidatePublication::PublishedCSharp) => state.prepare_exact_csharp_calls().is_ok(),
        Ok(
            CandidatePublication::Current
            | CandidatePublication::PublicationFailed
            | CandidatePublication::Failed,
        )
        | Err(_) => false,
    }
}

#[cfg(all(test, feature = "rust"))]
pub(super) fn compile_candidate_for(
    state: &McpState,
    resolved_path: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
) -> Result<CandidatePublication, String> {
    let paths = [resolved_path.to_string()];
    Ok(
        compile_candidates_for(state, &paths, workspace_root, requirement)?
            .into_iter()
            .next()
            .unwrap_or(CandidatePublication::Failed),
    )
}

struct PreparedPublication {
    validated: String,
    canonical: String,
    coverage: SemanticFidelity,
    published_outcome: CandidatePublication,
    ticket: SemanticPublicationTicket,
}

enum PreparedCandidate {
    Compile(PreparedPublication, Box<CompilationJob>),
    Terminal(CandidatePublication),
}

pub(in crate::mcp::tool_handlers) fn compile_candidates_for(
    state: &McpState,
    resolved_paths: &[String],
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
) -> Result<Vec<CandidatePublication>, String> {
    let mut publications = Vec::new();
    let mut jobs = Vec::new();
    let mut terminal = Vec::new();
    for resolved_path in resolved_paths {
        match prepare_candidate(state, resolved_path, workspace_root, requirement)? {
            PreparedCandidate::Compile(publication, job) => {
                publications.push(publication);
                jobs.push(*job);
            }
            PreparedCandidate::Terminal(outcome) => terminal.push(outcome),
        }
    }
    let compiled = state.semantic_compiler().compile(&state.config, jobs);
    #[cfg(all(test, feature = "rust"))]
    let supersession = {
        let mut guard = TEST_BATCH_SUPERSESSION
            .lock()
            .expect("batch supersession lock");
        if guard
            .as_ref()
            .is_some_and(|(owner, _, _)| *owner == std::thread::current().id())
        {
            guard.take().map(|(_, path, source)| (path, source))
        } else {
            None
        }
    };
    #[cfg(all(test, feature = "rust"))]
    if let Some((path, source)) = supersession {
        std::fs::write(&path, source).expect("superseding source write");
        state.invalidate_source_cache(&path);
    }
    if compiled.iter().any(|result| result.result.is_err())
        || publications
            .iter()
            .zip(&compiled)
            .any(|(publication, result)| {
                let Ok(artifact) = result.result.as_ref() else {
                    return true;
                };
                state
                    .read_source(&publication.validated)
                    .map(|source| {
                        crate::cache::content_hash(source.as_bytes()) != artifact.source_hash()
                    })
                    .unwrap_or(true)
            })
    {
        terminal.extend(std::iter::repeat_n(
            CandidatePublication::Failed,
            publications.len(),
        ));
        return Ok(terminal);
    }
    for (publication, compiled) in publications.into_iter().zip(compiled) {
        terminal.push(commit_candidate(state, publication, compiled)?);
    }
    Ok(terminal)
}

fn prepare_candidate(
    state: &McpState,
    resolved_path: &str,
    workspace_root: Option<&str>,
    requirement: HydrationRequirement,
) -> Result<PreparedCandidate, String> {
    let validated = match super::super::super::tool_helpers::resolve_file_path_checked(
        resolved_path,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(path) => path,
        Err(_) => return Ok(PreparedCandidate::Terminal(CandidatePublication::Failed)),
    };
    let canonical = state.semantic_owner_path(&validated);
    let published_outcome = if cfg!(feature = "csharp")
        && std::path::Path::new(&canonical)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("cs"))
    {
        CandidatePublication::PublishedCSharp
    } else {
        CandidatePublication::Published
    };
    let publication = state.begin_semantic_publication(&canonical);
    let (fidelity, coverage) = match requirement {
        HydrationRequirement::LegacyEdit => {
            if has_current_legacy_edit_projection(state, &canonical) {
                return Ok(PreparedCandidate::Terminal(CandidatePublication::Current));
            }
            (Fidelity::Edit, SemanticFidelity::High)
        }
        HydrationRequirement::Semantic(coverage) => {
            let source = state
                .read_source(&validated)
                .map_err(|error| error.to_string())?;
            let source_hash = state.cache_read().compute_hash(source.as_bytes());
            if state
                .workspace_index_read()
                .has_current_semantic_projection(&canonical, coverage, &source_hash)
            {
                return Ok(PreparedCandidate::Terminal(CandidatePublication::Current));
            }
            let fidelity = match coverage {
                SemanticFidelity::Low => Fidelity::Low,
                SemanticFidelity::Medium => Fidelity::Medium,
                SemanticFidelity::High => Fidelity::High,
            };
            (fidelity, coverage)
        }
    };

    #[cfg(all(test, feature = "rust"))]
    if super::take_test_publication_failure(&canonical) {
        return Ok(PreparedCandidate::Terminal(
            CandidatePublication::PublicationFailed,
        ));
    }

    state.preflight_semantic_publication(&validated)?;
    let source = match state.read_source(&validated) {
        Ok(source) => source,
        Err(_) => return Ok(PreparedCandidate::Terminal(CandidatePublication::Failed)),
    };
    let path_alias = state.get_or_create_alias(validated.clone());
    let previous_version = state.file_version(&path_alias).unwrap_or(0);
    let job = CompilationJob {
        authorized_roots: super::filesystem::configured_roots(state, workspace_root),
        typescript_project: None,
        file_path: validated.clone(),
        source,
        fidelity,
        path_alias,
        previous_version,
        canonical_owner: canonical.clone(),
        skip_set: state.get_skip_set(&validated),
    };
    Ok(PreparedCandidate::Compile(
        PreparedPublication {
            validated,
            canonical,
            coverage,
            published_outcome,
            ticket: publication,
        },
        Box::new(job),
    ))
}

fn commit_candidate(
    state: &McpState,
    publication: PreparedPublication,
    compiled: CompilationBatchResult,
) -> Result<CandidatePublication, String> {
    debug_assert_eq!(compiled.file_path, publication.validated);
    let Ok(artifact) = compiled.result else {
        return Ok(CandidatePublication::Failed);
    };
    let (semantic_edges, semantic_dependencies, compiled_hash) = artifact.published_semantics();
    #[cfg(feature = "csharp")]
    let retained = (publication.published_outcome == CandidatePublication::PublishedCSharp)
        .then(|| Arc::new(artifact));
    let expected_hash = compiled_hash.clone();
    let expected_dependencies = semantic_dependencies.clone();
    publication
        .ticket
        .commit(
            || {
                state
                    .read_source(&publication.validated)
                    .map(|current| {
                        crate::cache::content_hash(current.as_bytes()) == expected_hash
                            && expected_dependencies
                                .iter()
                                .all(|dependency| dependency.is_current())
                    })
                    .map_err(|error| error.to_string())
            },
            || {
                #[cfg(feature = "csharp")]
                if let Some(artifact) = retained {
                    state.csharp_call_authority.retain_artifact(artifact);
                }
                state
                    .workspace_index_lock()
                    .replace_semantic_projection_with_dependencies(
                        &publication.canonical,
                        semantic_edges,
                        publication.coverage,
                        compiled_hash,
                        semantic_dependencies,
                    );
                #[cfg(feature = "csharp")]
                if publication.published_outcome == CandidatePublication::PublishedCSharp {
                    state.mark_csharp_authority_dirty();
                }
                Ok::<_, String>(publication.published_outcome)
            },
        )
        .map(|published| published.unwrap_or(CandidatePublication::PublicationFailed))
}
