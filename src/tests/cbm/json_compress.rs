use super::*;

#[test]
fn test_compress_invalid_json_returns_none() {
    let result = compress_cbm_response("not json at all{{{");
    assert!(result.is_none());
}

#[test]
fn test_compress_empty_object() {
    let result = compress_cbm_response("{}").unwrap();
    assert!(result.cbm_error.is_none());
}

#[test]
fn test_compress_simple_response() {
    let json = r#"{"jsonrpc":"2.0","id":1,"result":{"name":"test"}}"#;
    let result = compress_cbm_response(json).unwrap();
    assert!(result.cbm_error.is_none());
    assert!(result.raw_bytes > 0);
    assert!(result.compressed_text.len() < result.raw_bytes);
}

#[test]
fn test_compress_extracts_error() {
    let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#;
    let result = compress_cbm_response(json).unwrap();
    assert_eq!(result.cbm_error, Some("Method not found".into()));
    assert!(result.compressed_text.contains("§E:"));
}

#[test]
fn test_compress_with_content_array() {
    let json = r#"{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"Hello from CBM"}]}}"#;
    let result = compress_cbm_response(json).unwrap();
    assert!(result.compressed_text.contains("Hello"));
}

#[test]
fn test_compress_graph_search_response() {
    // Simulate a real CBM graph_search response (simplified)
    let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "results": [
                    {"id": "1", "name": "UserService", "file": "src/user.rs", "label": "Class"},
                    {"id": "2", "name": "PaymentGateway", "file": "src/payment.rs", "label": "Class"}
                ]
            }
        }"#;
    let result = compress_cbm_response(json).unwrap();
    assert!(result.cbm_error.is_none());
    assert!(!result.compressed_text.is_empty());
    // Should be significantly shorter than the original
    assert!(result.compressed_text.len() < json.len());
}

#[test]
fn test_compress_large_response_not_returned_raw() {
    // RC-2 regression: even large responses must be compressed
    let mut items = Vec::new();
    for i in 0..100 {
        items.push(serde_json::json!({"name": format!("Symbol_{}", i), "file": "src/test.rs"}));
    }
    let json = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": { "results": items }
    })
    .to_string();

    let result = compress_cbm_response(&json).unwrap();
    assert!(result.cbm_error.is_none());
    // Compressed text must be shorter than raw
    assert!(
        result.compressed_text.len() < json.len(),
        "Compressed ({}) must be shorter than raw ({})",
        result.compressed_text.len(),
        json.len()
    );
}

#[test]
fn test_shorten_key_coverage() {
    // Ensure all known keys are shortened
    let keys = [
        "results",
        "symbols",
        "edges",
        "nodes",
        "name",
        "file",
        "label",
        "id",
        "score",
        "importance",
        "reason",
        "modules",
        "dependencies",
        "kind",
        "properties",
        "from",
        "to",
        "changes",
        "graph_version",
        "impact",
        "symbol",
        "change_type",
        "query",
        "project",
        "arguments",
        "content",
        "type",
        "text",
        "description",
        "status",
        "error",
        "message",
        "code",
        "jsonrpc",
        "method",
        "params",
        "result",
        "path",
        "file_count",
        "matched_symbols",
        "is_direct",
        "data",
    ];
    for key in &keys {
        let shortened = shorten_key(key);
        assert!(
            shortened.len() <= key.len(),
            "Key '{}' shortened to '{}' which is longer!",
            key,
            shortened
        );
    }
}
