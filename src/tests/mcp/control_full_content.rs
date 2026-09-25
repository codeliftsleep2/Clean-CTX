#![cfg(feature = "typescript")]

use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::tool_handlers::core::ContentKind;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

/// The model-visible presentation text (SCHEMA v5) a response puts in front of
/// the LLM. The CONTROL-FULL codec is code-side only (`result.ir`); these tests
/// assert the presentation, never the codec.
fn model_text(response: &Value) -> String {
    response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible text")
        .to_string()
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
    let text = model_text(&response);
    // Local structural facts survive in the model-visible presentation.
    assert!(
        text.starts_with("// SCHEMA v5"),
        "presentation, not codec: {text}"
    );
    assert!(
        text.contains("Consumer"),
        "typed owner in presentation: {text}"
    );
    assert!(
        text.contains("find"),
        "method identity in presentation: {text}"
    );
    // Semantic facts are auxiliary: they ride the workspace index, not content.
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
    let text = model_text(&response);
    // Both owners render; only the focused method's body is byte-exact.
    assert!(text.contains("Left"), "left owner must render: {text}");
    assert!(text.contains("Right"), "right owner must render: {text}");
    assert!(
        !text.contains("return 1"),
        "unfocused body must be omitted: {text}"
    );
    assert!(
        text.contains("return 2"),
        "focused body must be verbatim: {text}"
    );
    // The contract self-reports the focused-body category.
    assert_eq!(
        response["result"]["_meta"]["content_kind"],
        ContentKind::SkeletonWithFocusedVerbatimBodies.as_str()
    );
    assert_eq!(
        response["result"]["_meta"]["byte_exact"],
        json!(["focused_method_bodies"])
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
            .is_some_and(|text| text.starts_with("// SCHEMA v5"))
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
    assert!(
        text.starts_with("Δ delta for"),
        "delta content is the minimal summary, not a codec envelope: {text}"
    );
    assert!(
        delta["result"]["delta"].is_object(),
        "structured op list rides code-side in result.delta"
    );

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
    let applied_text = model_text(&applied);
    assert!(
        applied_text.starts_with("// SCHEMA v5"),
        "applied delta content is the presentation, not the codec: {applied_text}"
    );
}

#[test]
fn registered_restore_and_replay_regenerate_presentation_from_durable_facts() {
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
        let text = model_text(&response);
        assert!(
            text.contains("Durable"),
            "{tool} typed owner survives regeneration: {text}"
        );
        assert!(
            text.contains("run"),
            "{tool} method identity survives regeneration: {text}"
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
    assert_eq!(
        compressed["result"]["content_kind"],
        ContentKind::RawPassthrough.as_str()
    );

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
            ContentKind::RawPassthrough.as_str()
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
    if response["result"]["_meta"]["content_kind"] == ContentKind::RawPassthrough.as_str() {
        assert_eq!(visible.as_bytes(), source.as_bytes());
        assert_eq!(response["result"]["_meta"]["template_compressed"], false);
    } else {
        assert_eq!(response["result"]["_meta"]["template_compressed"], true);
    }
}
