#![cfg(feature = "typescript")]

use crate::layers::meta::semantic::SemanticRelation;
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
    if text.starts_with("// COMPACT-A A1") || text.starts_with("// COMPACT-A A2") {
        let payload = text.split("\n§PATHMAP").next().expect("COMPACT-A payload");
        let (_, after_header) = payload.split_once('\n').expect("A1 header");
        let (_, encoded_and_bodies) = after_header.split_once('\n').expect("A1 legend");
        let (encoded, body_wire) = encoded_and_bodies
            .split_once("\n§BODIES\n")
            .expect("A1 body boundary");
        let mut envelope: Value = serde_json::from_str(encoded).expect("compact envelope JSON");
        let is_a2 = envelope["A"] == 2;
        if is_a2 {
            envelope["g"]["E"] = json!([]);
            envelope["n"]["E"] = json!([]);
        }
        let mut decoded = crate::ir::control_full::compact_a_envelope_tests::decode(
            &envelope,
            body_wire.as_bytes(),
        )
        .expect("decoded normalized CONTROL-FULL");
        if is_a2 {
            decoded.as_object_mut().unwrap().remove("navigation");
        }
        return decoded;
    }
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
fn registered_provide_keeps_local_facts_in_content_and_workspace_edges_auxiliary() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("consumer.service.ts");
    let mut source = r#"import { Injectable } from '@angular/core';
@Injectable()
export class Consumer {
  constructor(private repo: Repo, private again: Repo) {}
  find(key: string): void { lookup(key); lookup(key); external(...items); }
  find(key: number): void { lookup(key); }
}
"#
    .to_string();
    source.push_str(&"// economics-padding-0123456789abcdef\n".repeat(1_000));
    std::fs::write(&path, source).unwrap();
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

    assert_eq!(payload["schema"], "clean-ctx/file-context");
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

    assert_eq!(payload["semantic_edges"], json!([]));
    assert!(
        response["result"]["_meta"]["semantic_edges"].is_null(),
        "semantic edges no longer ride on the content response"
    );
    let index = state.workspace_index_lock();
    let edges = index.forward_edges_by_identity("angular", "Service", "Consumer");
    let injection = edges
        .iter()
        .find(|edge| edge.relation == SemanticRelation::Injects)
        .expect("framework injection edge must remain available to the workspace index");
    assert_eq!(injection.layer, "angular");
    assert!(
        injection
            .subject
            .file
            .as_deref()
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
        format!(
            "class Left {{ run() {{ return 1; }} }}\nclass Right {{ run() {{ return 2; }} }}\n{}",
            "// economics-padding-0123456789abcdef\n".repeat(1_000)
        ),
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
    std::fs::write(
        &path,
        format!(
            "class Delta {{ run() {{ first(); }} }}\n{}",
            "// economics-padding-0123456789abcdef\n".repeat(1_000)
        ),
    )
    .unwrap();
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
        baseline["result"]["ir"].is_object(),
        "delta baseline must expose its structured lifecycle snapshot"
    );
    assert!(
        baseline["result"]["content"][0]["text"]
            .as_str()
            .is_some_and(|text| text.starts_with("// COMPACT-A A2"))
    );

    let cached = dispatch(&state, 2, "delta_code_context", args());
    assert_eq!(cached["result"]["cached"], true);
    assert!(cached["result"]["ir"].is_object());

    std::fs::write(
        &path,
        format!(
            "class Delta {{ run() {{ second(); }} }}\n{}",
            "// economics-padding-0123456789abcdef\n".repeat(1_000)
        ),
    )
    .unwrap();
    state.invalidate_source_cache(path.to_string_lossy().as_ref());
    let delta = dispatch(&state, 3, "delta_code_context", args());
    let text = delta["result"]["content"][0]["text"]
        .as_str()
        .expect("delta content");
    assert!(text.starts_with("// FILE-CONTEXT-DELTA v1"));
    assert!(text.contains("\"delta\""));
    assert!(!text.contains("semantic_edges_after_apply"));
    assert!(text.contains("workspace_query"));
    let (_, delta_json) = text.split_once('\n').expect("delta header");
    let delta_payload: serde_json::Value =
        serde_json::from_str(delta_json).expect("CONTROL-FULL delta JSON");
    assert_eq!(delta_payload["schema_version"], 1);
    assert_eq!(delta_payload["schema"], "clean-ctx/file-context-delta");

    let applied = dispatch(
        &state,
        4,
        "apply_delta",
        json!({
            "delta": delta["result"]["delta"].clone(),
            "currentVersion": delta["result"]["from_version"].clone()
        }),
    );
    assert!(applied.get("error").is_none(), "{applied}");
    assert!(
        applied["result"]["structuredContent"]["ir"].is_object(),
        "applied delta must expose its structured lifecycle snapshot"
    );
    let applied_payload = control_full_json(&applied);
    assert_eq!(applied_payload["schema_version"], 1);
    assert_eq!(applied_payload["semantic_edges"], json!([]));
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
    std::fs::write(
        &path,
        format!(
            "class Durable {{ run() {{ external(); }} }}\n{}",
            "// economics-padding-0123456789abcdef\n".repeat(1_000)
        ),
    )
    .unwrap();
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
        assert!(response["result"]["ir"].is_object(), "{tool} hierarchy");
        let payload = control_full_json(&response);
        assert_eq!(payload["schema"], "clean-ctx/file-context");
        assert!(payload.get("navigation").is_none());
        assert_eq!(payload["semantic_edges"], json!([]));
        assert!(
            payload["calls"]
                .as_array()
                .is_some_and(|calls| !calls.is_empty())
        );
    }
}

#[test]
fn small_file_lifecycle_uses_byte_exact_raw_without_a_wrapper() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("tiny.ts");
    let source = "class Tiny { run() { return 1; } }\r\n";
    std::fs::write(&path, source.as_bytes()).unwrap();
    let state = persistent_state(&root);
    let arguments = json!({
        "filePath": path.to_string_lossy(),
        "workspaceRoot": root.path().to_string_lossy(),
        "fidelity": "high"
    });

    let compressed = dispatch(&state, 1, "compress_code_context", arguments);
    assert_eq!(compressed["result"]["content"][0]["text"], source);
    assert_eq!(compressed["result"]["content_kind"], "raw_passthrough");

    for (id, tool) in [(2, "restore_context"), (3, "replay_history")] {
        let response = dispatch(
            &state,
            id,
            tool,
            json!({ "filePath": path.to_string_lossy() }),
        );
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(response["result"]["content"][0]["text"], source);
        assert_eq!(
            response["result"]["_meta"]["content_kind"],
            "raw_passthrough"
        );
    }
}

#[test]
fn small_angular_template_also_obeys_the_raw_economics_ceiling() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("tiny.component.html");
    let source = "<button (click)=\"save()\">Save</button>\r\n";
    std::fs::write(&path, source.as_bytes()).unwrap();
    let state = state(&root);
    let response = dispatch(
        &state,
        1,
        "provide_code_context",
        json!({
            "filePath": path.to_string_lossy(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "medium"
        }),
    );

    let visible = response["result"]["content"][0]["text"]
        .as_str()
        .expect("template content");
    let tokenizer = crate::tokenizer::create_tokenizer(Default::default()).unwrap();
    assert!(tokenizer.count_tokens(visible) <= tokenizer.count_tokens(source));
    if response["result"]["_meta"]["content_kind"] == "raw_passthrough" {
        assert_eq!(visible.as_bytes(), source.as_bytes());
        assert_eq!(response["result"]["_meta"]["template_compressed"], false);
    } else {
        assert_eq!(response["result"]["_meta"]["template_compressed"], true);
    }
}
