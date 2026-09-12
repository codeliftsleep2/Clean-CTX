// workspace_query bounded-hydration support.

use crate::mcp::McpState;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[cfg(all(test, feature = "rust"))]
use crate::cbm::GraphNode;
#[cfg(all(test, feature = "rust"))]
use std::collections::HashMap;
#[cfg(all(test, feature = "rust"))]
use std::sync::Mutex;

pub(crate) const HYDRATION_MAX_CANDIDATES: usize = 5;
const HYDRATION_MAX_PROJECT_COVERAGE: usize = 16;

#[derive(Default)]
pub(super) struct HydrationReport {
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
    let (candidate_paths, mut project_coverage) =
        discover_candidate_paths(state, discovery, query_name);
    let discovered = candidate_paths.len();
    let selected = select_candidates(state, candidate_paths, HYDRATION_MAX_CANDIDATES);
    let compiled = selected
        .iter()
        .filter(|path| compile_candidate(state, path, workspace_root))
        .count();
    project_coverage.sort_by(|left, right| left.project.cmp(&right.project));
    let project_coverage_truncated = project_coverage.len() > HYDRATION_MAX_PROJECT_COVERAGE;
    project_coverage.truncate(HYDRATION_MAX_PROJECT_COVERAGE);
    HydrationReport {
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
) -> (Vec<String>, Vec<ProjectCoverage>) {
    #[cfg(all(test, feature = "rust"))]
    {
        if !has_test_project_search() {
            if let Ok(injected) = TEST_HYDRATION_CANDIDATES.lock() {
                if let Some(paths) = injected.as_ref() {
                    return (paths.clone(), Vec::new());
                }
            }
        }
    }

    let mut bridge_guard = state.graph_bridge_lock();
    let Some(bridge) = bridge_guard.as_mut() else {
        return (Vec::new(), Vec::new());
    };
    let projects = bridge.configured_projects();
    let mut candidates = Vec::new();
    let mut coverage = Vec::new();

    for configured_root in &state.config.additional_roots {
        let resolved = bridge.resolve_project_id(configured_root);
        if !projects.iter().any(|(project, _)| project == &resolved) {
            coverage.push(ProjectCoverage {
                project: resolved,
                status: "skipped",
                readiness: None,
                reason: Some("additional_root_not_registered"),
            });
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
                    coverage.push(ProjectCoverage {
                        project,
                        status: "skipped",
                        readiness: None,
                        reason: Some("cbm_unavailable"),
                    });
                    continue;
                }
            };
            let result =
                test_project_search(&project, discovery).expect("configured project search result");
            match result {
                Ok(nodes) => {
                    append_project_paths(&mut candidates, &root, nodes);
                    coverage.push(ProjectCoverage {
                        project,
                        status: "searched",
                        readiness: Some(readiness),
                        reason: None,
                    });
                }
                Err(_) => coverage.push(ProjectCoverage {
                    project,
                    status: "search_failed",
                    readiness: Some(readiness),
                    reason: Some("search_failed"),
                }),
            }
            continue;
        }

        if !bridge.is_available() {
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
                coverage.push(ProjectCoverage {
                    project,
                    status: "searched",
                    readiness: Some(readiness),
                    reason: None,
                });
            }
            Err(_) => coverage.push(ProjectCoverage {
                project,
                status: "search_failed",
                readiness: Some(readiness),
                reason: Some("search_failed"),
            }),
        }
    }
    (candidates, coverage)
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

fn select_candidates(state: &McpState, candidates: Vec<String>, max: usize) -> Vec<String> {
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
    selected.truncate(max);
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
