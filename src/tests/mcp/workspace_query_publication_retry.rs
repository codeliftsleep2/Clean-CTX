// Regression for discovery completion across candidate publication failure.

use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::mcp::tool_handlers::hydration::{
    HydrationRequirement, TEST_PROJECT_HYDRATION_SERIALIZE, TestDiscoveryKind,
    clear_test_project_search_results, discovery_calls, fail_test_publication_once,
    hydrate_workspace_index_for, set_test_project_search_results,
};
use crate::workspace::index::SemanticFidelity;
use std::collections::HashMap;
use std::fs;

fn node(file: &str) -> GraphNode {
    GraphNode {
        id: file.into(),
        label: "Class".into(),
        name: "RetryTarget".into(),
        file: file.into(),
        properties: HashMap::new(),
    }
}

#[test]
fn failed_candidate_publication_remains_retryable_after_successful_discovery() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempfile::TempDir::new().unwrap();
    let current = root.path().join("current.ts");
    let retry = root.path().join("retry.ts");
    fs::write(
        &current,
        "export class Current {} // CurrentSeed RetryTarget\n",
    )
    .unwrap();
    fs::write(&retry, "export class RetryTarget {}\n").unwrap();

    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &crate::cbm::config::CbmConfig {
            enabled: false,
            ..Default::default()
        },
        root.path(),
        &[],
    ));
    let slug = cbm_project_slug(&root.path().canonicalize().unwrap());
    let requirement = HydrationRequirement::Semantic(SemanticFidelity::Low);

    set_test_project_search_results(HashMap::from([(
        slug.clone(),
        Ok(vec![node("current.ts")]),
    )]));
    let seeded = hydrate_workspace_index_for(
        &state,
        "find_entities",
        "CurrentSeed",
        Some(&root.path().to_string_lossy()),
        requirement,
    )
    .unwrap();
    assert_eq!(seeded.candidates_compiled, 1);

    set_test_project_search_results(HashMap::from([(
        slug.clone(),
        Ok(vec![node("current.ts"), node("retry.ts")]),
    )]));
    fail_test_publication_once(&retry.to_string_lossy());

    let first = hydrate_workspace_index_for(
        &state,
        "find_entities",
        "RetryTarget",
        Some(&root.path().to_string_lossy()),
        requirement,
    )
    .unwrap();
    assert_eq!(first.candidates_discovered, 2);
    assert_eq!(first.candidates_compiled, 0);
    assert_eq!(first.discovery_status, "partial");
    assert!(
        state
            .workspace_index_read()
            .find_entities_by_name("RetryTarget")
            .is_empty(),
        "the forced failure must publish no RetryTarget projection"
    );

    let second = hydrate_workspace_index_for(
        &state,
        "find_entities",
        "RetryTarget",
        Some(&root.path().to_string_lossy()),
        requirement,
    )
    .unwrap();
    assert_eq!(second.candidates_discovered, 2);
    assert_eq!(second.candidates_compiled, 1);
    assert_eq!(second.discovery_status, "completed");
    assert!(
        !state
            .workspace_index_read()
            .find_entities_by_name("RetryTarget")
            .is_empty(),
        "the retried candidate must publish after the failure is removed"
    );

    let third = hydrate_workspace_index_for(
        &state,
        "find_entities",
        "RetryTarget",
        Some(&root.path().to_string_lossy()),
        requirement,
    )
    .unwrap();
    assert_eq!(third.candidates_discovered, 0);
    assert_eq!(third.candidates_compiled, 0);
    assert_eq!(
        discovery_calls()
            .into_iter()
            .filter(|(project, kind)| {
                project == &slug && *kind == TestDiscoveryKind::Declaration
            })
            .count(),
        2,
        "RetryTarget must rediscover once after failure, then hit its committed completion"
    );

    clear_test_project_search_results();
}
