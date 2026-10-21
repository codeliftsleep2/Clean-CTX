// workspace_query semantic hydration support.
//
// Responsibilities split across submodules:
//   cache.rs         — hydration discovery-cache integration (hit/miss/mark)
//   filesystem.rs    — configured roots + the literal filesystem scan
//   invalidation.rs  — discovery-cache invalidation entry points
//   test_support.rs  — test-only discovery instrumentation (cfg(test,rust))

use crate::mcp::McpState;
use crate::mcp::acquisition::{AcquisitionOutcome, AcquisitionSatisfaction};
use crate::mcp::discovery_cache::{DiscoveryMode, DiscoveryScope};
use serde::Serialize;
#[cfg(all(test, feature = "rust"))]
use serde_json::Value;
use std::path::PathBuf;

mod cache;
use cache::{discovery_is_complete, discovery_scopes_for_roots, pending_discovery_roots};

mod candidates;
#[cfg(all(test, feature = "rust"))]
use candidates::append_project_paths;
use candidates::{append_rooted_paths, discovery_status};

mod acquisition;
#[cfg(feature = "csharp")]
mod dependencies;
#[cfg(feature = "csharp")]
pub(super) use dependencies::publish as publish_dependency_frontier;
#[cfg(feature = "csharp")]
pub(crate) use dependencies::{DependencyEvidence, acquire_csharp_dependencies};
mod execution;
#[cfg(all(test, feature = "rust"))]
pub(super) use execution::hydrate_workspace_index;
pub(super) use execution::{hydrate_workspace_index_for, hydrate_workspace_index_for_acquisition};

mod filesystem;
#[cfg(feature = "csharp")]
pub(crate) use filesystem::configured_roots as dependency_roots;
#[cfg(any(feature = "csharp", all(test, feature = "rust")))]
pub(crate) use filesystem::source_inventory;
use filesystem::{RootInventoryManifest, configured_roots, deduplicate_roots, root_key, scan};
pub(crate) use filesystem::{effective_refresh_roots, source_candidates_containing};

mod invalidation;
#[cfg(all(test, feature = "rust"))]
pub(crate) use invalidation::invalidate_discovery_for_root;
pub(crate) use invalidation::{
    RefreshReconciliation, invalidate_discovery_for_edited_path,
    reconcile_external_refresh_for_root,
};

mod publication;
#[cfg(feature = "angular")]
pub(super) use publication::compile_candidates_for as revalidate_module_owners;
mod readiness;
mod report;
#[cfg(all(test, feature = "rust"))]
pub(crate) use publication::compile_candidate;
#[cfg(all(test, feature = "rust"))]
pub(crate) use publication::supersede_test_batch_once;
#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/semantic_compilation_parallel.rs"]
mod semantic_compilation_parallel_tests;

#[cfg(all(test, feature = "rust"))]
pub(crate) use filesystem::{
    TraversalStats, last_test_traversal_stats, reset_test_scan_calls, reset_total_test_scan_calls,
    test_scan_calls, total_test_scan_calls,
};

// Test-only discovery instrumentation (project-search injection, traversal
// counters). Kept in its own module so this file stays inside the active-file
// ceiling; every name the regressions import from
// `crate::mcp::tool_handlers::hydration` is re-exported here unchanged.
#[cfg(all(test, feature = "rust"))]
mod test_support;

#[cfg(all(test, feature = "rust"))]
pub(crate) use test_support::*;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/acquisition_hydration.rs"]
mod acquisition_tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/acquisition_declarations.rs"]
mod declaration_acquisition_tests;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/hydration_single_flight.rs"]
mod single_flight_tests;

/// Diagnostic bound for the LLM-facing project-coverage projection.
///
/// The bound belongs to the RESPONSE projection (`query::diagnostics`), not to
/// the internal record: [HydrationReport] keeps every coverage entry so internal
/// debugging, telemetry and the discovery regressions can inspect the complete
/// result, while only the exceptional entries the caller actually sees count
/// against this cap. Exceeding it is reported structurally (`projects_truncated`)
/// rather than by dropping information silently.
pub(super) const HYDRATION_MAX_PROJECT_COVERAGE: usize = 16;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum HydrationRequirement {
    LegacyEdit,
    Semantic(crate::workspace::index::SemanticFidelity),
}

/// The authority obligation attached to one hydration request.
///
/// Provider discovery is advisory under both variants. `ProofBacked` enables
/// a capability-specific coverage contract after Clean inventory and
/// publication; `CleanAuthoritative` requires the filesystem-owned acquisition
/// path because no selective proof contract exists for that query family.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) enum HydrationAuthority {
    CleanAuthoritative,
    ProofBacked(crate::mcp::acquisition::AcquisitionTarget),
}

impl HydrationAuthority {
    pub(super) fn proof_target(&self) -> Option<&crate::mcp::acquisition::AcquisitionTarget> {
        match self {
            Self::CleanAuthoritative => None,
            Self::ProofBacked(target) => Some(target),
        }
    }
}

/// The complete result of one hydration discovery/compile cycle.
///
/// This is the INTERNAL record: it deliberately keeps the full provider,
/// completion, candidate and per-project coverage detail. What an LLM-facing
/// response serializes is a sparse projection of it
/// (`crate::mcp::tool_handlers::query::diagnostics`), which omits expected state
/// and surfaces only decision-relevant deviation.
#[derive(Clone, Default)]
pub(super) struct HydrationReport {
    pub(super) hydration_attempted: bool,
    pub(super) discovery_provider: &'static str,
    /// The single completion source of truth: `"completed"` iff discovery
    /// completed across every configured root and candidate publication did
    /// not fail. The former separate
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
    pub(super) acquisition_satisfaction: Option<AcquisitionSatisfaction>,
    pub(super) acquisition_outcome: Option<AcquisitionOutcome>,
}

#[derive(Clone, Serialize)]
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
    /// Advisory provider searches awaiting candidate-publication commit. These
    /// scopes may skip the same provider search later, but never satisfy a root.
    provider_completions: Vec<DiscoveryScope>,
    /// Clean filesystem acquisitions awaiting semantic-publication commit.
    /// Only these scopes may satisfy the root-selection boundary.
    root_completions: Vec<DiscoveryScope>,
    inventories: Vec<RootInventoryManifest>,
}

#[cfg(all(test, feature = "rust"))]
pub(super) fn is_hydration_eligible(query_type: &str, args: &Value) -> bool {
    matches!(
        query_type,
        "find_entities" | "forward_edges" | "reverse_edges" | "transitive_dependencies"
    ) && args["name"].as_str().is_some_and(|name| !name.is_empty())
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
    requirement: HydrationRequirement,
    authority: &HydrationAuthority,
) -> DiscoveryOutcome {
    #[cfg(all(test, feature = "rust"))]
    {
        if !has_test_project_search() {
            if let Some(paths) = test_hydration_candidates() {
                return DiscoveryOutcome {
                    candidates: paths,
                    project_coverage: Vec::new(),
                    provider: "cbm",
                    status: "completed",
                    attempted: true,
                    fallback_reason: None,
                    provider_completions: Vec::new(),
                    root_completions: Vec::new(),
                    inventories: Vec::new(),
                };
            }
        }
    }
    let filesystem_roots = configured_roots(state, workspace_root);
    state.prepare_cbm_for_consumer();
    let mut bridge_guard = state.graph_bridge_lock();
    let Some(bridge) = bridge_guard.as_mut() else {
        drop(bridge_guard);
        return filesystem_only_discovery(
            state,
            discovery,
            filesystem_roots,
            query_name,
            "cbm_unavailable",
            requirement,
        );
    };
    let projects = bridge.configured_projects();
    let provider_busy = readiness::provider_busy(bridge, &projects);
    let mut candidates = Vec::new();
    let mut coverage = Vec::new();
    let mut fallback_roots = Vec::new();
    let mut cbm_attempted = false;
    let mut saw_unavailable = false;
    let mut saw_search_failure = false;
    let mut saw_search_success = false;
    let mut provider_completions = Vec::new();

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
        // A successful provider search for this project/mode/name is not
        // repeated while the project's discovery generation is unchanged.
        // This cache entry reuses only advisory candidates; it never satisfies
        // the Clean-owned filesystem scope below. Skipping here also skips
        // `ensure_indexed_for` -- the only in-session producer of CBM dirty
        // state is `apply_edit`, which invalidates this scope.
        if discovery_is_complete(state, &scope, discovery, query_name, requirement) {
            cbm_attempted = true;
            saw_search_success = true;
            coverage.push(ProjectCoverage {
                project,
                status: "searched",
                readiness: Some("ready"),
                reason: None,
            });
            continue;
        }
        if readiness::skip_busy_provider(
            provider_busy,
            project.clone(),
            root.clone(),
            &mut fallback_roots,
            &mut coverage,
        ) {
            cbm_attempted = true;
            continue;
        }
        #[cfg(all(test, feature = "rust"))]
        if let Some(readiness) = test_project_readiness(&project) {
            let readiness = match readiness {
                TestProjectReadiness::Ready => "ready",
                TestProjectReadiness::StillIndexing => {
                    cbm_attempted = true;
                    fallback_roots.push(root);
                    coverage.push(ProjectCoverage {
                        project,
                        status: "skipped",
                        readiness: Some("still_indexing"),
                        reason: Some("still_indexing"),
                    });
                    continue;
                }
                TestProjectReadiness::Failed => {
                    cbm_attempted = true;
                    saw_search_failure = true;
                    fallback_roots.push(root);
                    coverage.push(ProjectCoverage {
                        project,
                        status: "skipped",
                        readiness: Some("failed"),
                        reason: Some("provider_degraded"),
                    });
                    continue;
                }
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
                    saw_search_success = true;
                    if readiness == "ready" {
                        provider_completions.push(scope);
                    }
                    coverage.push(ProjectCoverage {
                        project,
                        status: "searched",
                        readiness: Some(readiness),
                        reason: None,
                    });
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
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                cbm_attempted = true;
                fallback_roots.push(root);
                coverage.push(ProjectCoverage {
                    project,
                    status: "skipped",
                    readiness: Some("still_indexing"),
                    reason: Some("still_indexing"),
                });
                continue;
            }
            Err(_) => {
                cbm_attempted = true;
                saw_search_failure = true;
                fallback_roots.push(root);
                coverage.push(ProjectCoverage {
                    project,
                    status: "skipped",
                    readiness: Some("failed"),
                    reason: Some("provider_degraded"),
                });
                continue;
            }
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
                saw_search_success = true;
                if readiness == "ready" {
                    provider_completions.push(scope);
                }
                coverage.push(ProjectCoverage {
                    project,
                    status: "searched",
                    readiness: Some(readiness),
                    reason: None,
                });
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

    // Provider candidates are additive hints. Root satisfaction comes only
    // from a compatible proof or Clean-owned acquisition, so every configured
    // root remains in the fallback set until its filesystem scope is current.
    match authority {
        HydrationAuthority::CleanAuthoritative | HydrationAuthority::ProofBacked(_) => {
            fallback_roots.extend(filesystem_roots);
        }
    }
    deduplicate_roots(&mut fallback_roots);

    // Filesystem fallback discovery, split into roots that still require a
    // scan and roots whose discovery already completed for this generation.
    // A cached root contributes no new candidates (its results were compiled
    // when it completed) but still counts as covered, so it is neither
    // re-scanned nor reported as un-covered.
    let (pending_roots, cached_roots) =
        pending_discovery_roots(state, discovery, query_name, fallback_roots, requirement);

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
                provider_completions,
                root_completions: Vec::new(),
                inventories: Vec::new(),
            };
        }
        // Every fallback root is already discovered for this generation, so no
        // scan runs: the previously discovered (and compiled) candidates are
        // still the complete answer for this target.
        let any_cbm_success = saw_search_success;
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
            provider_completions,
            root_completions: Vec::new(),
            inventories: Vec::new(),
        };
    }

    let scan = scan(state, &pending_roots, query_name);
    let root_completions = if scan.completed {
        discovery_scopes_for_roots(&pending_roots)
    } else {
        Vec::new()
    };
    candidates.extend(scan.candidates);
    let any_cbm_success = saw_search_success;
    let attempted = cbm_attempted || scan.attempted || cached_roots > 0;
    let filesystem_covers = scan.attempted || cached_roots > 0;
    let provider = match (any_cbm_success, filesystem_covers) {
        (true, true) => "cbm_and_filesystem",
        (true, false) => "cbm",
        (false, true) => "filesystem",
        (false, false) => "none",
    };
    let completed = scan.completed;
    let inventories = scan.inventories;
    DiscoveryOutcome {
        candidates,
        project_coverage: coverage,
        provider,
        status: discovery_status(attempted, completed, any_cbm_success),
        attempted,
        fallback_reason: Some(fallback_reason),
        provider_completions,
        root_completions,
        inventories,
    }
}

fn filesystem_only_discovery(
    state: &McpState,
    discovery: DiscoveryMode,
    roots: Vec<PathBuf>,
    query_name: &str,
    fallback_reason: &'static str,
    requirement: HydrationRequirement,
) -> DiscoveryOutcome {
    let (pending_roots, cached_roots) =
        pending_discovery_roots(state, discovery, query_name, roots, requirement);
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
                provider_completions: Vec::new(),
                root_completions: Vec::new(),
                inventories: Vec::new(),
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
            provider_completions: Vec::new(),
            root_completions: Vec::new(),
            inventories: Vec::new(),
        };
    }
    let scan = scan(state, &pending_roots, query_name);
    let root_completions = if scan.completed {
        discovery_scopes_for_roots(&pending_roots)
    } else {
        Vec::new()
    };
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
        provider_completions: Vec::new(),
        root_completions,
        inventories: scan.inventories,
    }
}

// Discovery-cache invalidation entry points live in `hydration/invalidation.rs`
// (called by `apply_edit` and explicit repository reindexing), and the cache
// integration in `hydration/cache.rs`.
