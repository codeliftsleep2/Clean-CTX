//! Contract regression for the model-facing provider boundary.
//!
//! `provide_code_context` must return a complete current representation even
//! when an IR baseline exists and the compatibility `auto_delta` flag is true.
//! Structured delta transport is opt-in through `delta_code_context`.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

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
        .expect("registered handler response")
}

#[test]
fn changed_follow_up_provide_remains_complete_when_auto_delta_is_enabled() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("complete-provider.ts");
    let file = path.to_string_lossy().into_owned();
    let mut config = crate::tests::test_config();
    config.auto_delta = true;
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": root.path().to_string_lossy()
        })
    };
    let source = |method: &str| {
        format!(
            "export class CompleteProvider {{ {method}(): number {{ return 1; }} }}\n{}",
            "// economics-padding-0123456789abcdef\n".repeat(1_000)
        )
    };

    std::fs::write(&path, source("before")).expect("baseline source");
    let baseline = dispatch(&state, 1, args());
    assert_eq!(baseline["result"]["_meta"]["strategy"], "full");

    std::fs::write(&path, source("after")).expect("changed source");
    state.invalidate_source_cache(&file);
    let follow_up = dispatch(&state, 2, args());
    assert_eq!(follow_up["result"]["_meta"]["strategy"], "full");
    assert!(follow_up["result"]["_meta"].get("delta").is_none());
    assert!(
        follow_up["result"]["content"][0]["text"]
            .as_str()
            .expect("complete model-visible content")
            .contains("M after"),
        "changed declaration must be present: {follow_up}"
    );
}

#[test]
fn unchanged_follow_up_provide_reuses_the_compiled_ir_version() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("unchanged-provider.ts");
    let file = path.to_string_lossy().into_owned();
    let source = format!(
        "export class UnchangedProvider {{ value(): number {{ return 1; }} }}\n{}",
        "// unchanged-economics-padding-0123456789abcdef\n".repeat(1_000)
    );
    std::fs::write(&path, source).expect("source fixture");

    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let args = || {
        json!({
            "filePath": file.clone(),
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "high"
        })
    };

    let baseline = dispatch(&state, 1, args());
    let follow_up = dispatch(&state, 2, args());

    assert_eq!(
        follow_up["result"]["_meta"]["version"], baseline["result"]["_meta"]["version"],
        "byte-identical source must reuse the existing compiled IR: {follow_up}"
    );
    assert_eq!(
        follow_up["result"]["content"][0]["text"], baseline["result"]["content"][0]["text"],
        "cache reuse must preserve the complete model-visible response"
    );
}

#[test]
fn provide_uses_the_tokenizer_scoped_raw_token_cache() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("token-cache-provider.ts");
    let file = path.to_string_lossy().into_owned();
    let source = format!(
        "export class TokenCacheProvider {{ value(): number {{ return 1; }} }}\n{}",
        "// token-cache-padding-0123456789abcdef\n".repeat(1_000)
    );
    std::fs::write(&path, &source).expect("source fixture");

    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let content_hash = state.cache_read().compute_hash(source.as_bytes());
    let cache_key = format!("{content_hash}::o200k");
    state.cache_write().store_raw_token_count(&cache_key, 7);

    let response = dispatch(
        &state,
        1,
        json!({
            "filePath": file,
            "workspaceRoot": root.path().to_string_lossy(),
            "fidelity": "edit",
            "tokenizer": "o200k"
        }),
    );
    assert!(response.get("error").is_none(), "{response}");

    let stats = state.session_stats_lock();
    let file_stats = stats
        .file_stats(path.to_string_lossy().as_ref())
        .expect("provide must record file statistics");
    assert_eq!(
        file_stats.raw_tokens, 7,
        "both token-economics stages must consume the cached raw count"
    );
}
