use super::{CbmClient, a2_support as fixture};
use crate::cbm::bridge::GraphBridge;
use std::sync::{Arc, Mutex};

fn with_client(mode: &str, body: impl FnOnce(&mut CbmClient)) {
    let root = tempfile::tempdir().unwrap();
    let mut client =
        fixture::launch_for_admission(root.path(), &root.path().join("wire.json"), mode);
    client.admit().unwrap();
    body(&mut client);
}
pub(super) fn with_bridge(mode: &str, body: impl FnOnce(&mut GraphBridge)) {
    let root = tempfile::tempdir().unwrap();
    let mut client =
        fixture::launch_for_admission(root.path(), &root.path().join("wire.json"), mode);
    client.admit().unwrap();
    let mut bridge = GraphBridge::try_create(
        &crate::cbm::config::CbmConfig {
            enabled: false,
            ..Default::default()
        },
        root.path(),
    );
    bridge.status = crate::cbm::config::CbmStatus::Available;
    bridge.client = Arc::new(Mutex::new(Some(client)));
    bridge.project_root = root.path().canonicalize().unwrap();
    bridge.project = Some("test-project".into());
    bridge
        .project_paths
        .insert("test-project".into(), bridge.project_root.clone());
    bridge
        .project_ids
        .insert(bridge.project_root.clone(), "test-project".into());
    body(&mut bridge);
}
#[test]
fn cbm_provider_state_query_more_is_not_full_success() {
    with_client("b2-more", |c| {
        assert!(c.query_graph("query", "test-project").is_err())
    });
}
#[test]
fn cbm_provider_state_query_gte_is_not_exact_success() {
    with_client("b2-gte", |c| {
        assert!(c.query_graph("query", "test-project").is_err())
    });
}
#[test]
fn cbm_provider_state_query_budget_truncation_is_not_full_success() {
    with_client("b2-truncated", |c| {
        assert!(c.query_graph("query", "test-project").is_err())
    });
}
#[test]
fn cbm_provider_state_query_count_mismatch_is_not_full_success() {
    with_client("b2-count", |c| {
        assert!(c.query_graph("query", "test-project").is_err())
    });
}
#[test]
fn cbm_provider_state_search_more_is_not_full_success() {
    with_client("b2-more", |c| {
        assert!(c.search_graph("run", "test-project", None).is_err())
    });
}
#[test]
fn cbm_provider_state_trace_budget_truncation_is_not_empty_success() {
    with_client("b2-truncated", |c| {
        assert!(
            c.trace_path("fixture.run", "both", "test-project", None)
                .is_err()
        )
    });
}
#[test]
fn cbm_provider_state_running_job_is_not_index_completion() {
    with_bridge("b2-running", |b| assert!(b.index_repository().is_err()));
}
#[test]
fn cbm_provider_state_unknown_index_reply_is_not_completion() {
    with_bridge("b2-unknown-index", |b| {
        assert!(b.index_repository().is_err())
    });
}
#[test]
fn cbm_provider_state_degraded_index_is_not_clean_completion() {
    with_bridge("b2-degraded", |b| assert!(b.index_repository().is_err()));
}
#[test]
fn cbm_provider_state_wrong_project_cannot_complete_admitted_root() {
    with_bridge("b2-wrong-project", |b| {
        assert!(b.index_repository().is_err())
    });
}
#[test]
fn cbm_provider_state_running_job_does_not_clear_dirty_generation() {
    with_bridge("b2-running", |b| {
        let file = b.project_root.join("api.ts");
        b.mark_project_dirty(&file);
        assert!(b.reindex_for_file(&file, "fast").is_err());
        assert!(b.is_project_dirty("test-project"));
    });
}
#[test]
fn cbm_provider_state_real_terminal_index_still_succeeds() {
    with_bridge("b2-good", |b| assert!(b.index_repository().is_ok()));
}

#[test]
fn cbm_provider_state_unknown_page_cannot_claim_completeness() {
    with_client("b2-unknown-page", |c| {
        assert!(c.query_graph("query", "test-project").is_err())
    });
}
#[test]
fn cbm_provider_state_query_success_does_not_erase_running_job() {
    with_client("b2-running", |c| {
        assert!(c.index_project(".", "fast", "test-project").is_err());
        c.query_graph("query", "test-project").unwrap();
        assert_eq!(
            c.provider_observation("test-project", "index_repository")
                .unwrap()
                .metadata["state"],
            "running"
        );
    });
}
#[test]
fn cbm_provider_state_query_success_does_not_erase_failed_job_or_prior_serving() {
    with_client("b2-failed-index", |c| {
        assert!(c.index_project(".", "fast", "test-project").is_err());
        c.query_graph("query", "test-project").unwrap();
        let job = c
            .provider_observation("test-project", "index_repository")
            .unwrap();
        assert_eq!(job.metadata["serving_index_preserved"], true);
        assert!(job.failure.is_some());
    });
}
#[test]
fn cbm_provider_state_frontend_retirement_invalidates_observations() {
    with_client("b2-good", |c| {
        c.query_graph("query", "test-project").unwrap();
        assert!(
            c.provider_observation("test-project", "query_graph")
                .is_some()
        );
        c.retire_frontend("B2 test");
        assert!(
            c.provider_observation("test-project", "query_graph")
                .is_none()
        );
    });
}
#[test]
fn cbm_provider_state_native_candidates_preserve_best_effort_unknown_coverage() {
    with_bridge("b2-good", |b| {
        assert_eq!(b.search_in_project("test-project", "run").unwrap().len(), 1);
        let client = b.client.lock().unwrap();
        let client = client.as_ref().unwrap();
        let coverage = client
            .provider_observation("test-project", "check_index_coverage")
            .unwrap();
        assert_eq!(coverage.metadata["metadata"]["generation_matches"], false);
        assert_eq!(coverage.metadata["metadata"]["recording_status"], "unknown");
        assert_eq!(coverage.metadata["paths"][0]["requested_path"], "api.ts");
        assert!(
            client
                .provider_observation("test-project", "index_status")
                .is_some()
        );
    });
}
#[test]
fn cbm_provider_state_native_candidate_wrong_serving_root_fails() {
    with_bridge("b2-wrong-root", |b| {
        assert!(b.search_in_project("test-project", "run").is_err())
    });
}
#[test]
fn cbm_provider_state_native_candidate_partial_coverage_fails() {
    with_bridge("b2-coverage-more", |b| {
        assert!(b.search_in_project("test-project", "run").is_err())
    });
}
#[test]
fn cbm_provider_state_partial_query_retains_opaque_continuation() {
    with_client("b2-more", |c| {
        assert!(c.query_graph("query", "test-project").is_err());
        let result = c
            .provider_observation("test-project", "query_graph")
            .unwrap();
        assert_eq!(result.metadata["next_cursor"], "opaque");
        assert_eq!(result.metadata["pages_consumed"], 1);
    });
}

fn proxy_refresh_case(mode: &str, expect_retained: bool) {
    with_bridge(mode, |bridge| {
        let root = bridge.project_root.clone();
        let source = root.join("api.ts");
        std::fs::write(&source, "export function run() {}\n").unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config.clone());
        state.read_source(&source.to_string_lossy()).unwrap();
        bridge.indexing_state.lock().unwrap().insert(
            "test-project".into(),
            crate::cbm::bridge::IndexingState::Complete,
        );
        let replacement = GraphBridge::try_create(&config.cbm, &root);
        *state.graph_bridge_lock() = Some(std::mem::replace(bridge, replacement));
        crate::protocol::captured_responses().clear();
        crate::mcp::tool_dispatch::dispatch_tools_call(
            &serde_json::json!(992),
            "cbm_proxy",
            &serde_json::json!({"arguments":{"cbm_tool":"index_repository","workspaceRoot":root,
                "parameters":{"repo_path":root,"mode":"fast","project":"test-project"}}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(
            response.get("result").is_some(),
            "MCP invocation must reach proxy: {response}"
        );
        assert_eq!(
            state
                .cached_source_snapshot(&source.to_string_lossy())
                .is_some(),
            expect_retained,
            "only published indexing may refresh native source authority"
        );
    });
}
#[test]
fn cbm_provider_state_proxy_running_job_does_not_refresh_native_source() {
    proxy_refresh_case("b2-running", true);
}
#[test]
fn cbm_provider_state_proxy_degraded_index_does_not_refresh_native_source() {
    proxy_refresh_case("b2-degraded", true);
}
#[test]
fn cbm_provider_state_proxy_published_index_still_refreshes_native_source() {
    proxy_refresh_case("b2-good", false);
}
