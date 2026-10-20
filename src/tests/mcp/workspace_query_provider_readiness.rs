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
fn still_indexing_skips_advisory_search_and_cannot_authorize_completion() {
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
        0,
        "a non-ready provider must not execute an advisory search"
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
fn still_indexing_uses_only_immediate_clean_acquisition() {
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
        "StillIndexing must proceed directly to filesystem acquisition"
    );
    assert_eq!(
        discovery_calls().len(),
        0,
        "StillIndexing must not execute the configured provider search"
    );
    assert_eq!(
        report.candidates_discovered, 1,
        "only the Clean-owned matching path reaches candidate selection"
    );
    assert_eq!(
        report.candidates_compiled, 1,
        "only the Clean-owned matching owner is compiled"
    );
    assert_eq!(
        report.discovery_provider, "filesystem",
        "a skipped provider search is not reported as candidate provenance"
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

    let shared_owner = state.semantic_owner_path(&shared.to_string_lossy());
    let index = state.workspace_index_read();
    assert!(
        !index
            .file_map()
            .contains_key(&state.semantic_owner_path(&cbm_only.to_string_lossy())),
        "a CBM-only hint cannot cross the boundary while the provider is indexing"
    );
    assert!(
        index.file_map().contains_key(&shared_owner),
        "the filesystem-discovered candidate must be compiled"
    );
    assert_eq!(
        index.file_map().len(),
        1,
        "the Clean owner is published once"
    );
}

#[test]
fn still_indexing_provider_scope_remains_retryable_after_becoming_ready() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = ResetProjectSearch;
    let root = tempfile::tempdir().expect("workspace");
    let matching = root.path().join("ProjectContext.ts");
    std::fs::write(&matching, "export class ProjectContext {}\n").expect("matching source");

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
    set_test_project_readiness(HashMap::from([(
        project.clone(),
        TestProjectReadiness::StillIndexing,
    )]));
    reset_test_scan_calls();

    let workspace_root = root.path().to_string_lossy();
    let cold = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&workspace_root),
    )
    .expect("Clean acquisition succeeds while provider indexes");
    assert!(cold.is_complete());
    assert!(discovery_calls().is_empty());

    set_test_project_readiness(HashMap::from([(project, TestProjectReadiness::Ready)]));
    let warm = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&workspace_root),
    )
    .expect("ready provider search remains retryable");

    assert!(warm.is_complete());
    assert_eq!(
        discovery_calls().len(),
        1,
        "ready transition must retry CBM"
    );
    assert_eq!(test_scan_calls(), 1, "warm Clean completion must be reused");
    assert_eq!(warm.candidates_discovered, 1);
    assert_eq!(
        warm.candidates_compiled, 0,
        "current owner is not recompiled"
    );
    assert_eq!(
        state.hydration_discovery_lock().completed_entries(),
        2,
        "ready provider and Clean root retain separate completion entries"
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
    assert_eq!(report.candidates_discovered, 1);
    assert_eq!(report.candidates_compiled, 1);
    assert_eq!(
        state.hydration_discovery_lock().completed_entries(),
        2,
        "provider search reuse and authoritative filesystem completion are separate scopes"
    );
}
