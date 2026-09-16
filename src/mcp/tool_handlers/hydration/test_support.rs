// Test-only discovery instrumentation for workspace_query hydration.
//
// Declared by its parent under `#[cfg(all(test, feature = "rust"))]`, so none
// of it is ever compiled into release artifacts. It holds the existing
// project-search / readiness injection slot used by the hydration regressions
// (`src/tests/mcp/workspace_query_*.rs`) plus the serialization mutex that
// keeps those process-wide statics from interfering across threads.
//
// Split out of `hydration.rs` so the hydration-discovery-cache integration
// could be added without pushing that file past the active-file ceiling. Every
// name the tests import from `crate::mcp::tool_handlers::hydration` is
// re-exported there, so test import paths are unchanged.

use crate::cbm::GraphNode;
use crate::mcp::discovery_cache::DiscoveryMode;
use std::collections::HashMap;
use std::sync::Mutex;

/// Candidate paths injected by tests in place of real CBM discovery.
pub(crate) static TEST_HYDRATION_CANDIDATES: Mutex<Option<Vec<String>>> = Mutex::new(None);

/// Serializes the hydration regressions (these statics are process-wide).
pub(crate) static TEST_PROJECT_HYDRATION_SERIALIZE: Mutex<()> = Mutex::new(());

pub(crate) type TestProjectSearchResult = Result<Vec<GraphNode>, String>;

#[derive(Clone, Copy)]
pub(crate) enum TestProjectReadiness {
    Ready,
    StillIndexing,
    Failed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TestDiscoveryKind {
    Declaration,
    InboundReference,
}

pub(crate) struct TestProjectSearchConfig {
    pub(crate) owner: std::thread::ThreadId,
    pub(crate) results: HashMap<String, TestProjectSearchResult>,
    pub(crate) inbound_results: HashMap<String, TestProjectSearchResult>,
    pub(crate) readiness: HashMap<String, TestProjectReadiness>,
}

pub(crate) static TEST_PROJECT_SEARCH_RESULTS: Mutex<Option<TestProjectSearchConfig>> =
    Mutex::new(None);

pub(crate) static TEST_SEARCHED_PROJECTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

static TEST_DISCOVERY_CALLS: Mutex<Vec<(String, TestDiscoveryKind)>> = Mutex::new(Vec::new());

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

pub(crate) fn set_test_project_readiness(readiness: HashMap<String, TestProjectReadiness>) {
    TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned")
        .as_mut()
        .expect("project search results configured first")
        .readiness = readiness;
}

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

pub(crate) fn searched_projects() -> Vec<String> {
    TEST_SEARCHED_PROJECTS
        .lock()
        .expect("TEST_SEARCHED_PROJECTS lock poisoned")
        .clone()
}

pub(crate) fn discovery_calls() -> Vec<(String, TestDiscoveryKind)> {
    TEST_DISCOVERY_CALLS
        .lock()
        .expect("TEST_DISCOVERY_CALLS lock poisoned")
        .clone()
}

/// Injected project search for `project`.
///
/// Returns `None` when no injection is configured for this thread (the real CBM
/// path then runs). A configured-but-absent project yields a successful empty
/// result, which is how the regressions exercise "zero candidates discovered".
pub(crate) fn test_project_search(
    project: &str,
    discovery: DiscoveryMode,
) -> Option<TestProjectSearchResult> {
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
        DiscoveryMode::Declaration => TestDiscoveryKind::Declaration,
        DiscoveryMode::InboundReference => TestDiscoveryKind::InboundReference,
    };
    TEST_DISCOVERY_CALLS
        .lock()
        .expect("TEST_DISCOVERY_CALLS lock poisoned")
        .push((project.to_string(), test_discovery));
    let results = match discovery {
        DiscoveryMode::Declaration => &configured.results,
        DiscoveryMode::InboundReference => &configured.inbound_results,
    };
    Some(
        results
            .get(project)
            .cloned()
            .unwrap_or_else(|| Ok(Vec::new())),
    )
}

/// Injected readiness for `project` (defaults to `Ready`).
pub(crate) fn test_project_readiness(project: &str) -> Option<TestProjectReadiness> {
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

/// True when this thread has an injected project-search configuration.
pub(crate) fn has_test_project_search() -> bool {
    TEST_PROJECT_SEARCH_RESULTS
        .lock()
        .expect("TEST_PROJECT_SEARCH_RESULTS lock poisoned")
        .as_ref()
        .is_some_and(|configured| configured.owner == std::thread::current().id())
}
