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
//   2. Compacts current MCP JSON without losing provider truth.
//   3. Preserves tree text and independent MCP channels.
// Historical bare enrichment objects retain their legacy compact notation.
//
// Current wire compression prioritizes faithful provider evidence over a
// fixed savings ratio.

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
///   2. Retain RPC/tool errors and all accompanying evidence
///   3. Unwrap only a sole unannotated text channel
///   4. Compact JSON losslessly or preserve tree text verbatim
///   5. Return compressed text with metadata
pub fn compress_cbm_response(raw_text: &str) -> Option<CompressedCbmResponse> {
    let raw_bytes = raw_text.len();
    let raw_tokens_est = raw_bytes.div_ceil(4);

    // Step 1: Parse the JSON-RPC response
    let parsed: Value = match serde_json::from_str(raw_text.trim()) {
        Ok(v) => v,
        Err(_) => return None,
    };

    // The current MCP boundary is lossless: compact JSON keeps positional
    // cells, keys, null/empty values, cursors and independent result channels.
    let cbm_error = parsed
        .get("error")
        .and_then(|e| e["message"].as_str())
        .map(str::to_owned)
        .or_else(|| {
            parsed.get("result").and_then(|result| {
                match crate::cbm::client::check_soft_error("provider", result) {
                    Err(crate::cbm::client::CbmError::ToolError { message, .. }) => Some(message),
                    _ => None,
                }
            })
        });
    let compressed = if let Some(error) = parsed.get("error") {
        format!("§E: {}", serde_json::to_string(error).ok()?)
    } else if let Some(result) = parsed.get("result") {
        // Unwrap only a sole unannotated text block. Additional content,
        // structuredContent, isError and metadata remain distinct channels.
        let single_text = result
            .as_object()
            .filter(|map| map.len() == 1)
            .and_then(|_| result["content"].as_array())
            .filter(|blocks| blocks.len() == 1)
            .and_then(|blocks| {
                let block = blocks[0].as_object()?;
                if block.keys().any(|key| key != "type" && key != "text") {
                    return None;
                }
                if block.get("type").is_some_and(|kind| kind != "text") {
                    return None;
                }
                block.get("text")?.as_str()
            });
        if let Some(text) = single_text {
            match serde_json::from_str::<Value>(text) {
                Ok(value) => serde_json::to_string(&value).ok()?,
                // Tree columns/indentation and opaque tokens are data.
                Err(_) => text.to_owned(),
            }
        } else {
            serde_json::to_string(result).ok()?
        }
    } else {
        // Historical internal enrichment compression is not an MCP response.
        // Keep its existing notation/savings contract; admitted wire responses
        // always take the result/error branches above.
        compress_json_body(raw_text)
    };

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
    let stripped: String = body
        .chars()
        .filter(|c| !c.is_whitespace() || *c == ' ' || *c == '\n')
        .collect();
    stripped
}

/// Recursively compress a JSON value with key shortening.
fn compress_value(val: &Value) -> String {
    match val {
        Value::Object(map) => {
            let entries: Vec<String> = map
                .iter()
                .filter(|(_, v)| !v.is_null()) // strip nulls (RM-2)
                .map(|(k, v)| {
                    let key = shorten_key(k);
                    let val_str = compress_value(v);
                    if val_str.is_empty() {
                        return String::new();
                    }
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
            let entries: Vec<String> = arr
                .iter()
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
#[path = "../tests/cbm/json_compress.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/cbm/proxy_truth.rs"]
mod truth_tests;
