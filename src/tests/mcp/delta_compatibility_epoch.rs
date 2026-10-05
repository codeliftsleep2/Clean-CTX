use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn config(
    root: &tempfile::TempDir,
    configure: impl FnOnce(&mut crate::config::CleanCtxConfig),
) -> crate::config::CleanCtxConfig {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.auto_save = true;
    config.persistence.db_path = root
        .path()
        .join("delta-compatibility-epoch.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    configure(&mut config);
    config
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn establish_baseline(
    state: &crate::mcp::McpState,
    root: &tempfile::TempDir,
    path: &std::path::Path,
    source: &str,
) -> String {
    std::fs::write(path, source).expect("baseline source");
    let file = path.to_string_lossy().into_owned();
    let response = dispatch(
        state,
        1,
        "provide_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path(),
            "fidelity": "high"
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
    file
}

fn generate_delta(
    state: &crate::mcp::McpState,
    root: &tempfile::TempDir,
    file: &str,
    target: &str,
) -> (Value, Value) {
    std::fs::write(file, target).expect("target source");
    state.invalidate_source_cache(file);
    let response = dispatch(
        state,
        2,
        "delta_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path(),
            "fidelity": "high"
        }),
    );
    assert_eq!(response["result"]["strategy"], "delta", "{response}");
    (
        response["result"]["delta"].clone(),
        response["result"]["from_version"].clone(),
    )
}

fn durable_version(state: &crate::mcp::McpState, file: &str) -> u64 {
    let store = state.persistence_store_lock();
    let sqlite = store
        .as_ref()
        .expect("persistence")
        .sqlite()
        .expect("SQLite");
    sqlite
        .load_durable_context(file, None)
        .expect("durable load")
        .expect("durable context")
        .ir
        .version
}

#[test]
fn matching_canonical_epoch_accepts_delta_and_advances_chain() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("matching-epoch.ts");
    let state = crate::mcp::McpState::new(config(&root, |_| {}));
    let file = establish_baseline(
        &state,
        &root,
        &path,
        "export class Matching { first(): void {} }\n",
    );
    let (delta, from) = generate_delta(
        &state,
        &root,
        &file,
        "export class Matching { second(): void {} }\n",
    );
    let target_version = delta["to"].as_u64().expect("target version");

    let applied = dispatch(
        &state,
        3,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": from }),
    );

    assert!(applied.get("error").is_none(), "{applied}");
    assert_eq!(durable_version(&state, &file), target_version);
}

#[test]
fn new_compatible_baseline_establishes_replacement_epoch() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("replacement-epoch.ts");
    let initial = crate::mcp::McpState::new(config(&root, |_| {}));
    let file = establish_baseline(
        &initial,
        &root,
        &path,
        "export class Replacement { first(): UserId { return 1 as UserId; } }\n",
    );
    drop(initial);

    let replacement = crate::mcp::McpState::new(config(&root, |config| {
        config
            .type_aliases
            .insert("uid".to_string(), "UserId".to_string());
    }));
    let established = dispatch(
        &replacement,
        4,
        "compress_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": root.path(),
            "fidelity": "high"
        }),
    );
    assert!(established.get("error").is_none(), "{established}");

    let (delta, from) = generate_delta(
        &replacement,
        &root,
        &file,
        "export class Replacement { second(): UserId { return 2 as UserId; } }\n",
    );
    let target_version = delta["to"].as_u64().expect("target version");
    let applied = dispatch(
        &replacement,
        5,
        "apply_delta",
        json!({ "delta": delta, "currentVersion": from }),
    );

    assert!(applied.get("error").is_none(), "{applied}");
    assert_eq!(durable_version(&replacement, &file), target_version);
}
