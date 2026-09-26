//! Visible-content metadata across delta, apply, restore, and replay.
//!
//! Delta payloads remain code-side auxiliary fields. These tests describe the
//! human/model-visible text only; they never make the LLM a delta consumer.

use crate::mcp::tool_handlers::core::ContentKind;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn config(root: &tempfile::TempDir, database: &str) -> crate::config::CleanCtxConfig {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root.path().join(database).to_string_lossy().into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    config
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn workspace(root: &tempfile::TempDir) -> String {
    root.path().to_string_lossy().into_owned()
}

fn edit_source(right_value: u8) -> String {
    format!(
        "class Left {{ run() {{ return 1; }} }}\n\
         class Right {{ run() {{ return {right_value}; }} }}\n{}",
        "// economics-padding-0123456789abcdef\n".repeat(1_000)
    )
}

fn assert_meta_contract(response: &Value, kind: ContentKind, byte_exact: Value) {
    assert_eq!(response["result"]["_meta"]["content_kind"], kind.as_str());
    assert_eq!(response["result"]["_meta"]["byte_exact"], byte_exact);
}

#[test]
fn dedicated_delta_labels_only_its_visible_acknowledgement_as_summary() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("dedicated.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, edit_source(2)).expect("baseline source");
    let state = crate::mcp::McpState::new(config(&root, "dedicated.db"));
    let args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": workspace(&root),
            "fidelity": "edit"
        })
    };

    let baseline = dispatch(&state, 1, "delta_code_context", args());
    assert!(baseline.get("error").is_none(), "{baseline}");
    std::fs::write(&path, edit_source(3)).expect("target source");
    state.invalidate_source_cache(&file);
    let delta = dispatch(&state, 2, "delta_code_context", args());
    assert!(delta["result"]["delta"].is_object(), "{delta}");
    assert_eq!(
        delta["result"]["content_kind"],
        ContentKind::DeltaSummary.as_str()
    );
    assert_eq!(delta["result"]["byte_exact"], json!([]));
    let visible = delta["result"]["content"][0]["text"]
        .as_str()
        .expect("visible summary");
    assert!(visible.starts_with("Δ delta for"), "{visible}");
    assert!(
        !visible.contains("\"ops\""),
        "delta payload leaked: {visible}"
    );
}

#[test]
fn delta_baseline_and_cache_report_their_visible_edit_bodies() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("cached-edit.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, edit_source(2)).expect("source");
    let state = crate::mcp::McpState::new(config(&root, "cached-edit.db"));
    let args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": workspace(&root),
            "fidelity": "edit"
        })
    };

    for (id, cached) in [(5, false), (6, true)] {
        let response = dispatch(&state, id, "delta_code_context", args());
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(
            response["result"]["cached"].as_bool().unwrap_or(false),
            cached
        );
        assert_eq!(
            response["result"]["content_kind"],
            ContentKind::SkeletonWithVerbatimBodies.as_str()
        );
        assert_eq!(response["result"]["byte_exact"], json!(["method_bodies"]));
    }
}

#[test]
fn edit_metadata_survives_explicit_delta_apply_restart_restore_and_replay() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("focused.ts");
    let file = path.to_string_lossy().into_owned();
    let workspace_root = workspace(&root);
    std::fs::write(&path, edit_source(2)).expect("baseline source");
    let config = config(&root, "focused.db");
    let state = crate::mcp::McpState::new(config.clone());
    let delta_args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": workspace_root.clone(),
            "fidelity": "edit"
        })
    };

    let baseline = dispatch(&state, 10, "delta_code_context", delta_args());
    assert!(baseline.get("error").is_none(), "{baseline}");
    assert_eq!(
        baseline["result"]["content_kind"],
        ContentKind::SkeletonWithVerbatimBodies.as_str()
    );
    assert_eq!(baseline["result"]["byte_exact"], json!(["method_bodies"]));

    std::fs::write(&path, edit_source(3)).expect("target source");
    state.invalidate_source_cache(&file);
    let generated = dispatch(&state, 11, "delta_code_context", delta_args());
    assert_eq!(generated["result"]["strategy"], "delta");
    assert!(generated["result"]["delta"].is_object());
    assert_eq!(
        generated["result"]["content_kind"],
        ContentKind::DeltaSummary.as_str()
    );
    assert_eq!(generated["result"]["byte_exact"], json!([]));

    let applied = dispatch(
        &state,
        12,
        "apply_delta",
        json!({
            "delta": generated["result"]["delta"].clone(),
            "currentVersion": generated["result"]["from_version"].clone()
        }),
    );
    assert!(applied.get("error").is_none(), "{applied}");
    assert_meta_contract(
        &applied,
        ContentKind::SkeletonWithFocusedVerbatimBodies,
        json!(["focused_method_bodies"]),
    );
    drop(state);

    let restarted = crate::mcp::McpState::new(config);
    for (id, tool) in [(13, "restore_context"), (14, "replay_history")] {
        let response = dispatch(&restarted, id, tool, json!({ "filePath": file.clone() }));
        assert!(response.get("error").is_none(), "{tool}: {response}");
        assert_meta_contract(
            &response,
            ContentKind::SkeletonWithVerbatimBodies,
            json!(["method_bodies"]),
        );
        let visible = response["result"]["content"][0]["text"]
            .as_str()
            .expect("visible Edit presentation");
        assert!(visible.contains("return 3"), "{tool}: {visible}");
        assert!(!visible.contains("return 1"), "{tool}: {visible}");
    }
}

#[test]
fn regenerated_edit_contract_distinguishes_all_bodies_from_no_bodies() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let full_path = root.path().join("full.ts");
    let empty_path = root.path().join("empty.ts");
    let full = full_path.to_string_lossy().into_owned();
    let empty = empty_path.to_string_lossy().into_owned();
    std::fs::write(&full_path, edit_source(2)).expect("full source");
    std::fs::write(&empty_path, edit_source(2)).expect("empty-focus source");
    let config = config(&root, "coverage.db");
    let state = crate::mcp::McpState::new(config.clone());

    let full_response = dispatch(
        &state,
        20,
        "provide_code_context",
        json!({
            "filePath": full.clone(),
            "workspaceRoot": workspace(&root),
            "fidelity": "edit"
        }),
    );
    assert!(full_response.get("error").is_none(), "{full_response}");
    let empty_response = dispatch(
        &state,
        21,
        "provide_code_context",
        json!({
            "filePath": empty.clone(),
            "workspaceRoot": workspace(&root),
            "fidelity": "edit",
            "focusMethods": []
        }),
    );
    assert!(empty_response.get("error").is_none(), "{empty_response}");
    drop(state);

    let restarted = crate::mcp::McpState::new(config);
    for (id, file, kind, exact) in [
        (
            22,
            &full,
            ContentKind::SkeletonWithVerbatimBodies,
            json!(["method_bodies"]),
        ),
        (23, &empty, ContentKind::Skeleton, json!([])),
    ] {
        let restored = dispatch(
            &restarted,
            id,
            "restore_context",
            json!({ "filePath": file }),
        );
        assert!(restored.get("error").is_none(), "{restored}");
        assert_meta_contract(&restored, kind, exact);
    }
}

#[test]
fn tiny_follow_up_provider_keeps_complete_raw_content() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("tiny.ts");
    let file = path.to_string_lossy().into_owned();
    let state = crate::mcp::McpState::new(config(&root, "tiny.db"));
    let baseline_args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": workspace(&root),
            "fidelity": "high"
        })
    };
    let follow_up_args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": workspace(&root)
        })
    };
    std::fs::write(&path, "class Tiny { one() {} }\n").expect("baseline source");
    let baseline = dispatch(&state, 30, "provide_code_context", baseline_args());
    assert!(baseline.get("error").is_none(), "{baseline}");

    let target = "class Tiny { two() {} }\n";
    std::fs::write(&path, target).expect("target source");
    state.invalidate_source_cache(&file);
    let follow_up = dispatch(&state, 31, "provide_code_context", follow_up_args());
    assert_eq!(follow_up["result"]["_meta"]["strategy"], "full");
    assert!(follow_up["result"]["_meta"].get("delta").is_none());
    assert_meta_contract(&follow_up, ContentKind::RawPassthrough, json!(["document"]));
    assert_eq!(follow_up["result"]["content"][0]["text"], target);
}
