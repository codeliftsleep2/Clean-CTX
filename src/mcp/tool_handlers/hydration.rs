// workspace_query bounded-hydration support.

use crate::cbm::GraphNode;
use crate::mcp::McpState;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

pub(crate) const HYDRATION_MAX_CANDIDATES: usize = 5;

#[cfg(test)]
pub(crate) static TEST_HYDRATION_CANDIDATES: Mutex<Option<Vec<String>>> = Mutex::new(None);

#[cfg(test)]
pub(crate) type TestProjectSearchResult = Result<Vec<GraphNode>, String>;

#[cfg(test)]
struct TestProjectSearchConfig {
    owner: std::thread::ThreadId,
    results: HashMap<String, TestProjectSearchResult>,
}

#[cfg(test)]
static TEST_PROJECT_SEARCH_RESULTS: Mutex<Option<TestProjectSearchConfig>> = Mutex::new(None);

#[cfg(test)]
pub(crate) static TEST_SEARCHED_PROJECTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[cfg(all(test, feature = "rust"))]
pub(crate) fn set_test_project_search_results(results: HashMap<String, TestProjectSearchResult>) {
    *TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned") = Some(TestProjectSearchConfig {
        owner: std::thread::current().id(),
        results,
    });
    TEST_SEARCHED_PROJECTS
        .lock()
        .expect("TEST_SEARCHED_PROJECTS lock poisoned")
        .clear();
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
}

#[cfg(all(test, feature = "rust"))]
pub(crate) fn searched_projects() -> Vec<String> {
    TEST_SEARCHED_PROJECTS
        .lock()
        .expect("TEST_SEARCHED_PROJECTS lock poisoned")
        .clone()
}

pub(super) fn is_hydration_eligible(query_type: &str, args: &Value) -> bool {
    matches!(
        query_type,
        "find_entities" | "forward_edges" | "reverse_edges" | "transitive_dependencies"
    ) && args["name"].as_str().is_some_and(|name| !name.is_empty())
}

pub(super) fn hydrate_workspace_index(
    state: &McpState,
    query_name: &str,
    workspace_root: Option<&str>,
) -> (usize, usize) {
    let candidate_paths = discover_candidate_paths(state, query_name);
    let discovered = candidate_paths.len();
    let selected = select_candidates(state, candidate_paths, HYDRATION_MAX_CANDIDATES);
    let compiled = selected
        .iter()
        .filter(|path| compile_candidate(state, path, workspace_root))
        .count();
    (discovered, compiled)
}

fn discover_candidate_paths(state: &McpState, query_name: &str) -> Vec<String> {
    #[cfg(test)]
    {
        if !has_test_project_search() {
            if let Ok(injected) = TEST_HYDRATION_CANDIDATES.lock() {
                if let Some(paths) = injected.as_ref() {
                    return paths.clone();
                }
            }
        }
    }

    let mut bridge_guard = state.graph_bridge_lock();
    let Some(bridge) = bridge_guard.as_mut() else {
        return Vec::new();
    };
    let projects = bridge.configured_projects();
    let mut candidates = Vec::new();

    for (project, root) in projects {
        #[cfg(test)]
        if let Some(result) = test_project_search(&project) {
            if let Ok(nodes) = result {
                append_project_paths(&mut candidates, &root, nodes);
            }
            continue;
        }

        if !bridge.is_available() {
            continue;
        }
        if !matches!(
            bridge.ensure_indexed_for(&project),
            Ok(crate::cbm::bridge::IndexingStatus::Ready)
        ) {
            continue;
        }
        if let Ok(nodes) = bridge.search_in_project(&project, query_name) {
            append_project_paths(&mut candidates, &root, nodes);
        }
    }
    candidates
}

fn append_project_paths(candidates: &mut Vec<String>, root: &Path, nodes: Vec<GraphNode>) {
    candidates.extend(nodes.into_iter().filter_map(|node| {
        if node.file.is_empty() {
            return None;
        }
        let path = PathBuf::from(node.file);
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

#[cfg(test)]
fn test_project_search(project: &str) -> Option<TestProjectSearchResult> {
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
    Some(
        configured
            .results
            .get(project)
            .cloned()
            .unwrap_or_else(|| Ok(Vec::new())),
    )
}

#[cfg(test)]
fn has_test_project_search() -> bool {
    TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned")
        .as_ref()
        .is_some_and(|configured| configured.owner == std::thread::current().id())
}
