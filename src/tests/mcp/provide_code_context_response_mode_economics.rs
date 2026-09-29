//! Cache-identity and serialization measurements for batch response modes.

use crate::mcp::tools::dispatch_tools_call;
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
        .expect("provide_code_context response")
}

fn batch_arguments(root: &tempfile::TempDir, files: &[String], count: usize, mode: &str) -> Value {
    json!({
        "workspaceRoot": root.path(),
        "tokenizer": "o200k",
        "responseMode": mode,
        "files": files[..count]
            .iter()
            .enumerate()
            .map(|(index, file)| json!({
                "id": format!("file-{index}"),
                "filePath": file,
                "fidelity": "verbatim"
            }))
            .collect::<Vec<_>>()
    })
}

fn breaker(response: &Value) -> &str {
    response["result"]["_meta"]["cache_hints"]["breakpoints"][0]["breaker"]
        .as_str()
        .expect("outer baseline breaker")
}

#[test]
fn response_modes_have_distinct_reusable_cache_identities_and_measured_sizes() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let files: Vec<_> = (0..8)
        .map(|index| {
            let path = root.path().join(format!("mode-economics-{index}.ts"));
            let source = format!(
                "export class ModeEconomics{index} {{ value(): number {{ return {index}; }} }}\n{}",
                "// response-mode-economics-padding-0123456789abcdef\n".repeat(200)
            );
            std::fs::write(&path, source).expect("fixture");
            path.to_string_lossy().into_owned()
        })
        .collect();
    let state = state(&root);
    let modes = ["mirrored", "structured", "indexed"];

    let mut breakers = Vec::new();
    for mode in modes {
        let arguments = batch_arguments(&root, &files, 2, mode);
        let first = dispatch(&state, 200, arguments.clone());
        let first_breaker = breaker(&first).to_string();
        let hits_before_repeat = state.cache_metrics_lock().hits;
        let _repeated = dispatch(&state, 201, arguments);
        assert!(
            state.cache_metrics_lock().hits > hits_before_repeat,
            "{mode} repeat must reuse its mode-scoped cache identity"
        );
        breakers.push(first_breaker);
    }
    assert_ne!(breakers[0], breakers[1]);
    assert_ne!(breakers[0], breakers[2]);
    assert_ne!(breakers[1], breakers[2]);

    for count in [2, 4, 8] {
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
                        "fidelity": "verbatim"
                    }),
                );
                serde_json::to_vec(&single).expect("single JSON").len()
            })
            .sum();

        for mode in modes {
            let response = dispatch(
                &state,
                count as i64,
                batch_arguments(&root, &files, count, mode),
            );
            let batch_bytes = serde_json::to_vec(&response).expect("batch JSON").len();
            println!(
                "provide_code_context response-mode measurement: mode={mode} items={count} batch_bytes={batch_bytes} repeated_single_bytes={single_bytes}"
            );
        }
    }
}
