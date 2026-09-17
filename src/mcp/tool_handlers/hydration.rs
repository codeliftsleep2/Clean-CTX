// workspace_query semantic hydration support.
//
// Responsibilities split across submodules:
//   cache.rs         — hydration discovery-cache integration (hit/miss/mark)
//   filesystem.rs    — configured roots + the literal filesystem scan
//   invalidation.rs  — discovery-cache invalidation entry points
//   test_support.rs  — test-only discovery instrumentation (cfg(test,rust))

use crate::mcp::McpState;
use crate::mcp::discovery_cache::{DiscoveryMode, DiscoveryScope};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod cache;
use cache::{
    discovery_is_complete, mark_discovery_complete, mark_discovery_complete_for_roots,
    pending_discovery_roots,
};

mod filesystem;
use filesystem::{configured_roots, deduplicate_roots, root_key, scan};

mod invalidation;
pub(crate) use invalidation::{
    invalidate_discovery_for_edited_path, invalidate_discovery_for_root,
};

#[cfg(all(test, feature = "rust"))]
pub(crate) use filesystem::{
    TraversalStats, last_test_traversal_stats, reset_test_scan_calls, test_scan_calls,
};

#[cfg(all(test, feature = "rust"))]
use crate::cbm::GraphNode;

// Test-only discovery instrumentation (project-search injection, traversal
// counters). Kept in its own module so this file stays inside the active-file
// ceiling; every name the regressions import from
// `crate::mcp::tool_handlers::hydration` is re-exported here unchanged.
#[cfg(all(test, feature = "rust"))]
mod test_support;

#[cfg(all(test, feature = "rust"))]
pub(crate) use test_support::*;

/// Diagnostic bound for the LLM-facing project-coverage projection.
///
/// The bound belongs to the RESPONSE projection (`query::diagnostics`), not to
/// the internal record: [HydrationReport] keeps every coverage entry so internal
/// debugging, telemetry and the discovery regressions can inspect the complete
/// result, while only the exceptional entries the caller actually sees count
/// against this cap. Exceeding it is reported structurally (`projects_truncated`)
/// rather than by dropping information silently.
pub(super) const HYDRATION_MAX_PROJECT_COVERAGE: usize = 16;

/// The complete result of one hydration discovery/compile cycle.
///
/// This is the INTERNAL record: it deliberately keeps the full provider,
/// completion, candidate and per-project coverage detail. What an LLM-facing
/// response serializes is a sparse projection of it
/// (`crate::mcp::tool_handlers::query::diagnostics`), which omits expected state
/// and surfaces only decision-relevant deviation.
#[derive(Default)]
pub(super) struct HydrationReport {
    pub(super) hydration_attempted: bool,
    pub(super) discovery_provider: &'static str,
    /// The single completion source of truth: `"completed"` iff discovery
    /// completed across every configured root. The former separate
    /// `discovery_completed` boolean encoded exactly this state (it was `true`
    /// iff this was `"completed"` on every return path, verified against the
    /// tree), and two encodings of one state are not retained — neither in the
    /// response nor in the internal record.
    pub(super) discovery_status: &'static str,
    /// The ONE encoding of "the filesystem fallback was engaged": when this is
    /// `Some`, fallback ran. The former `fallback_occurred` boolean was `true` on
    /// exactly the return paths that carry a reason and `false` on exactly the
    /// paths that carry none, so it encoded nothing this does not.
    pub(super) fallback_reason: Option<&'static str>,
    pub(super) candidates_discovered: usize,
    pub(super) candidates_compiled: usize,
    pub(super) project_coverage: Vec<ProjectCoverage>,
}

#[derive(Serialize)]
pub(super) struct ProjectCoverage {
    pub(super) project: String,
    pub(super) status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) readiness: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) reason: Option<&'static str>,
}

struct DiscoveryOutcome {
    candidates: Vec<String>,
    project_coverage: Vec<ProjectCoverage>,
    provider: &'static str,
    /// `"completed"` / `"partial"` / `"failed"` — the one completion encoding.
    status: &'static str,
    attempted: bool,
    fallback_reason: Option<&'static str>,
}

pub(super) fn is_hydration_eligible(query_type: &str, args: &Value) -> bool {
    matches!(
        query_type,
        "find_entities" | "forward_edges" | "reverse_edges" | "transitive_dependencies"
    ) && args["name"].as_str().is_some_and(|name| !name.is_empty())
}

pub(super) fn hydrate_workspace_index(
    state: &McpState,
    query_type: &str,
    query_name: &str,
    workspace_root: Option<&str>,
) -> HydrationReport {
    let discovery = discovery_kind(query_type);
    let outcome = discover_candidate_paths(state, discovery, query_name, workspace_root);
    let candidate_paths = outcome.candidates;
    let mut project_coverage = outcome.project_coverage;
    let discovered = candidate_paths.len();
    let selected = select_candidates(state, candidate_paths);
    let compiled = selected
        .iter()
        .filter(|path| compile_candidate(state, path, workspace_root))
        .count();
    project_coverage.sort_by(|left, right| left.project.cmp(&right.project));
    // The complete coverage set is retained here; the diagnostic bound is applied
    // by the LLM-facing projection (`query::diagnostics`), which also drops the
    // healthy entries so they cannot consume the bound.
    HydrationReport {
        hydration_attempted: outcome.attempted,
        discovery_provider: outcome.provider,
        discovery_status: outcome.status,
        fallback_reason: outcome.fallback_reason,
        candidates_discovered: discovered,
        candidates_compiled: compiled,
        project_coverage,
    }
}

/// Map a `workspace_query` type to the discovery operation hydration must run.
///
/// Discovery-mode equivalence classes (one shared discovery cache entry per
/// project/root + name, because the underlying discovery operation is
/// identical):
///   - `find_entities`, `forward_edges`, `transitive_dependencies`
///     → `DiscoveryMode::Declaration` (name/declaration search)
///   - `reverse_edges` → `DiscoveryMode::InboundReference`
///     (`MATCH (caller)-[r]->(target)`)
fn discovery_kind(query_type: &str) -> DiscoveryMode {
    match query_type {
        "reverse_edges" => DiscoveryMode::InboundReference,
        _ => DiscoveryMode::Declaration,
    }
}

fn discover_candidate_paths(
    state: &McpState,
    discovery: DiscoveryMode,
    query_name: &str,
    workspace_root: Option<&str>,
) -> DiscoveryOutcome {
    #[cfg(all(test, feature = "rust"))]
    {
        if !has_test_project_search() {
            if let Ok(injected) = TEST_HYDRATION_CANDIDATES.lock() {
                if let Some(paths) = injected.as_ref() {
                    return DiscoveryOutcome {
                        candidates: paths.clone(),
                        project_coverage: Vec::new(),
                        provider: "cbm",
                        status: "completed",
                        attempted: true,
                        fallback_reason: None,
                    };
                }
            }
        }
    }

    let filesystem_roots = configured_roots(state, workspace_root);
    let mut bridge_guard = state.graph_bridge_lock();
    let Some(bridge) = bridge_guard.as_mut() else {
        drop(bridge_guard);
        return filesystem_only_discovery(
            state,
            discovery,
            filesystem_roots,
            query_name,
            "cbm_unavailable",
        );
    };
    let projects = bridge.configured_projects();
    let mut candidates = Vec::new();
    let mut coverage = Vec::new();
    let mut successful_roots = HashSet::new();
    let mut fallback_roots = Vec::new();
    let mut cbm_attempted = false;
    let mut saw_unavailable = false;
    let mut saw_search_failure = false;

    for configured_root in &state.config.additional_roots {
        let resolved = bridge.resolve_project_id(configured_root);
        if !projects.iter().any(|(project, _)| project == &resolved) {
            coverage.push(ProjectCoverage {
                project: resolved,
                status: "skipped",
                readiness: None,
                reason: Some("additional_root_not_registered"),
            });
            fallback_roots.push(PathBuf::from(configured_root));
        }
    }

    for (project, root) in projects {
        let scope = DiscoveryScope::cbm(root_key(&root));
        // Discovery completion gate (see `crate::mcp::discovery_cache`).
        //
        // A successful discovery for this project/mode/name is not repeated
        // while the project's discovery generation is unchanged: every
        // candidate it found was compiled into the WorkspaceIndex by the
        // hydration that completed it, so this generation is already hydrated
        // for this target. Skipping here also skips `ensure_indexed_for` — the
        // only in-session producer of CBM dirty state is `apply_edit`, which
        // invalidates this scope, and every other CBM consumer resolves its
        // own indexing gate.
        if discovery_is_complete(state, &scope, discovery, query_name) {
            cbm_attempted = true;
            successful_roots.insert(root_key(&root));
            coverage.push(ProjectCoverage {
                project,
                status: "searched",
                readiness: Some("ready"),
                reason: None,
            });
            continue;
        }

        #[cfg(all(test, feature = "rust"))]
        if let Some(readiness) = test_project_readiness(&project) {
            let readiness = match readiness {
                TestProjectReadiness::Ready => "ready",
                TestProjectReadiness::StillIndexing => "still_indexing",
                TestProjectReadiness::Failed => "failed",
                TestProjectReadiness::Unavailable => {
                    saw_unavailable = true;
                    fallback_roots.push(root.clone());
                    coverage.push(ProjectCoverage {
                        project,
                        status: "skipped",
                        readiness: None,
                        reason: Some("cbm_unavailable"),
                    });
                    continue;
                }
            };
            cbm_attempted = true;
            let result =
                test_project_search(&project, discovery).expect("configured project search result");
            match result {
                Ok(nodes) => {
                    append_project_paths(&mut candidates, &root, nodes);
                    successful_roots.insert(root_key(&root));
                    coverage.push(ProjectCoverage {
                        project,
                        status: "searched",
                        readiness: Some(readiness),
                        reason: None,
                    });
                    // Only a *completed* discovery is recorded: a project that
                    // is still indexing, or that has not yet finished, may
                    // return an incomplete candidate set and must stay
                    // retryable instead of being cached as "searched".
                    if readiness == "ready" {
                        mark_discovery_complete(state, &scope, discovery, query_name);
                    }
                }
                Err(_) => {
                    saw_search_failure = true;
                    fallback_roots.push(root);
                    coverage.push(ProjectCoverage {
                        project,
                        status: "search_failed",
                        readiness: Some(readiness),
                        reason: Some("search_failed"),
                    });
                }
            }
            continue;
        }

        if !bridge.is_available() {
            saw_unavailable = true;
            fallback_roots.push(root);
            coverage.push(ProjectCoverage {
                project,
                status: "skipped",
                readiness: None,
                reason: Some("cbm_unavailable"),
            });
            continue;
        }
        let readiness = match bridge.ensure_indexed_for(&project) {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => "ready",
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => "still_indexing",
            Err(_) => "failed",
        };
        cbm_attempted = true;
        let discovered = match discovery {
            DiscoveryMode::Declaration => bridge
                .search_in_project(&project, query_name)
                .map(|nodes| nodes.into_iter().map(|node| node.file).collect()),
            DiscoveryMode::InboundReference => {
                bridge.inbound_reference_paths_in_project(&project, query_name)
            }
        };
        match discovered {
            Ok(paths) => {
                append_rooted_paths(&mut candidates, &root, paths);
                successful_roots.insert(root_key(&root));
                coverage.push(ProjectCoverage {
                    project,
                    status: "searched",
                    readiness: Some(readiness),
                    reason: None,
                });
                // Successful discovery only — including discovery that found
                // zero candidates. Failed or not-yet-ready discovery is never
                // recorded, so it stays eligible for retry.
                if readiness == "ready" {
                    mark_discovery_complete(state, &scope, discovery, query_name);
                }
            }
            Err(_) => {
                saw_search_failure = true;
                fallback_roots.push(root);
                coverage.push(ProjectCoverage {
                    project,
                    status: "search_failed",
                    readiness: Some(readiness),
                    reason: Some("search_failed"),
                });
            }
        }
    }
    drop(bridge_guard);

    for root in filesystem_roots {
        if !successful_roots.contains(&root_key(&root)) {
            fallback_roots.push(root);
        }
    }
    deduplicate_roots(&mut fallback_roots);

    // Filesystem fallback discovery, split into roots that still require a
    // scan and roots whose discovery already completed for this generation.
    // A cached root contributes no new candidates (its results were compiled
    // when it completed) but still counts as covered, so it is neither
    // re-scanned nor reported as un-covered.
    let (pending_roots, cached_roots) =
        pending_discovery_roots(state, discovery, query_name, fallback_roots);

    let fallback_reason = match (saw_unavailable, saw_search_failure) {
        (true, true) => "cbm_partial_failure",
        (true, false) => "cbm_unavailable",
        (false, true) => "cbm_discovery_failed",
        (false, false) => "cbm_scope_unavailable",
    };

    if pending_roots.is_empty() {
        if cached_roots == 0 {
            // No root requires filesystem discovery at all: unchanged CBM-only
            // outcome.
            return DiscoveryOutcome {
                candidates,
                project_coverage: coverage,
                provider: "cbm",
                status: "completed",
                attempted: cbm_attempted,
                fallback_reason: None,
            };
        }
        // Every fallback root is already discovered for this generation, so no
        // scan runs: the previously discovered (and compiled) candidates are
        // still the complete answer for this target.
        let any_cbm_success = !successful_roots.is_empty();
        return DiscoveryOutcome {
            candidates,
            project_coverage: coverage,
            provider: if any_cbm_success {
                "cbm_and_filesystem"
            } else {
                "filesystem"
            },
            status: "completed",
            attempted: true,
            fallback_reason: Some(fallback_reason),
        };
    }

    let scan = scan(state, &pending_roots, query_name);
    if scan.completed {
        mark_discovery_complete_for_roots(state, discovery, query_name, &pending_roots);
    }
    candidates.extend(scan.candidates);
    let any_cbm_success = !successful_roots.is_empty();
    let attempted = cbm_attempted || scan.attempted || cached_roots > 0;
    let filesystem_covers = scan.attempted || cached_roots > 0;
    let provider = match (any_cbm_success, filesystem_covers) {
        (true, true) => "cbm_and_filesystem",
        (true, false) => "cbm",
        (false, true) => "filesystem",
        (false, false) => "none",
    };
    let completed = scan.completed;
    DiscoveryOutcome {
        candidates,
        project_coverage: coverage,
        provider,
        status: discovery_status(attempted, completed, any_cbm_success),
        attempted,
        fallback_reason: Some(fallback_reason),
    }
}

fn filesystem_only_discovery(
    state: &McpState,
    discovery: DiscoveryMode,
    roots: Vec<PathBuf>,
    query_name: &str,
    fallback_reason: &'static str,
) -> DiscoveryOutcome {
    let (pending_roots, cached_roots) =
        pending_discovery_roots(state, discovery, query_name, roots);
    if pending_roots.is_empty() {
        if cached_roots == 0 {
            // No root was configured (or every root was already unusable):
            // identical to scanning nothing.
            return DiscoveryOutcome {
                candidates: Vec::new(),
                project_coverage: Vec::new(),
                provider: "none",
                status: "failed",
                attempted: false,
                fallback_reason: Some("filesystem_unavailable"),
            };
        }
        // Nothing left to scan: every configured root's fallback discovery
        // already completed for this generation (and its candidates were
        // compiled then), so the full-repo walk is skipped entirely.
        return DiscoveryOutcome {
            candidates: Vec::new(),
            project_coverage: Vec::new(),
            provider: "filesystem",
            status: "completed",
            attempted: true,
            fallback_reason: Some(fallback_reason),
        };
    }
    let scan = scan(state, &pending_roots, query_name);
    if scan.completed {
        mark_discovery_complete_for_roots(state, discovery, query_name, &pending_roots);
    }
    let covered = scan.attempted || cached_roots > 0;
    let provider = if covered { "filesystem" } else { "none" };
    DiscoveryOutcome {
        candidates: scan.candidates,
        project_coverage: Vec::new(),
        provider,
        status: discovery_status(covered, scan.completed, false),
        attempted: covered,
        fallback_reason: Some(if covered {
            fallback_reason
        } else {
            "filesystem_unavailable"
        }),
    }
}

fn discovery_status(attempted: bool, completed: bool, has_partial_coverage: bool) -> &'static str {
    if completed {
        "completed"
    } else if attempted || has_partial_coverage {
        "partial"
    } else {
        "failed"
    }
}

#[cfg(all(test, feature = "rust"))]
fn append_project_paths(candidates: &mut Vec<String>, root: &Path, nodes: Vec<GraphNode>) {
    append_rooted_paths(
        candidates,
        root,
        nodes.into_iter().map(|node| node.file).collect(),
    );
}

fn append_rooted_paths(candidates: &mut Vec<String>, root: &Path, paths: Vec<String>) {
    candidates.extend(paths.into_iter().filter_map(|file| {
        if file.is_empty() {
            return None;
        }
        let path = PathBuf::from(file);
        let rooted = if path.is_absolute() {
            path
        } else {
            root.join(path)
        };
        Some(crate::dictionary::path::canonical_identity_key(
            &rooted.to_string_lossy(),
        ))
    }));
}

fn select_candidates(state: &McpState, candidates: Vec<String>) -> Vec<String> {
    let indexed = state.workspace_index_read();
    let mut seen = HashSet::new();
    let mut selected: Vec<_> = candidates
        .into_iter()
        .map(|path| crate::dictionary::path::canonical_identity_key(&path))
        .filter(|path| seen.insert(path.clone()))
        .filter(|path| !indexed.file_map().contains_key(path))
        .collect();
    drop(indexed);
    selected.sort();
    selected
}

pub(crate) fn compile_candidate(
    state: &McpState,
    resolved_path: &str,
    workspace_root: Option<&str>,
) -> bool {
    let validated = match super::super::tool_helpers::resolve_file_path_checked(
        resolved_path,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(path) => path,
        Err(_) => return false,
    };

    match super::super::tool_helpers::compile_file_ir_focused(
        &validated,
        crate::compression::Fidelity::Edit,
        state,
        None,
    ) {
        Ok((_, semantic_edges, _)) => {
            if !semantic_edges.is_empty() {
                let canonical = crate::dictionary::path::canonical_identity_key(&validated);
                let mut index = state.workspace_index_lock();
                index.remove_file(&canonical);
                index.add_edges(&canonical, semantic_edges);
            }
            true
        }
        Err(_) => false,
    }
}

// Discovery-cache invalidation entry points live in `hydration/invalidation.rs`
// (called by `apply_edit` and explicit repository reindexing), and the cache
// integration in `hydration/cache.rs`.
