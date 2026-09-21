//! Registered P9-22 observational context-stats contract.

use crate::mcp::context_store::ContextStore;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("context-stats.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn dispatch(state: &crate::mcp::McpState, id: i64, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(id),
        "context_stats",
        &json!({ "arguments": args }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered response")
}

fn compile(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &str) {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "compress_code_context",
        &json!({ "arguments": {
            "filePath": file,
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "low"
        }}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("compression response");
    assert!(response.get("error").is_none(), "{response}");
}

fn domain_stats(state: &crate::mcp::McpState) -> Value {
    serde_json::to_value(state.session_stats_lock().domain_breakdown()).unwrap()
}

#[test]
fn registered_description_declares_observational_behavior() {
    let tool = crate::mcp::tools::tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "context_stats")
        .expect("registered context_stats tool");
    let description = tool["description"].as_str().unwrap();
    assert!(description.contains("without flushing persistence"));
    assert!(description.contains("mutating lifecycle state"));
}

#[test]
fn stats_does_not_flush_queued_save_or_clear_work() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let committed = root
        .path()
        .join("committed.ts")
        .to_string_lossy()
        .into_owned();
    let queued = root.path().join("queued.ts").to_string_lossy().into_owned();
    std::fs::write(&committed, "export class Committed {}\n").unwrap();
    let state = state(&root);
    compile(&state, &root, &committed);

    {
        let guard = state.persistence_store_lock();
        let store = guard.as_ref().unwrap();
        store.queue_save_context(
            &queued,
            crate::compression::Fidelity::Low,
            "queued",
            b"",
            "queued-hash",
            10,
            5,
        );
        store.queue_clear_file(&committed);
        assert_eq!(store.pending_count(), 2);
    }

    let first = dispatch(&state, 2, json!({ "filePath": &committed }));
    assert!(first.get("error").is_none(), "{first}");
    assert!(
        first["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|text| text.contains("Persistence: enabled")),
        "{first}"
    );
    let second = dispatch(&state, 3, json!({ "filePath": &committed }));
    assert_eq!(first["result"]["content"], second["result"]["content"]);
    let guard = state.persistence_store_lock();
    let store = guard.as_ref().unwrap();
    assert_eq!(store.pending_count(), 2);
    let sqlite = store.sqlite().unwrap();
    assert!(sqlite.has_context(&committed));
    assert!(!sqlite.has_context(&queued));
}

#[test]
fn proxy_observation_changes_only_the_response_snapshot() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let state = state(&root);
    let before = domain_stats(&state);
    let mut programs = std::collections::HashMap::new();
    programs.insert(
        "cargo".to_string(),
        crate::mcp::proxy_stats::ProxyProgramFilterStats {
            program: "cargo".to_string(),
            applications: 1,
            original_tokens: 100,
            filtered_tokens: 20,
            tokens_saved: 80,
            original_lines: 10,
            filtered_lines: 2,
            lines_removed: 8,
            reduction_pct: 80.0,
        },
    );
    crate::mcp::proxy_stats::inject_test_proxy_stats(crate::mcp::proxy_stats::ProxyStatsResponse {
        filter_stats: crate::mcp::proxy_stats::ProxyFilterStats {
            total_tokens_saved: 80,
            total_lines_filtered: 8,
            total_applications: 1,
            programs,
        },
        cache_stats: crate::mcp::proxy_stats::ProxyCacheStats::default(),
    });

    let response = dispatch(&state, 10, json!({ "format": "json" }));
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("tool_filter"), "{response}");
    assert_eq!(domain_stats(&state), before);
}

#[test]
fn stats_preserves_semantic_and_pending_edit_ownership() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("owned.ts").to_string_lossy().into_owned();
    std::fs::write(&file, "export class Owned {}\n").unwrap();
    let state = state(&root);
    compile(&state, &root, &file);
    let alias = state.alias_for_path(&file).unwrap();
    let prior_ir = state.ir_context_read().get_ir(&alias).cloned();
    let prior_hash = state.ir_context_read().get_source_hash(&alias).cloned();
    let prior_edges = serde_json::to_value(state.semantic_edges(&alias)).unwrap();
    let prior_index = state.workspace_index_read().edge_count();
    let prior_fidelity = state.context_fidelity(&alias);

    let source = std::fs::read(&file).unwrap();
    let (mut target_ir, target_edges, target_hash) =
        crate::mcp::tool_helpers::compile_source_ir_candidate(
            &file,
            std::str::from_utf8(&source).unwrap(),
            crate::compression::Fidelity::Edit,
            &state,
        )
        .unwrap();
    target_ir.file_id.clone_from(&file);
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: "stats-pending".to_string(),
        file_path: file.clone(),
        prior_hash: prior_hash.clone().unwrap(),
        target_hash,
        prior_version: state.file_version(&alias).unwrap(),
        target_version: target_ir.version,
        prior_source: source.clone(),
        target_source: source,
        target_ir: crate::ir::binary_wire::encode(&target_ir),
        target_edges,
        fidelity: crate::compression::Fidelity::Edit,
        stage_path: String::new(),
    };
    state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .establish_edit_intent(&intent)
        .unwrap();

    let response = dispatch(&state, 20, json!({}));
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(state.ir_context_read().get_ir(&alias).cloned(), prior_ir);
    assert_eq!(
        state.ir_context_read().get_source_hash(&alias).cloned(),
        prior_hash
    );
    assert_eq!(
        serde_json::to_value(state.semantic_edges(&alias)).unwrap(),
        prior_edges
    );
    assert_eq!(state.workspace_index_read().edge_count(), prior_index);
    assert_eq!(state.context_fidelity(&alias), prior_fidelity);
    assert!(
        state
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap()
    );
}
