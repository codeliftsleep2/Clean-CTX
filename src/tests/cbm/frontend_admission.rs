use super::{CbmClient, a2_support as fixture};
use crate::cbm::config::CbmStatus;
use serde_json::json;
use tempfile::TempDir;

fn client(root: &TempDir, mode: &str) -> CbmClient {
    fixture::launch_for_admission(root.path(), &root.path().join("admission.json"), mode)
}
fn requests(root: &TempDir) -> Vec<serde_json::Value> {
    std::fs::read_to_string(root.path().join("admission.requests"))
        .unwrap_or_default()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn cbm_a3_spawn_is_not_provider_available() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "normal");
    assert!(
        !client.status().is_available(),
        "spawn alone published Available"
    );
    assert!(
        client.call_tool_raw("query_graph", json!({})).is_err(),
        "unadmitted provider accepted tool work"
    );
}

#[test]
fn cbm_a3_successful_admission_requires_supported_wire_sequence() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "normal");
    fixture::admit(&mut client).unwrap();
    assert!(client.status().is_available());
    client.call_tool_raw("query_graph", json!({})).unwrap();
    let calls = requests(&root);
    assert_eq!(
        calls[0]["method"], "initialize",
        "admission skipped initialize"
    );
    assert_eq!(calls[0]["params"]["protocolVersion"], "2024-11-05");
    assert_eq!(calls[1]["method"], "notifications/initialized");
    assert!(calls[1].get("id").is_none());
    assert_eq!(calls[2]["method"], "tools/list");
    assert_eq!(calls[2]["params"], json!({}));
    assert_eq!(calls[3]["method"], "tools/call");
    assert_eq!(calls[0]["id"], 1);
    assert_eq!(calls[2]["id"], 2);
    assert_eq!(calls[3]["id"], 3);
}

#[test]
fn cbm_a3_wrong_initialize_id_cannot_admit() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a3-wrong-id");
    assert!(
        fixture::admit(&mut client).is_err(),
        "wrong initialize ID admitted provider"
    );
    assert!(!client.status().is_available());
    assert_eq!(fixture::owned_resources(&client), 0);
}

#[test]
fn cbm_a3_notification_alone_cannot_admit() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a3-notify-only");
    assert!(
        fixture::admit(&mut client).is_err(),
        "notification alone admitted provider"
    );
    assert!(!client.status().is_available());
}

#[test]
fn cbm_a3_startup_refusal_is_preserved_and_retires_only_owned_frontend() {
    let root = TempDir::new().unwrap();
    let mut refused = client(&root, "a3-startup-refusal");
    let pid = fixture::child_id(&refused);
    let mut other = fixture::launch(root.path(), &root.path().join("other.json"), "normal", None)
        .0
        .unwrap()
        .unwrap();
    let result = fixture::admit(&mut refused);
    assert!(
        result.is_err(),
        "structured startup refusal admitted provider"
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("provider admission refused")
    );
    assert!(!refused.status().is_available());
    assert_eq!(fixture::owned_resources(&refused), 0);
    fixture::assert_process_gone(pid);
    assert!(other.call_tool_raw("query_graph", json!({})).is_ok());
}

#[test]
fn cbm_a3_frontend_exit_during_initialize_cannot_admit() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a3-exit-init");
    assert!(
        fixture::admit(&mut client).is_err(),
        "frontend exit admitted provider"
    );
    assert_eq!(fixture::owned_resources(&client), 0);
}

#[test]
fn cbm_a3_each_required_tool_absence_prevents_usability() {
    for tool in [
        "index_repository",
        "search_graph",
        "query_graph",
        "trace_path",
        "get_architecture",
        "list_projects",
        "detect_changes",
    ] {
        let root = TempDir::new().unwrap();
        let mut client = client(&root, &format!("a3-missing-{tool}"));
        assert!(
            fixture::admit(&mut client).is_err(),
            "missing {tool} admitted provider"
        );
        assert!(!client.status().is_available());
        assert_eq!(fixture::owned_resources(&client), 0);
    }
}

#[test]
fn cbm_a3_invalid_initialize_contract_prevents_usability() {
    for mode in [
        "a3-bad-version",
        "a3-no-tools-capability",
        "a3-bad-identity",
    ] {
        let root = TempDir::new().unwrap();
        let mut client = client(&root, mode);
        assert!(
            fixture::admit(&mut client).is_err(),
            "invalid {mode} admitted provider"
        );
        assert!(!client.status().is_available());
    }
}

#[test]
fn cbm_a3_bridge_indexes_only_after_admission_and_capabilities() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("admission.json");
    let mut bridge = fixture::bridge_with_mode(root.path(), &[], &log, "normal");
    bridge.proxy_call("query_graph", json!({})).unwrap();
    let _guard = bridge.client.lock().unwrap();
    let calls = requests(&root);
    assert_eq!(
        calls[0]["method"], "initialize",
        "bridge indexed before admission"
    );
    let capability = calls
        .iter()
        .position(|call| call["method"] == "tools/list")
        .unwrap();
    assert!(
        calls
            .iter()
            .enumerate()
            .filter(|(_, call)| call["method"] == "tools/call")
            .all(|(index, _)| index > capability)
    );
}

#[test]
fn cbm_a3_failed_admission_cannot_publish_successful_empty_provider_result() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("admission.json");
    let mut bridge = fixture::bridge_with_mode(root.path(), &[], &log, "a3-missing-query_graph");
    assert!(
        bridge.proxy_call("query_graph", json!({})).is_err(),
        "unusable provider returned successful empty evidence"
    );
    assert!(!bridge.is_available());
    assert!(matches!(bridge.status(), CbmStatus::Degraded(_)));
    let calls = requests(&root);
    assert!(
        calls.iter().all(|call| call["method"] != "tools/call"),
        "failed admission allowed background indexing"
    );
}

#[test]
fn cbm_a3_baseline_notification_then_response_is_compatible() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a3-notify-success");
    fixture::admit(&mut client).unwrap();
    assert!(client.call_tool_raw("query_graph", json!({})).is_ok());
}

#[test]
fn cbm_a3_baseline_native_heuristics_remain_independent() {
    let root = TempDir::new().unwrap();
    let _bridge = fixture::bridge_with_mode(
        root.path(),
        &[],
        &root.path().join("admission.json"),
        "a3-missing-query_graph",
    );
    let decision = crate::mcp::heuristics::decide(
        "/project/User.cs",
        None,
        None,
        &crate::config::CleanCtxConfig::default(),
        &crate::ir::replay::ContextState::new(),
        "public class User { public string Name { get; set; } }",
        None,
        None,
    )
    .unwrap();
    assert!(!decision.cbm_informed);
}

#[path = "frontend_admission_exit.rs"]
mod exit_tests;
