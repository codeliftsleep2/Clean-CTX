// workspace_query semantic hydration support.

use crate::mcp::McpState;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod filesystem;
use filesystem::{configured_roots, deduplicate_roots, root_key, scan};

#[cfg(all(test, feature = "rust"))]
use crate::cbm::GraphNode;
#[cfg(all(test, feature = "rust"))]
use std::collections::HashMap;
#[cfg(all(test, feature = "rust"))]
use std::sync::Mutex;

const HYDRATION_MAX_PROJECT_COVERAGE: usize = 16;

#[derive(Default)]
pub(super) struct HydrationReport {
    pub(super) hydration_attempted: bool,
    pub(super) discovery_provider: &'static str,
    pub(super) discovery_status: &'static str,
    pub(super) discovery_completed: bool,
    pub(super) fallback_occurred: bool,
    pub(super) fallback_reason: Option<&'static str>,
    pub(super) candidates_discovered: usize,
    pub(super) candidates_compiled: usize,
    pub(super) project_coverage: Vec<ProjectCoverage>,
    pub(super) project_coverage_truncated: bool,
}

#[derive(Serialize)]
pub(super) struct ProjectCoverage {
    project: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    readiness: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
}

#[cfg(all(test, feature = "rust"))]
pub(crate) static TEST_HYDRATION_CANDIDATES: Mutex<Option<Vec<String>>> = Mutex::new(None);

#[cfg(all(test, feature = "rust"))]
pub(crate) static TEST_PROJECT_HYDRATION_SERIALIZE: Mutex<()> = Mutex::new(());

#[cfg(all(test, feature = "rust"))]
pub(crate) type TestProjectSearchResult = Result<Vec<GraphNode>, String>;

#[cfg(all(test, feature = "rust"))]
#[derive(Clone, Copy)]
pub(crate) enum TestProjectReadiness {
    Ready,
    StillIndexing,
    Failed,
    Unavailable,
}

#[cfg(all(test, feature = "rust"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TestDiscoveryKind {
    Declaration,
    InboundReference,
}

#[cfg(all(test, feature = "rust"))]
struct TestProjectSearchConfig {
    owner: std::thread::ThreadId,
    results: HashMap<String, TestProjectSearchResult>,
    inbound_results: HashMap<String, TestProjectSearchResult>,
    readiness: HashMap<String, TestProjectReadiness>,
}

#[cfg(all(test, feature = "rust"))]
static TEST_PROJECT_SEARCH_RESULTS: Mutex<Option<TestProjectSearchConfig>> = Mutex::new(None);

#[cfg(all(test, feature = "rust"))]
pub(crate) static TEST_SEARCHED_PROJECTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[cfg(all(test, feature = "rust"))]
static TEST_DISCOVERY_CALLS: Mutex<Vec<(String, TestDiscoveryKind)>> = Mutex::new(Vec::new());

#[cfg(all(test, feature = "rust"))]
pub(crate) fn set_test_project_search_results(results: HashMap<String, TestProjectSearchResult>) {
    *TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned") = Some(TestProjectSearchConfig {
        owner: std::thread::current().id(),
        results,
        inbound_results: HashMap::new(),
        readiness: HashMap::new(),
    });
    TEST_SEARCHED_PROJECTS
        .lock()
        .expect("TEST_SEARCHED_PROJECTS lock poisoned")
        .clear();
    TEST_DISCOVERY_CALLS
        .lock()
        .expect("TEST_DISCOVERY_CALLS lock poisoned")
        .clear();
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn set_test_inbound_project_search_results(
    results: HashMap<String, TestProjectSearchResult>,
) {
    TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned")
        .as_mut()
        .expect("project search results configured first")
        .inbound_results = results;
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn set_test_project_readiness(readiness: HashMap<String, TestProjectReadiness>) {
    TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned")
        .as_mut()
        .expect("project search results configured first")
        .readiness = readiness;
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn clear_test_project_search_results() {
    *TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned") = None;
    TEST_SEARCHED_PROJECTS
        .lock()
        .expect("TEST_SEARCHED_PROJECTS lock poisoned")
        .clear();
    TEST_DISCOVERY_CALLS
        .lock()
        .expect("TEST_DISCOVERY_CALLS lock poisoned")
        .clear();
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn searched_projects() -> Vec<String> {
    TEST_SEARCHED_PROJECTS
        .lock()
        .expect("TEST_SEARCHED_PROJECTS lock poisoned")
        .clone()
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn discovery_calls() -> Vec<(String, TestDiscoveryKind)> {
    TEST_DISCOVERY_CALLS
        .lock()
        .expect("TEST_DISCOVERY_CALLS lock poisoned")
        .clone()
}

#[derive(Clone, Copy)]
enum DiscoveryKind {
    Declaration,
    InboundReference,
}

struct DiscoveryOutcome {
    candidates: Vec<String>,
    project_coverage: Vec<ProjectCoverage>,
    provider: &'static str,
    status: &'static str,
    completed: bool,
    attempted: bool,
    fallback_occurred: bool,
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
    let project_coverage_truncated = project_coverage.len() > HYDRATION_MAX_PROJECT_COVERAGE;
    project_coverage.truncate(HYDRATION_MAX_PROJECT_COVERAGE);
    HydrationReport {
        hydration_attempted: outcome.attempted,
        discovery_provider: outcome.provider,
        discovery_status: outcome.status,
        discovery_completed: outcome.completed,
        fallback_occurred: outcome.fallback_occurred,
        fallback_reason: outcome.fallback_reason,
        candidates_discovered: discovered,
        candidates_compiled: compiled,
        project_coverage,
        project_coverage_truncated,
    }
}

fn discovery_kind(query_type: &str) -> DiscoveryKind {
    match query_type {
        "reverse_edges" => DiscoveryKind::InboundReference,
        _ => DiscoveryKind::Declaration,
    }
}

fn discover_candidate_paths(
    state: &McpState,
    discovery: DiscoveryKind,
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
                        completed: true,
                        attempted: true,
                        fallback_occurred: false,
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
        return filesystem_only_discovery(state, filesystem_roots, query_name, "cbm_unavailable");
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
            DiscoveryKind::Declaration => bridge
                .search_in_project(&project, query_name)
                .map(|nodes| nodes.into_iter().map(|node| node.file).collect()),
            DiscoveryKind::InboundReference => {
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

    if fallback_roots.is_empty() {
        return DiscoveryOutcome {
            candidates,
            project_coverage: coverage,
            provider: "cbm",
            status: "completed",
            completed: true,
            attempted: cbm_attempted,
            fallback_occurred: false,
            fallback_reason: None,
        };
    }

    let fallback_reason = match (saw_unavailable, saw_search_failure) {
        (true, true) => "cbm_partial_failure",
        (true, false) => "cbm_unavailable",
        (false, true) => "cbm_discovery_failed",
        (false, false) => "cbm_scope_unavailable",
    };
    let scan = scan(state, &fallback_roots, query_name);
    candidates.extend(scan.candidates);
    let any_cbm_success = !successful_roots.is_empty();
    let attempted = cbm_attempted || scan.attempted;
    let provider = match (any_cbm_success, scan.attempted) {
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
        completed,
        attempted,
        fallback_occurred: true,
        fallback_reason: Some(fallback_reason),
    }
}

fn filesystem_only_discovery(
    state: &McpState,
    roots: Vec<PathBuf>,
    query_name: &str,
    fallback_reason: &'static str,
) -> DiscoveryOutcome {
    let scan = scan(state, &roots, query_name);
    let provider = if scan.attempted { "filesystem" } else { "none" };
    DiscoveryOutcome {
        candidates: scan.candidates,
        project_coverage: Vec::new(),
        provider,
        status: discovery_status(scan.attempted, scan.completed, false),
        completed: scan.completed,
        attempted: scan.attempted,
        fallback_occurred: true,
        fallback_reason: Some(if scan.attempted {
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

#[cfg(all(test, feature = "rust"))]
fn test_project_search(project: &str, discovery: DiscoveryKind) -> Option<TestProjectSearchResult> {
    let results = TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned");
    let configured = results.as_ref()?;
    if configured.owner != std::thread::current().id() {
        return None;
    }
    TEST_SEARCHED_PROJECTS
        .lock()
        .expect("TEST_SEARCHED_PROJECTS lock poisoned")
        .push(project.to_string());
    let test_discovery = match discovery {
        DiscoveryKind::Declaration => TestDiscoveryKind::Declaration,
        DiscoveryKind::InboundReference => TestDiscoveryKind::InboundReference,
    };
    TEST_DISCOVERY_CALLS
        .lock()
        .expect("TEST_DISCOVERY_CALLS lock poisoned")
        .push((project.to_string(), test_discovery));
    let results = match discovery {
        DiscoveryKind::Declaration => &configured.results,
        DiscoveryKind::InboundReference => &configured.inbound_results,
    };
    Some(
        results
            .get(project)
            .cloned()
            .unwrap_or_else(|| Ok(Vec::new())),
    )
}

#[cfg(all(test, feature = "rust"))]
fn test_project_readiness(project: &str) -> Option<TestProjectReadiness> {
    let results = TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned");
    let configured = results.as_ref()?;
    if configured.owner != std::thread::current().id() {
        return None;
    }
    Some(
        configured
            .readiness
            .get(project)
            .copied()
            .unwrap_or(TestProjectReadiness::Ready),
    )
}

#[cfg(all(test, feature = "rust"))]
fn has_test_project_search() -> bool {
    TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned")
        .as_ref()
        .is_some_and(|configured| configured.owner == std::thread::current().id())
}
