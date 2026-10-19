//! Structural RED for provider readiness versus aggregate hydration completeness.

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    TestProjectReadiness, clear_test_project_search_results, discovery_calls,
    fail_test_publication_once, hydrate_workspace_index, reset_test_scan_calls,
    set_test_inbound_project_search_results, set_test_project_readiness,
    set_test_project_search_results, test_scan_calls,
};
use std::collections::HashMap;

static TEST_SERIALIZE: &std::sync::Mutex<()> =
    &crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;

struct ResetProjectSearch;

impl Drop for ResetProjectSearch {
    fn drop(&mut self) {
        clear_test_project_search_results();
    }
}

fn node(file: impl Into<String>) -> GraphNode {
    GraphNode {
        id: "candidate".into(),
        label: "Class".into(),
        name: "ProjectContext".into(),
        file: file.into(),
        properties: HashMap::new(),
    }
}

#[test]
fn red_successful_still_indexing_search_cannot_authorize_complete_hydration() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = ResetProjectSearch;
    let root = tempfile::tempdir().expect("workspace");
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create(&bridge_config, root.path()));
    let project = cbm_project_slug(&root.path().canonicalize().expect("canonical root"));
    let candidate = root.path().join("ProjectContext.ts");
    std::fs::write(&candidate, "export class ProjectContext {}\n").expect("candidate source");
    set_test_project_search_results(HashMap::from([(
        project.clone(),
        Ok(vec![node("ProjectContext.ts")]),
    )]));
    set_test_inbound_project_search_results(HashMap::from([(project.clone(), Ok(Vec::new()))]));
    set_test_project_readiness(HashMap::from([(
        project.clone(),
        TestProjectReadiness::StillIndexing,
    )]));
    fail_test_publication_once(&candidate.to_string_lossy());

    let workspace_root = root.path().to_string_lossy().into_owned();
    let first = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&workspace_root),
    )
    .expect("successful provider request returns a hydration report");
    let second = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&workspace_root),
    )
    .expect("non-ready provider remains retryable");

    assert_eq!(
        discovery_calls().len(),
        2,
        "a non-ready provider scope must be retried rather than cached"
    );
    assert!(
        !first.is_complete(),
        "non-ready CBM cannot authorize completion when Clean publication fails"
    );
    assert!(
        second.is_complete(),
        "completed Clean fallback may authorize completion independently of CBM readiness"
    );
    assert_eq!(
        state.hydration_discovery_lock().completed_entries(),
        1,
        "only the successful Clean filesystem scope may be cached as complete"
    );
}

#[test]
fn still_indexing_cbm_candidates_are_unioned_with_filesystem_fallback() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = ResetProjectSearch;
    let root = tempfile::tempdir().expect("workspace");
    let cbm_only = root.path().join("CbmHint.ts");
    let shared = root.path().join("FilesystemMatch.ts");
    std::fs::write(&cbm_only, "export class AdvisoryOnly {}\n").expect("CBM-only candidate");
    std::fs::write(&shared, "export class ProjectContext {}\n").expect("shared candidate");

    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create(&bridge_config, root.path()));
    let project = cbm_project_slug(&root.path().canonicalize().expect("canonical root"));
    set_test_project_search_results(HashMap::from([(
        project.clone(),
        Ok(vec![node("CbmHint.ts"), node("FilesystemMatch.ts")]),
    )]));
    set_test_inbound_project_search_results(HashMap::from([(project.clone(), Ok(Vec::new()))]));
    set_test_project_readiness(HashMap::from([(
        project,
        TestProjectReadiness::StillIndexing,
    )]));
    reset_test_scan_calls();

    let report = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&root.path().to_string_lossy()),
    )
    .expect("non-ready CBM and filesystem fallback hydrate successfully");

    assert_eq!(
        test_scan_calls(),
        1,
        "successful StillIndexing CBM search must not suppress filesystem fallback"
    );
    assert_eq!(
        report.candidates_discovered, 3,
        "two advisory CBM paths plus one filesystem match form the candidate multiset"
    );
    assert_eq!(
        report.candidates_compiled, 2,
        "the shared path is deduplicated while the CBM-only hint remains usable"
    );
    assert_eq!(
        report.discovery_provider, "cbm_and_filesystem",
        "diagnostics must retain both candidate providers"
    );
    assert!(
        report.is_complete(),
        "completed Clean fallback authorizes completion without CBM readiness"
    );
    assert_eq!(
        state.hydration_discovery_lock().completed_entries(),
        1,
        "only the successful filesystem fallback may record completion"
    );

    let cbm_owner = state.semantic_owner_path(&cbm_only.to_string_lossy());
    let shared_owner = state.semantic_owner_path(&shared.to_string_lossy());
    let index = state.workspace_index_read();
    assert!(
        index.file_map().contains_key(&cbm_owner),
        "the CBM-only advisory candidate must be compiled"
    );
    assert!(
        index.file_map().contains_key(&shared_owner),
        "the filesystem-discovered candidate must be compiled"
    );
    assert_eq!(
        index.file_map().len(),
        2,
        "the shared path is published once"
    );
}

#[test]
fn ready_cbm_candidates_do_not_suppress_authoritative_filesystem_acquisition() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = ResetProjectSearch;
    let root = tempfile::tempdir().expect("workspace");
    let candidate = root.path().join("ProjectContext.ts");
    std::fs::write(&candidate, "export class ProjectContext {}\n").expect("candidate source");

    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create(&bridge_config, root.path()));
    let project = cbm_project_slug(&root.path().canonicalize().expect("canonical root"));
    set_test_project_search_results(HashMap::from([(
        project.clone(),
        Ok(vec![node("ProjectContext.ts")]),
    )]));
    set_test_inbound_project_search_results(HashMap::from([(project.clone(), Ok(Vec::new()))]));
    set_test_project_readiness(HashMap::from([(project, TestProjectReadiness::Ready)]));
    reset_test_scan_calls();

    let report = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&root.path().to_string_lossy()),
    )
    .expect("ready CBM hydration succeeds");

    assert_eq!(
        test_scan_calls(),
        1,
        "ready provider success is advisory and cannot satisfy the root"
    );
    assert!(
        report.is_complete(),
        "completed Clean acquisition authorizes completion"
    );
    assert_eq!(report.discovery_provider, "cbm_and_filesystem");
    assert_eq!(report.candidates_discovered, 2);
    assert_eq!(report.candidates_compiled, 1);
    assert_eq!(
        state.hydration_discovery_lock().completed_entries(),
        2,
        "provider search reuse and authoritative filesystem completion are separate scopes"
    );
}
