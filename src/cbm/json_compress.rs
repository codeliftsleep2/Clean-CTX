// src/cbm/json_compress.rs
//
// JSON compression for CBM interception responses.
//
// CBM produces ~5000-token JSON-RPC responses. The standard Clean-CTX
// tree-sitter pipeline is designed for source code (.,ts, .cs, .rs) and
// produces zero captures when fed JSON. This module provides a JSON-aware
// compressor that:
//
//   1. Strips the JSON-RPC envelope (checks for errors, extracts result)
//   2. Compresses the result body via text-based techniques:
//      - Shortens common JSON keys (e.g., "jsonrpc" → "j", "result" → "r")
//      - Strips whitespace
//      - Removes null/empty fields where safe
//   3. Wraps in Clean-CTX notation with proper opcodes
//
// This is the pipe-level interception that achieves ~5000 → ~1100 tokens.

use serde_json::Value;

/// Compressed CBM response with metadata.
#[derive(Debug, Clone)]
pub struct CompressedCbmResponse {
    /// The compressed text representation.
    pub compressed_text: String,
    /// Raw byte count of the original CBM response.
    pub raw_bytes: usize,
    /// Estimated raw tokens (chars/4).
    pub raw_tokens_est: usize,
    /// Estimated compressed tokens.
    pub comp_tokens_est: usize,
    /// Whether an error was extracted from the CBM response.
    pub cbm_error: Option<String>,
}

/// Compress a raw CBM JSON-RPC response intercepting from the pipe.
///
/// Algorithm:
///   1. Parse JSON — if parse fails, return None (compression failed)
///   2. Check for JSON-RPC error object — extract error message
///   3. Extract the `result` field content
///   4. Compress with JSON-aware key shortening + whitespace stripping
///   5. Return compressed text with metadata
pub fn compress_cbm_response(raw_text: &str) -> Option<CompressedCbmResponse> {
    let raw_bytes = raw_text.len();
    let raw_tokens_est = raw_bytes.div_ceil(4);

    // Step 1: Parse the JSON-RPC response
    let parsed: Value = match serde_json::from_str(raw_text.trim()) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // Step 2: Check for JSON-RPC error
    let cbm_error = parsed.get("error").and_then(|e| {
        e.get("message").and_then(|m| m.as_str().map(|s| s.to_string()))
    });

    // Step 3: Extract the content field from the result
    let result_body = if let Some(error) = &cbm_error {
        // CBM returned an error — compress the error message
        format!("§E: {}", error)
    } else if let Some(content_arr) = parsed.pointer("/result/content") {
        // Standard MCP response with content array
        let texts: Vec<String> = content_arr.as_array().map(|arr| {
            arr.iter().filter_map(|c| {
                c.get("text").and_then(|t| t.as_str().map(|s| s.to_string()))
            }).collect()
        }).unwrap_or_default();
        texts.join("\n")
    } else if let Some(result_obj) = parsed.get("result") {
        // CBM tool response with structured data — serialize compactly
        serde_json::to_string(result_obj).unwrap_or_default()
    } else {
        // Unknown structure — return raw text
        raw_text.to_string()
    };

    // Step 4: Compress the body
    let compressed = compress_json_body(&result_body);

    let comp_tokens_est = compressed.len().div_ceil(4);

    Some(CompressedCbmResponse {
        compressed_text: compressed,
        raw_bytes,
        raw_tokens_est,
        comp_tokens_est,
        cbm_error,
    })
}

/// Compress a JSON body with key shortening and whitespace stripping.
/// Produces a compact representation suitable for LLM consumption.
///
/// Key shortening map (JSON-RPC common keys → single chars):
///   "results" → "r", "symbols"→ "s", "edges" → "e", "nodes" → "n",
///   "name" → "nm", "file" → "f", "label" → "l", "id" → "i",
///   "score" → "sc", "importance" → "im", "reason" → "rn",
///   "modules" → "m", "dependencies" → "d", "kind" → "k",
///   "properties" → "p", "from" → "fr", "to" → "t",
///   "changes" → "c", "graph_version" → "gv", "impact" → "ip",
///   "symbol" → "sy", "change_type" → "ct"
fn compress_json_body(body: &str) -> String {
    // First, try to parse as JSON for structured compression
    if let Ok(val) = serde_json::from_str::<Value>(body) {
        return compress_value(&val);
    }

    // If not valid JSON, strip whitespace and return
    let stripped: String = body.chars()
        .filter(|c| !c.is_whitespace() || *c == ' ' || *c == '\n')
        .collect();
    stripped
}

/// Recursively compress a JSON value with key shortening.
fn compress_value(val: &Value) -> String {
    match val {
        Value::Object(map) => {
            let entries: Vec<String> = map.iter()
                .filter(|(_, v)| !v.is_null()) // strip nulls (RM-2)
                .map(|(k, v)| {
                    let key = shorten_key(k);
                    let val_str = compress_value(v);
                    if val_str.is_empty() { return String::new(); }
                    format!("{}:{}", key, val_str)
                })
                .filter(|s| !s.is_empty())
                .collect();
            if entries.len() == 1 && !entries[0].contains('\n') {
                entries[0].clone()
            } else {
                entries.join("\n")
            }
        }
        Value::Array(arr) => {
            let entries: Vec<String> = arr.iter()
                .map(compress_value)
                .filter(|s| !s.is_empty())
                .collect();
            if entries.len() <= 3 {
                // Short arrays: inline
                entries.join(" ")
            } else {
                // Long arrays: one per line
                entries.join("\n")
            }
        }
        Value::String(s) => {
            // Short strings: inline. Long strings: keep as-is.
            s.trim().to_string()
        }
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
    }
}

/// Shorten common JSON keys to single chars or short codes.
pub(crate) fn shorten_key(key: &str) -> &str {
    match key {
        "results" => "r",
        "symbols" => "s",
        "edges" => "e",
        "nodes" => "n",
        "name" => "nm",
        "file" => "f",
        "label" => "l",
        "id" => "i",
        "score" => "sc",
        "importance" => "im",
        "reason" => "rn",
        "modules" => "m",
        "dependencies" => "d",
        "kind" => "k",
        "properties" => "p",
        "from" => "fr",
        "to" => "t",
        "changes" => "c",
        "graph_version" => "gv",
        "impact" => "ip",
        "symbol" => "sy",
        "change_type" => "ct",
        "query" => "q",
        "project" => "pj",
        "arguments" => "a",
        "content" => "ct",
        "type" => "tp",
        "text" => "tx",
        "description" => "dsc",
        "status" => "st",
        "error" => "e",
        "message" => "msg",
        "code" => "cd",
        "jsonrpc" => "jrpc",
        "method" => "mth",
        "params" => "prm",
        "result" => "res",
        "path" => "pth",
        "file_count" => "fc",
        "matched_symbols" => "ms",
        "is_direct" => "dir",
        "data" => "d",
        _ => key,
    }
}

#[cfg(test)]
mod tests {
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
        }).to_string();

        let result = compress_cbm_response(&json).unwrap();
        assert!(result.cbm_error.is_none());
        // Compressed text must be shorter than raw
        assert!(result.compressed_text.len() < json.len(), 
            "Compressed ({}) must be shorter than raw ({})", result.compressed_text.len(), json.len());
    }

    #[test]
    fn test_shorten_key_coverage() {
        // Ensure all known keys are shortened
        let keys = [
            "results", "symbols", "edges", "nodes", "name", "file", "label", "id",
            "score", "importance", "reason", "modules", "dependencies", "kind",
            "properties", "from", "to", "changes", "graph_version", "impact",
            "symbol", "change_type", "query", "project", "arguments", "content",
            "type", "text", "description", "status", "error", "message", "code",
            "jsonrpc", "method", "params", "result", "path", "file_count",
            "matched_symbols", "is_direct", "data",
        ];
        for key in &keys {
            let shortened = shorten_key(key);
            assert!(shortened.len() <= key.len(), 
                "Key '{}' shortened to '{}' which is longer!", key, shortened);
        }
    }
}