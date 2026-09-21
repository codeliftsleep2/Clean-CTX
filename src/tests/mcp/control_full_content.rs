#![cfg(feature = "typescript")]

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn control_full_json(response: &Value) -> Value {
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible text");
    let payload = text
        .split("\n§PATHMAP")
        .next()
        .expect("CONTROL-FULL payload");
    let (_, json_text) = payload.split_once('\n').expect("versioned header");
    serde_json::from_str(json_text).expect("named CONTROL-FULL JSON")
}

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn persistent_state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = root
        .path()
        .join("control-full.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

#[test]
fn registered_provide_exposes_calls_injections_edges_and_provenance_in_content() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("consumer.service.ts");
    std::fs::write(
        &path,
        r#"import { Injectable } from '@angular/core';
@Injectable()
export class Consumer {
  constructor(private repo: Repo, private again: Repo) {}
  find(key: string): void { lookup(key); lookup(key); external(...items); }
  find(key: number): void { lookup(key); }
}
"#,
    )
    .unwrap();
    let state = state(&root);
    let response = dispatch(
        &state,
        1,
        "provide_code_context",
        json!({
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "high"
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
    let payload = control_full_json(&response);

    assert_eq!(payload["schema"], "clean-ctx/control-full");
    assert!(payload["classes"][0]["id"].as_str().is_some());
    let methods = payload["classes"][0]["methods"].as_array().unwrap();
    let overloads = methods
        .iter()
        .filter(|method| method["name"] == "find")
        .collect::<Vec<_>>();
    assert_eq!(overloads.len(), 2, "same-owner overload occurrences");
    assert_ne!(overloads[0]["id"], overloads[1]["id"]);

    let calls = payload["calls"].as_array().unwrap();
    assert_eq!(
        calls
            .iter()
            .filter(|call| call["callee_written_name"] == "lookup")
            .count(),
        3,
        "duplicate calls remain distinct and ordered"
    );
    assert!(calls.iter().any(|call| {
        call["callee_written_name"] == "external"
            && call["has_spread"] == true
            && call["callee_resolution"] == "unresolved"
    }));

    let edges = payload["semantic_edges"].as_array().unwrap();
    let injection = edges
        .iter()
        .find(|edge| edge["relation"] == "Injects")
        .expect("framework injection edge must be model-visible");
    assert_eq!(injection["layer"], "angular");
    assert!(
        injection["subject"]["file"]
            .as_str()
            .is_some_and(|file| file.ends_with("consumer.service.ts"))
    );
}

#[test]
fn registered_focus_resolves_owner_then_filters_by_canonical_method_id() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("owners.ts");
    std::fs::write(
        &path,
        "class Left { run() { return 1; } }\nclass Right { run() { return 2; } }\n",
    )
    .unwrap();
    let state = state(&root);
    let args = json!({
        "filePath": path.to_string_lossy(),
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": "edit",
        "focusMethods": ["Right.run"]
    });
    let response = dispatch(&state, 1, "provide_code_context", args);
    assert!(response.get("error").is_none(), "{response}");
    let payload = control_full_json(&response);
    let classes = payload["classes"].as_array().unwrap();
    let left = classes
        .iter()
        .find(|owner| owner["name"] == "Left")
        .unwrap();
    let right = classes
        .iter()
        .find(|owner| owner["name"] == "Right")
        .unwrap();
    assert!(left["methods"][0]["body"].is_null());
    assert_eq!(right["methods"][0]["body"], "{ return 2; }");
    assert_eq!(
        payload["mode"]["exact_body_method_ids"][0],
        right["methods"][0]["id"]
    );
}

#[test]
fn registered_focus_rejects_ambiguous_bare_method() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("owners.ts");
    std::fs::write(
        &path,
        "class Left { run() { return 1; } }\nclass Right { run() { return 2; } }\n",
    )
    .unwrap();
    let state = state(&root);
    let response = dispatch(
        &state,
        1,
        "provide_code_context",
        json!({
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "edit",
            "focusMethods": ["run"]
        }),
    );
    assert_eq!(response["error"]["code"], -32602);
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("owned by multiple types"))
    );
}

#[test]
fn registered_delta_exposes_full_baseline_then_exact_delta_in_content() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("delta.ts");
    std::fs::write(&path, "class Delta { run() { first(); } }\n").unwrap();
    let state = state(&root);
    let args = || {
        json!({
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "high"
        })
    };

    let baseline = dispatch(&state, 1, "delta_code_context", args());
    assert!(
        baseline["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|text| text.starts_with("// CONTROL-FULL v1"))
    );

    std::fs::write(&path, "class Delta { run() { second(); } }\n").unwrap();
    state.invalidate_source_cache(path.to_string_lossy().as_ref());
    let delta = dispatch(&state, 2, "delta_code_context", args());
    let text = delta["result"]["content"][0]["text"]
        .as_str()
        .expect("delta content");
    assert!(text.starts_with("// CONTROL-FULL-DELTA v1"));
    assert!(text.contains("\"delta\""));
    assert!(text.contains("\"semantic_edges_after_apply\""));

    let applied = dispatch(
        &state,
        3,
        "apply_delta",
        json!({
            "delta": delta["result"]["delta"].clone(),
            "currentVersion": delta["result"]["from_version"].clone()
        }),
    );
    assert!(applied.get("error").is_none(), "{applied}");
    let applied_payload = control_full_json(&applied);
    assert!(applied_payload["calls"].as_array().is_some_and(|calls| {
        calls
            .iter()
            .any(|call| call["callee_written_name"] == "second")
    }));
}

#[test]
fn registered_restore_and_replay_regenerate_control_full_from_durable_facts() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("durable.ts");
    std::fs::write(&path, "class Durable { run() { external(); } }\n").unwrap();
    let state = persistent_state(&root);
    let file = path.to_string_lossy().into_owned();
    let produced = dispatch(
        &state,
        1,
        "compress_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "high"
        }),
    );
    assert!(produced.get("error").is_none(), "{produced}");

    for (id, tool) in [(2, "restore_context"), (3, "replay_history")] {
        let response = dispatch(
            &state,
            id,
            tool,
            json!({ "filePath": path.to_string_lossy() }),
        );
        assert!(response.get("error").is_none(), "{response}");
        let payload = control_full_json(&response);
        assert_eq!(payload["schema"], "clean-ctx/control-full");
        assert!(
            payload["calls"]
                .as_array()
                .is_some_and(|calls| !calls.is_empty())
        );
    }
}
