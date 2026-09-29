//! Registered-path contracts for owner-aware file-local call inspection.

use crate::mcp::tools::dispatch_tools_call;
use crate::tests::assert_valid_mcp_envelope;
use serde_json::{Value, json};

fn dispatch(state: &crate::mcp::McpState, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered workspace_query response")
}

fn query(
    state: &crate::mcp::McpState,
    root: &tempfile::TempDir,
    file: &str,
    owner: &str,
    method: Value,
) -> Value {
    dispatch(
        state,
        json!({
            "type": "calls_in_file",
            "filePath": file,
            "workspaceRoot": root.path(),
            "owner": { "kind": "class", "name": owner },
            "method": method,
        }),
    )
}

fn structured(response: &Value) -> &Value {
    let result = response["result"].as_object().expect("result object");
    assert_valid_mcp_envelope(result);
    &response["result"]["structuredContent"]
}

#[test]
fn calls_in_file_separates_same_named_callers_and_preserves_overloads() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("Owners.cs");
    std::fs::write(
        &path,
        concat!(
            "class Alpha {\n",
            "  void Run(string value) { Lookup(value); Lookup(value); }\n",
            "  void Run(int value, int count) { Numeric(value, count); }\n",
            "}\n",
            "class Beta {\n",
            "  void Run(string value) { Hidden(value); }\n",
            "}\n",
        ),
    )
    .expect("fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let file = path.to_string_lossy();

    let alpha = query(&state, &root, &file, "Alpha", json!({ "name": "Run" }));
    assert!(alpha.get("error").is_none(), "{alpha}");
    let answer = structured(&alpha);
    assert_eq!(answer["owner"], json!({ "kind": "class", "name": "Alpha" }));
    assert_eq!(answer["overload_count"], 2);
    assert_eq!(answer["count"], 3);
    assert_eq!(answer["overloads"][0]["overload_occurrence"], 0);
    assert_eq!(
        answer["overloads"][0]["parameters"],
        json!(["string value"])
    );
    assert_eq!(
        answer["overloads"][0]["calls"],
        json!([
            { "occurrence": 0, "callee_written": "Lookup", "explicit_argument_count": 1, "has_spread": false },
            { "occurrence": 1, "callee_written": "Lookup", "explicit_argument_count": 1, "has_spread": false }
        ])
    );
    assert_eq!(
        answer["overloads"][1]["parameters"],
        json!(["int value", "int count"])
    );
    assert_eq!(
        answer["overloads"][1]["calls"][0]["callee_written"],
        "Numeric"
    );
    assert!(!answer.to_string().contains("Hidden"));
    assert!(!answer.to_string().contains("\"M1\""));
    let content = alpha["result"]["content"][0]["text"]
        .as_str()
        .expect("model-facing content");
    assert!(content.contains("fresh_canonical_file_ir"), "{content}");

    let beta = query(&state, &root, &file, "Beta", json!({ "name": "Run" }));
    assert_eq!(
        structured(&beta)["overloads"][0]["calls"][0]["callee_written"],
        "Hidden"
    );

    let missing = query(&state, &root, &file, "Gamma", json!({ "name": "Run" }));
    assert_eq!(structured(&missing)["overload_count"], 0);
    assert_eq!(structured(&missing)["count"], 0);
}

#[test]
fn calls_in_file_visible_signature_selector_never_guesses_an_overload() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("Overloads.cs");
    std::fs::write(
        &path,
        concat!(
            "class Alpha {\n",
            "  void Run(string value) { Text(value); }\n",
            "  void Run(int value) { Number(value); }\n",
            "}\n",
        ),
    )
    .expect("fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let file = path.to_string_lossy();

    let selected = query(
        &state,
        &root,
        &file,
        "Alpha",
        json!({ "name": "Run", "parameters": ["int value"], "return_type": "void" }),
    );
    let answer = structured(&selected);
    assert_eq!(answer["overload_count"], 1);
    assert_eq!(answer["overloads"][0]["overload_occurrence"], 1);
    assert_eq!(
        answer["overloads"][0]["calls"][0]["callee_written"],
        "Number"
    );

    let incomplete = query(&state, &root, &file, "Alpha", json!({ "name": "Run" }));
    assert_eq!(structured(&incomplete)["overload_count"], 2);

    let absent = query(
        &state,
        &root,
        &file,
        "Alpha",
        json!({ "name": "Run", "parameters": ["bool value"] }),
    );
    assert_eq!(structured(&absent)["overload_count"], 0);
    assert_eq!(structured(&absent)["count"], 0);
}

#[test]
fn calls_in_file_preserves_typescript_spread_and_duplicate_occurrences() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("spread.ts");
    std::fs::write(
        &path,
        "class Alpha { run(items: string[]): void { save(...items); save(...items); } }\n",
    )
    .expect("fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let file = path.to_string_lossy();

    let response = query(&state, &root, &file, "Alpha", json!({ "name": "run" }));
    let answer = structured(&response);
    assert_eq!(answer["overloads"][0]["parameters"], json!(["string[]"]));
    assert_eq!(answer["count"], 2);
    for (occurrence, call) in answer["overloads"][0]["calls"]
        .as_array()
        .expect("calls")
        .iter()
        .enumerate()
    {
        assert_eq!(call["occurrence"], occurrence);
        assert_eq!(call["callee_written"], "save");
        assert_eq!(call["explicit_argument_count"], 1);
        assert_eq!(call["has_spread"], true);
    }
}

#[test]
fn calls_in_file_reads_fresh_source_without_publishing_session_state() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("fresh.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(&path, "class Alpha { run(): void { before(); } }\n").expect("fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let before = query(&state, &root, &file, "Alpha", json!({ "name": "run" }));
    assert_eq!(
        structured(&before)["overloads"][0]["calls"][0]["callee_written"],
        "before"
    );
    assert!(state.alias_for_path(&file).is_none());

    std::fs::write(&path, "class Alpha { run(): void { after(); } }\n").expect("updated fixture");
    state.invalidate_source_cache(&file);
    let after = query(&state, &root, &file, "Alpha", json!({ "name": "run" }));
    assert_eq!(
        structured(&after)["overloads"][0]["calls"][0]["callee_written"],
        "after"
    );
    assert!(state.alias_for_path(&file).is_none());
    assert_eq!(state.workspace_index_read().edge_count(), 0);

    let excluded = root.path().join("elsewhere");
    std::fs::create_dir(&excluded).expect("narrowing directory");
    let narrowed = dispatch(
        &state,
        json!({
            "type": "calls_in_file",
            "filePath": file,
            "workspaceRoot": root.path(),
            "withinPath": excluded,
            "owner": { "kind": "class", "name": "Alpha" },
            "method": { "name": "run" }
        }),
    );
    assert_eq!(structured(&narrowed)["overload_count"], 0);
    assert_eq!(structured(&narrowed)["count"], 0);
}

#[test]
fn calls_in_file_rejects_ambiguous_typed_owner_and_invalid_shape() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("ambiguous.ts");
    let file = path.to_string_lossy().into_owned();
    std::fs::write(
        &path,
        "class Alpha { run(): void { one(); } } class Alpha { run(): void { two(); } }\n",
    )
    .expect("fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let ambiguous = query(&state, &root, &file, "Alpha", json!({ "name": "run" }));
    assert_eq!(ambiguous["error"]["code"], -32602);
    assert!(
        ambiguous["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("owner is ambiguous")),
        "{ambiguous}"
    );

    let invalid = dispatch(
        &state,
        json!({
            "type": "calls_in_file",
            "filePath": file,
            "workspaceRoot": root.path(),
            "owner": { "kind": "namespace", "name": "Alpha" },
            "method": { "name": "run" }
        }),
    );
    assert_eq!(invalid["error"]["code"], -32602);

    let missing_root = dispatch(
        &state,
        json!({
            "type": "calls_in_file",
            "filePath": file,
            "owner": { "kind": "class", "name": "Alpha" },
            "method": { "name": "run" }
        }),
    );
    assert_eq!(missing_root["error"]["code"], -32602);
}

#[test]
fn workspace_query_schema_declares_calls_in_file_contract() {
    let tools = crate::mcp::tools::tool_list();
    let query = tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .expect("workspace_query tool");
    let variants = query["inputSchema"]["properties"]["type"]["enum"]
        .as_array()
        .expect("query variants");
    assert!(variants.contains(&json!("calls_in_file")));
    for property in ["filePath", "owner", "method"] {
        assert!(
            query["inputSchema"]["properties"].get(property).is_some(),
            "missing calls_in_file input property {property}"
        );
    }
    for property in ["file", "owner", "method", "overloads", "overload_count"] {
        assert!(
            query["outputSchema"]["properties"].get(property).is_some(),
            "missing calls_in_file output property {property}"
        );
    }
}
