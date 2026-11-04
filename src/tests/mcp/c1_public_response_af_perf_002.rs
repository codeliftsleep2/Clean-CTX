//! Frozen original public result oracle, including independently validated cache breakers.
use serde_json::{Value, json};

fn normalized_result(mut result: Value, root: &std::path::Path) -> Value {
    let actual_text = result["content"][0]["text"].as_str().unwrap();
    let actual_breaker = crate::mcp::cache_hints::compute_baseline_breaker(actual_text);
    for bp in result["_meta"]["cache_hints"]["breakpoints"]
        .as_array()
        .unwrap()
    {
        assert_eq!(
            bp["breaker"], actual_breaker,
            "actual opaque breaker must match actual unnormalized content"
        );
    }
    fn normalize(value: &mut Value, root: &str) {
        match value {
            Value::String(text) => *text = text.replace(root, "<ROOT>"),
            Value::Array(values) => {
                for item in values {
                    normalize(item, root);
                }
            }
            Value::Object(values) => {
                for item in values.values_mut() {
                    normalize(item, root);
                }
            }
            _ => {}
        }
    }
    normalize(&mut result, &root.to_string_lossy());
    let breaker = crate::mcp::cache_hints::compute_baseline_breaker(
        result["content"][0]["text"].as_str().unwrap(),
    );
    for bp in result["_meta"]["cache_hints"]["breakpoints"]
        .as_array_mut()
        .unwrap()
    {
        bp["breaker"] = json!(breaker);
    }
    result
}

#[test]
fn afperf_c1_public_original_responses_and_raw_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    let fixtures: Value =
        serde_json::from_str(include_str!("fixtures/c1_public_responses.json")).unwrap();
    for (n, tokenizer) in [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, "o200k"))
        .chain([(256, "llama3")])
    {
        let root = tempfile::tempdir().unwrap();
        let source = format!(
            "// {}\nexport class Workload {{ run() {{ return 1; }} }}",
            "ordinary source words ".repeat(n)
        );
        let file = root.path().join("workload.ts");
        std::fs::write(&file, &source).unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(1),
            "compress_code_context",
            &json!({"arguments":{
                "filePath":file,"workspaceRoot":root.path(),"fidelity":"high","tokenizer":tokenizer
            }}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let actual = normalized_result(response["result"].clone(), root.path());
        let mut expected = fixtures[n.to_string()].clone();
        if tokenizer == "llama3" {
            expected["content"][0]["text"] = json!(source);
            expected["content_kind"] = json!("raw_passthrough");
            expected["byte_exact"] = json!(["document"]);
            let breaker = crate::mcp::cache_hints::compute_baseline_breaker(&source);
            for bp in expected["_meta"]["cache_hints"]["breakpoints"]
                .as_array_mut()
                .unwrap()
            {
                bp["breaker"] = json!(breaker);
            }
        }
        assert_eq!(
            actual, expected,
            "complete public result n={n} tokenizer={tokenizer}; only temp-root-derived values normalized"
        );
    }
}
