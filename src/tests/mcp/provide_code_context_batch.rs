//! RED contract regressions for batched `provide_code_context` requests.

use crate::mcp::tools::{dispatch_tools_call, tool_list};
use crate::tests::assert_valid_mcp_envelope;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn dispatch(state: &crate::mcp::McpState, id: i64, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(id),
        "provide_code_context",
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered provide_code_context response")
}

fn result_items(response: &Value) -> &Vec<Value> {
    let result = response["result"].as_object().expect("batch MCP result");
    assert_valid_mcp_envelope(result);
    assert_eq!(result["structuredContent"]["batch"], true, "{response}");
    result["structuredContent"]["results"]
        .as_array()
        .expect("ordered batch results")
}

fn content_text(response: &Value, index: usize) -> &str {
    response["result"]["content"][index]["text"]
        .as_str()
        .expect("model-visible text block")
}

fn write_fixture(root: &tempfile::TempDir, name: &str, owner: &str, method: &str) -> String {
    let path = root.path().join(name);
    let source = format!(
        "export class {owner} {{\n  {method}(): number {{\n    return 41 + 1;\n  }}\n}}\n{}",
        "// batch-economics-padding-0123456789abcdef\n".repeat(200)
    );
    std::fs::write(&path, source).expect("TypeScript fixture");
    path.to_string_lossy().into_owned()
}

#[test]
fn red_batch_schema_declares_single_or_files_contract_and_ordered_outcomes() {
    let provide = tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "provide_code_context")
        .expect("provide_code_context tool");
    let input = &provide["inputSchema"];
    let branches = input["oneOf"].as_array().expect("request-form union");
    assert_eq!(branches.len(), 2, "{provide}");

    let batch = branches
        .iter()
        .find(|branch| {
            branch["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|field| field == "files"))
        })
        .expect("batch request branch");
    let files = &batch["properties"]["files"];
    assert_eq!(files["type"], "array");
    assert_eq!(files["maxItems"], 8);
    let item = &files["items"];
    for field in ["id", "filePath", "intent", "fidelity", "focusMethods"] {
        assert!(item["properties"].get(field).is_some(), "missing {field}");
    }
    for shared in ["workspaceRoot", "tokenizer"] {
        assert!(
            item["properties"].get(shared).is_none(),
            "item owns {shared}"
        );
    }

    let output = &provide["outputSchema"]["properties"];
    assert_eq!(output["results"]["type"], "array");
    for field in ["id", "status", "content_index", "content", "meta", "error"] {
        assert!(
            output["results"]["items"]["properties"]
                .get(field)
                .is_some(),
            "missing output {field}"
        );
    }
}

#[test]
fn red_batch_mixed_modes_match_equivalent_legacy_single_calls() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let high = write_fixture(&root, "overview.ts", "Overview", "inspect");
    let edit = write_fixture(&root, "edit.ts", "Editable", "change");
    let workspace = root.path().to_string_lossy().into_owned();

    let batch = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": workspace,
            "tokenizer": "o200k",
            "files": [
                { "id": "overview", "filePath": high, "fidelity": "high" },
                {
                    "id": "edit", "filePath": edit, "fidelity": "edit",
                    "focusMethods": ["Editable.change"]
                }
            ]
        }),
    );
    assert!(batch.get("error").is_none(), "{batch}");
    let items = result_items(&batch);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["id"], "overview");
    assert_eq!(items[0]["status"], "ok");
    assert_eq!(items[0]["content_index"], 0);
    assert_eq!(items[1]["id"], "edit");
    assert_eq!(items[1]["status"], "ok");
    assert_eq!(items[1]["content_index"], 1);

    let singles = state(&root);
    let single_high = dispatch(
        &singles,
        2,
        json!({
            "workspaceRoot": root.path(), "tokenizer": "o200k",
            "filePath": high, "fidelity": "high"
        }),
    );
    let single_edit = dispatch(
        &singles,
        3,
        json!({
            "workspaceRoot": root.path(), "tokenizer": "o200k",
            "filePath": edit, "fidelity": "edit",
            "focusMethods": ["Editable.change"]
        }),
    );
    assert_eq!(content_text(&batch, 0), content_text(&single_high, 0));
    assert_eq!(content_text(&batch, 1), content_text(&single_edit, 0));
    assert_eq!(
        items[0]["meta"]["content_kind"],
        single_high["result"]["_meta"]["content_kind"]
    );
    assert_eq!(
        items[1]["meta"]["content_kind"],
        single_edit["result"]["_meta"]["content_kind"]
    );
}

#[test]
fn red_batch_isolates_failure_and_preserves_exact_content_indexes() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let first = write_fixture(&root, "first.ts", "First", "one");
    let third = write_fixture(&root, "third.ts", "Third", "three");
    let first_source = std::fs::read_to_string(&first).expect("first source");
    let third_source = std::fs::read_to_string(&third).expect("third source");
    let response = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": root.path(),
            "files": [
                { "id": "first", "filePath": first, "fidelity": "verbatim" },
                { "id": "missing", "filePath": root.path().join("missing.ts"), "fidelity": "high" },
                { "id": "third", "filePath": third, "fidelity": "verbatim" }
            ]
        }),
    );

    assert!(response.get("error").is_none(), "{response}");
    let items = result_items(&response);
    assert_eq!(items.len(), 3);
    assert_eq!(items[0]["status"], "ok");
    assert_eq!(items[0]["content_index"], 0);
    assert_eq!(items[1]["status"], "error");
    assert_eq!(items[1]["error"]["code"], -32602);
    assert!(items[1].get("content_index").is_none(), "{response}");
    assert_eq!(items[2]["status"], "ok");
    assert_eq!(items[2]["content_index"], 1);
    assert_eq!(content_text(&response, 0), first_source);
    assert_eq!(content_text(&response, 1), third_source);
    assert!(
        response["result"]["_meta"]["cache_hints"].is_object(),
        "batch must own one outer cache hint: {response}"
    );
}

#[test]
fn red_batch_rejects_later_duplicate_canonical_file_as_item_error() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_fixture(&root, "duplicate.ts", "Duplicate", "value");
    let response = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": root.path(),
            "files": [
                { "id": "first", "filePath": file, "fidelity": "edit", "focusMethods": ["value"] },
                { "id": "duplicate", "filePath": "duplicate.ts", "fidelity": "high" }
            ]
        }),
    );

    let items = result_items(&response);
    assert_eq!(items[0]["status"], "ok", "{response}");
    assert_eq!(items[1]["status"], "error", "{response}");
    assert_eq!(items[1]["error"]["code"], -32602);
    let message = items[1]["error"]["message"].as_str().expect("message");
    assert!(message.contains("duplicate"), "{response}");
    assert!(message.contains("focusMethods"), "{response}");
}

#[test]
fn red_batch_structure_errors_reject_before_item_execution() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let state = state(&root);
    let nine: Vec<_> = (0..9)
        .map(|index| json!({ "id": format!("f{index}"), "filePath": format!("f{index}.ts") }))
        .collect();
    let cases = [
        (json!({ "files": [] }), "non-empty"),
        (json!({ "files": {} }), "array"),
        (json!({ "files": [{ "filePath": "a.ts" }] }), "id"),
        (
            json!({ "files": [{ "id": "same", "filePath": "a.ts" }, { "id": "same", "filePath": "b.ts" }] }),
            "duplicate",
        ),
        (
            json!({ "filePath": "a.ts", "files": [{ "id": "a", "filePath": "a.ts" }] }),
            "mutually exclusive",
        ),
        (
            json!({ "files": [{ "id": "a", "filePath": "a.ts", "workspaceRoot": root.path() }] }),
            "workspaceRoot",
        ),
        (json!({ "files": nine }), "8"),
    ];

    for (arguments, reason) in cases {
        let response = dispatch(&state, 1, arguments);
        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert!(
            response["error"]["message"]
                .as_str()
                .is_some_and(|message| message.contains(reason)),
            "batch rejection must report {reason:?}: {response}"
        );
        assert!(response.get("result").is_none(), "{response}");
    }
}

#[test]
fn red_batch_all_item_failures_keep_a_valid_nonempty_mcp_envelope() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let response = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": root.path(),
            "files": [
                { "id": "missing-a", "filePath": "missing-a.ts" },
                { "id": "missing-b", "filePath": "missing-b.ts" }
            ]
        }),
    );

    let items = result_items(&response);
    assert_eq!(items.len(), 2, "{response}");
    assert!(items.iter().all(|item| item["status"] == "error"));
    assert!(
        items.iter().all(|item| item.get("content_index").is_none()),
        "fallback content must not be attributed to an item: {response}"
    );
    assert_eq!(
        content_text(&response, 0),
        "No context items succeeded; inspect structuredContent.results for item errors."
    );
    assert_eq!(
        response["result"]["content"]
            .as_array()
            .expect("nonempty MCP content")
            .len(),
        1,
        "{response}"
    );
}

#[test]
fn red_batch_successes_mirror_exact_content_in_each_structured_item() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let first = write_fixture(&root, "mirrored-first.ts", "MirroredFirst", "one");
    let third = write_fixture(&root, "mirrored-third.ts", "MirroredThird", "three");
    let first_source = std::fs::read_to_string(&first).expect("first source");
    let third_source = std::fs::read_to_string(&third).expect("third source");
    let response = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": root.path(),
            "files": [
                { "id": "first", "filePath": first, "fidelity": "verbatim" },
                { "id": "missing", "filePath": "missing.ts", "fidelity": "high" },
                { "id": "third", "filePath": third, "fidelity": "verbatim" }
            ]
        }),
    );

    let items = result_items(&response);
    assert_eq!(items[0]["content"][0]["type"], "text", "{response}");
    assert_eq!(items[0]["content"][0]["text"], first_source, "{response}");
    assert_eq!(items[0]["content_index"], 0, "{response}");
    assert_eq!(items[1]["status"], "error", "{response}");
    assert!(items[1].get("content").is_none(), "{response}");
    assert_eq!(items[2]["content"][0]["type"], "text", "{response}");
    assert_eq!(items[2]["content"][0]["text"], third_source, "{response}");
    assert_eq!(items[2]["content_index"], 1, "{response}");
}

#[test]
fn batch_preserves_token_accounting_cache_reuse_and_records_envelope_sizes() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let files: Vec<_> = (0..8)
        .map(|index| {
            write_fixture(
                &root,
                &format!("economics-{index}.ts"),
                &format!("Economics{index}"),
                "inspect",
            )
        })
        .collect();
    let state = state(&root);
    let tokenizer = crate::tokenizer::create_tokenizer(crate::tokenizer::TokenizerKind::O200k)
        .expect("o200k tokenizer");

    let batch_arguments = |count: usize| {
        json!({
            "workspaceRoot": root.path(),
            "tokenizer": "o200k",
            "files": files[..count]
                .iter()
                .enumerate()
                .map(|(index, file)| json!({
                    "id": format!("file-{index}"),
                    "filePath": file,
                    "fidelity": "high"
                }))
                .collect::<Vec<_>>()
        })
    };

    let first = dispatch(&state, 1, batch_arguments(8));
    let hits_before_repeat = state.cache_metrics_lock().hits;
    let repeated = dispatch(&state, 1, batch_arguments(8));
    assert_eq!(first["result"]["content"], repeated["result"]["content"]);
    assert_eq!(
        first["result"]["structuredContent"],
        repeated["result"]["structuredContent"]
    );
    assert!(
        state.cache_metrics_lock().hits > hits_before_repeat,
        "the repeated outer batch breakpoint must reuse the same cache identity"
    );

    let results = result_items(&first);
    let stats = state.session_stats_lock();
    for (index, file) in files.iter().enumerate() {
        let source = std::fs::read_to_string(file).expect("economics source");
        let content_index = results[index]["content_index"]
            .as_u64()
            .expect("successful content index") as usize;
        let output = content_text(&first, content_index);
        let file_stats = stats.file_stats(file).expect("per-file token statistics");
        assert_eq!(file_stats.raw_tokens, tokenizer.count_tokens(&source));
        assert_eq!(file_stats.compressed_tokens, tokenizer.count_tokens(output));
    }
    drop(stats);

    for count in [2, 4, 8] {
        let batch = dispatch(&state, count as i64, batch_arguments(count));
        let batch_bytes = serde_json::to_vec(&batch).expect("batch JSON").len();
        let single_bytes: usize = files[..count]
            .iter()
            .enumerate()
            .map(|(index, file)| {
                let single = dispatch(
                    &state,
                    100 + index as i64,
                    json!({
                        "workspaceRoot": root.path(),
                        "tokenizer": "o200k",
                        "filePath": file,
                        "fidelity": "high"
                    }),
                );
                serde_json::to_vec(&single).expect("single JSON").len()
            })
            .sum();
        println!(
            "provide_code_context envelope measurement: items={count} batch_bytes={batch_bytes} repeated_single_bytes={single_bytes}"
        );
    }
}

#[test]
fn red_response_mode_schema_declares_batch_only_policy_and_resolved_discriminator() {
    let provide = tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "provide_code_context")
        .expect("provide_code_context tool");
    let input = &provide["inputSchema"];
    assert_eq!(
        input["properties"]["responseMode"]["enum"],
        json!(["mirrored", "structured", "indexed"]),
        "{provide}"
    );
    let single = input["oneOf"]
        .as_array()
        .expect("request union")
        .iter()
        .find(|branch| {
            branch["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|field| field == "filePath"))
        })
        .expect("single branch");
    let forbidden = single["not"]["anyOf"]
        .as_array()
        .expect("single-form forbidden fields");
    assert!(
        forbidden.iter().any(|rule| {
            rule["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|field| field == "responseMode"))
        }),
        "single request must forbid responseMode: {single}"
    );
    assert_eq!(
        provide["outputSchema"]["properties"]["response_mode"]["enum"],
        json!(["mirrored", "structured", "indexed"]),
        "{provide}"
    );
}

#[test]
fn red_omitted_response_mode_resolves_to_safe_mirrored_shape() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_fixture(&root, "default-mode.ts", "DefaultMode", "read");
    let response = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": root.path(),
            "files": [{ "id": "default", "filePath": file, "fidelity": "verbatim" }]
        }),
    );
    let items = result_items(&response);
    assert_eq!(
        response["result"]["structuredContent"]["response_mode"], "mirrored",
        "{response}"
    );
    assert_eq!(items[0]["content_index"], 0, "{response}");
    assert_eq!(
        items[0]["content"], response["result"]["content"],
        "{response}"
    );
}

#[test]
fn red_structured_response_mode_emits_code_once_in_structured_items() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_fixture(&root, "structured-mode.ts", "StructuredMode", "read");
    let source = std::fs::read_to_string(&file).expect("source");
    let response = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": root.path(),
            "responseMode": "structured",
            "files": [{ "id": "structured", "filePath": file, "fidelity": "verbatim" }]
        }),
    );
    let items = result_items(&response);
    assert_eq!(
        response["result"]["structuredContent"]["response_mode"], "structured",
        "{response}"
    );
    assert_eq!(
        content_text(&response, 0),
        "Batch context is available in structuredContent.results; inspect every item status and content."
    );
    assert_eq!(response["result"]["content"].as_array().unwrap().len(), 1);
    assert_eq!(items[0]["content"][0]["text"], source, "{response}");
    assert!(items[0].get("content_index").is_none(), "{response}");
}

#[test]
fn red_indexed_response_mode_emits_code_once_in_top_level_content() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_fixture(&root, "indexed-mode.ts", "IndexedMode", "read");
    let source = std::fs::read_to_string(&file).expect("source");
    let response = dispatch(
        &state(&root),
        1,
        json!({
            "workspaceRoot": root.path(),
            "responseMode": "indexed",
            "files": [{ "id": "indexed", "filePath": file, "fidelity": "verbatim" }]
        }),
    );
    let items = result_items(&response);
    assert_eq!(
        response["result"]["structuredContent"]["response_mode"], "indexed",
        "{response}"
    );
    assert_eq!(content_text(&response, 0), source);
    assert_eq!(items[0]["content_index"], 0, "{response}");
    assert!(items[0].get("content").is_none(), "{response}");
}

#[test]
fn red_response_mode_rejects_unknown_values_and_singular_requests() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let file = write_fixture(&root, "invalid-mode.ts", "InvalidMode", "read");
    let state = state(&root);
    let cases = [
        json!({
            "workspaceRoot": root.path(),
            "responseMode": "unknown",
            "files": [{ "id": "invalid", "filePath": file }]
        }),
        json!({
            "workspaceRoot": root.path(),
            "responseMode": "structured",
            "filePath": file
        }),
    ];
    for arguments in cases {
        let response = dispatch(&state, 1, arguments);
        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert!(
            response["error"]["message"]
                .as_str()
                .is_some_and(|message| message.contains("responseMode")),
            "{response}"
        );
    }
}
