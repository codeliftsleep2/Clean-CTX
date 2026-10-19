use super::{CbmClient, CbmError, a2_support as fixture};
use serde_json::{Value, json};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn log(root: &TempDir) -> std::path::PathBuf {
    root.path().join("recovery.json")
}
fn client(root: &TempDir, mode: &str) -> CbmClient {
    let mut client = fixture::launch_for_admission(root.path(), &log(root), mode);
    client.admit().unwrap();
    client
}
fn call(client: &mut CbmClient, root: &TempDir, mode: &str, tool: &str) -> Result<Value, CbmError> {
    fixture::with_options(&log(root), mode, || {
        client.call_tool(tool, json!({"project":"fixture"}))
    })
}
fn launches(root: &TempDir) -> usize {
    std::fs::read_to_string(log(root).with_extension("launches"))
        .unwrap_or_default()
        .lines()
        .count()
}
fn requests(root: &TempDir) -> Vec<Value> {
    std::fs::read_to_string(log(root).with_extension("requests"))
        .unwrap_or_default()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn tool_count(root: &TempDir, tool: &str) -> usize {
    requests(root)
        .iter()
        .filter(|r| r["params"]["name"] == tool)
        .count()
}

fn assert_same_directory(observed: &Value, expected: &Path) {
    let observed = Path::new(observed.as_str().expect("fixture cwd string"))
        .canonicalize()
        .expect("canonical observed fixture cwd");
    let expected = expected
        .canonicalize()
        .expect("canonical expected fixture cwd");
    assert_eq!(observed, expected);
}

#[test]
fn cbm_a4_observational_eof_replaces_once_and_replays_after_full_admission() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-eof-once");
    let old = fixture::child_id(&client);
    assert!(
        call(&mut client, &root, "a4-eof-once", "query_graph").is_ok(),
        "recoverable read did not replay on an admitted replacement"
    );
    assert_eq!(launches(&root), 2);
    assert_ne!(fixture::child_id(&client), old);
    fixture::assert_process_gone(old);
    let methods: Vec<_> = requests(&root)
        .iter()
        .map(|r| r["method"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        methods,
        [
            "initialize",
            "notifications/initialized",
            "tools/list",
            "tools/call",
            "initialize",
            "notifications/initialized",
            "tools/list",
            "tools/call"
        ]
    );
    let cwd: Value = serde_json::from_str(&std::fs::read_to_string(log(&root)).unwrap()).unwrap();
    assert_same_directory(&cwd["cwd"], root.path());
}
#[test]
fn cbm_a4_timeout_read_replaces_but_mutation_returns_original_without_replay() {
    for tool in ["query_graph", "index_repository", "future_tool"] {
        let root = TempDir::new().unwrap();
        let mut client = client(&root, "a4-timeout-once");
        let result = call(&mut client, &root, "a4-timeout-once", tool);
        if tool == "query_graph" {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(CbmError::Timeout(_))));
        }
        assert_eq!(
            launches(&root),
            2,
            "frontend recovery was confused with replay safety"
        );
        assert_eq!(
            tool_count(&root, tool),
            if tool == "query_graph" { 2 } else { 1 }
        );
        assert!(client.status().is_available());
    }
}
#[test]
fn cbm_a4_all_proven_observational_tools_have_bounded_replay() {
    for tool in [
        "search_graph",
        "query_graph",
        "trace_path",
        "get_architecture",
        "list_projects",
        "detect_changes",
    ] {
        let root = TempDir::new().unwrap();
        let mut client = client(&root, "a4-eof-once");
        assert!(
            call(&mut client, &root, "a4-eof-once", tool).is_ok(),
            "{tool}"
        );
        assert_eq!(launches(&root), 2);
        assert_eq!(tool_count(&root, tool), 2);
    }
}
#[test]
fn cbm_a4_mutation_internal_error_is_not_retried() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-rpc");
    assert!(matches!(
        call(&mut client, &root, "a4-rpc", "index_repository"),
        Err(CbmError::RpcError { code: -32603, .. })
    ));
    assert_eq!(
        tool_count(&root, "index_repository"),
        1,
        "uncertain mutation was duplicated"
    );
    assert_eq!(launches(&root), 1);
}
#[test]
fn cbm_a4_unknown_internal_error_defaults_to_no_replay() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-rpc");
    assert!(call(&mut client, &root, "a4-rpc", "future_tool").is_err());
    assert_eq!(
        tool_count(&root, "future_tool"),
        1,
        "unknown replay safety was assumed"
    );
}
#[test]
fn cbm_a4_baseline_surviving_observational_rpc_retry_budget_is_one() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-rpc");
    assert!(call(&mut client, &root, "a4-rpc", "query_graph").is_err());
    assert_eq!(tool_count(&root, "query_graph"), 2);
    assert_eq!(launches(&root), 1);
}
#[test]
fn cbm_a4_raw_boundary_replaces_without_changing_rpc_envelopes() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-eof-once");
    let result = fixture::with_options(&log(&root), "a4-eof-once", || {
        client.call_tool_raw("query_graph", json!({}))
    });
    assert!(result.is_ok());
    assert_eq!(launches(&root), 2);
}
#[test]
fn cbm_a4_refusal_and_protocol_poison_never_respawn() {
    for mode in ["a4-refusal", "a4-poison"] {
        let root = TempDir::new().unwrap();
        let mut client = client(&root, mode);
        let error = call(&mut client, &root, mode, "query_graph").unwrap_err();
        if mode == "a4-refusal" {
            assert!(matches!(error, CbmError::AdmissionError { .. }));
        } else {
            assert!(matches!(error, CbmError::ParseError(_)));
        }
        assert!(call(&mut client, &root, mode, "query_graph").is_err());
        assert_eq!(launches(&root), 1);
    }
}
#[test]
fn cbm_a4_replacement_refusal_preserves_original_and_recovery_cause() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-replacement-refusal");
    let error = call(&mut client, &root, "a4-replacement-refusal", "query_graph").unwrap_err();
    assert!(
        matches!(error, CbmError::ConnectionLost(_)),
        "original failure erased"
    );
    assert_eq!(launches(&root), 2, "replacement was never attempted");
    let status = format!("{:?}", client.status());
    assert!(
        status.contains("provider admission refused"),
        "replacement cause absent: {status}"
    );
    assert!(
        status.contains("original"),
        "original cause absent from recovery status"
    );
    assert!(!client.status().is_available());
    assert_eq!(fixture::owned_resources(&client), 0);
    for _ in 0..3 {
        assert!(call(&mut client, &root, "a4-replacement-refusal", "query_graph").is_err());
    }
    assert_eq!(launches(&root), 2, "permanent refusal respawn loop");
}
#[test]
fn cbm_a4_missing_replacement_capability_never_replays_or_loops() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-replacement-missing");
    assert!(call(&mut client, &root, "a4-replacement-missing", "query_graph").is_err());
    assert_eq!(launches(&root), 2);
    assert_eq!(tool_count(&root, "query_graph"), 1);
    assert!(format!("{:?}", client.status()).contains("required tool missing: query_graph"));
    assert!(call(&mut client, &root, "a4-replacement-missing", "query_graph").is_err());
    assert_eq!(launches(&root), 2);
}
#[test]
fn cbm_a4_repeated_transport_failure_stops_after_single_replacement() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-eof-always");
    client.circuit_cooldown_secs = 30;
    assert!(call(&mut client, &root, "a4-eof-always", "query_graph").is_err());
    assert_eq!(launches(&root), 2);
    assert_eq!(tool_count(&root, "query_graph"), 2);
    assert!(call(&mut client, &root, "a4-eof-always", "query_graph").is_err());
    assert_eq!(launches(&root), 2, "cooldown spawned another frontend");
}
#[test]
fn cbm_a4_half_open_uses_new_admitted_frontend() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "normal");
    let old = fixture::child_id(&client);
    client.consecutive_failures = client.max_consecutive_failures;
    client.degraded_since = Some(Instant::now() - Duration::from_secs(1));
    client.status = crate::cbm::config::CbmStatus::Degraded("circuit open".into());
    assert!(call(&mut client, &root, "normal", "query_graph").is_ok());
    assert_eq!(launches(&root), 2, "half-open revived old frontend");
    assert_ne!(fixture::child_id(&client), old);
    fixture::assert_process_gone(old);
}
#[test]
fn cbm_a4_concurrent_callers_share_one_recovery_owner() {
    let root = TempDir::new().unwrap();
    let client = Arc::new(Mutex::new(client(&root, "a4-eof-once")));
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let client = Arc::clone(&client);
            let root = &root;
            scope.spawn(move || {
                assert!(
                    call(
                        &mut client.lock().unwrap(),
                        root,
                        "a4-eof-once",
                        "query_graph"
                    )
                    .is_ok()
                )
            });
        }
    });
    assert_eq!(
        launches(&root),
        2,
        "serialized callers created extra replacements"
    );
}
#[test]
fn cbm_a4_baseline_explicit_retirement_cannot_be_revived_by_cooldown() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "normal");
    fixture::retire(&mut client);
    client.degraded_since = Some(Instant::now() - Duration::from_secs(100));
    assert!(!client.circuit_allows());
    assert!(call(&mut client, &root, "normal", "query_graph").is_err());
    assert_eq!(launches(&root), 1);
}

#[test]
fn cbm_a4_background_indexing_internal_error_does_not_duplicate_uncertain_job() {
    let root = TempDir::new().unwrap();
    let additional = TempDir::new().unwrap();
    let mut bridge = fixture::bridge_with_mode(
        root.path(),
        &[additional.path().to_path_buf()],
        &log(&root),
        "a4-rpc",
    );
    bridge.complete_provider_startup_and_start_additional_roots_for_test();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if bridge
            .indexing_state()
            .values()
            .any(|state| matches!(state, crate::cbm::bridge::IndexingState::Failed(_)))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "background indexing did not finish fixture failure"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        tool_count(&root, "index_repository"),
        1,
        "background job duplicated after uncertain RPC failure"
    );
    assert_eq!(launches(&root), 1);
}

#[test]
fn cbm_a4_baseline_native_heuristics_remain_usable_after_recovery_failure() {
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "a4-replacement-refusal");
    assert!(call(&mut client, &root, "a4-replacement-refusal", "query_graph").is_err());
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

#[test]
fn cbm_a4_registered_mcp_query_can_recover_after_circuit_cooldown() {
    let _serial = crate::protocol::handler_response_serial();
    let root = TempDir::new().unwrap();
    let mut client = client(&root, "normal");
    client.consecutive_failures = client.max_consecutive_failures;
    client.degraded_since = Some(Instant::now() - Duration::from_secs(1));
    client.status = crate::cbm::config::CbmStatus::Degraded("circuit open".into());
    let config = crate::cbm::CbmConfig {
        enabled: false,
        cache_db_path: Some(root.path().join("bridge.db").to_string_lossy().into()),
        ..Default::default()
    };
    let mut bridge = crate::cbm::GraphBridge::try_create(&config, root.path());
    bridge.indexing_state.lock().unwrap().insert(
        bridge.project.clone().unwrap(),
        crate::cbm::bridge::IndexingState::Complete,
    );
    *bridge.client.lock().unwrap() = Some(client);
    bridge.update_status();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    *state.graph_bridge_lock() = Some(bridge);
    crate::mcp::tools::setup_handler_registry_for_tests();
    crate::protocol::captured_responses().clear();
    fixture::with_options(&log(&root), "normal", || {
        crate::mcp::tools::dispatch_tools_call(
            &json!(904),
            "graph_query",
            &json!({"arguments":{"query":"MATCH (n) RETURN n"}}),
            &state,
        )
    });
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(
        response.get("error").is_none(),
        "MCP guard blocked half-open recovery: {response}"
    );
    assert_eq!(launches(&root), 2);
    assert_eq!(tool_count(&root, "query_graph"), 1);
}

#[path = "frontend_recovery_audit.rs"]
mod audit;
