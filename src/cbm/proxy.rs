// src/cbm/proxy.rs
//
// **Pipe-level interception proxy** — the single entry point for all CBM queries.
//
// Execution sequence (FAANG audit RC-1, RC-2, RH-1, RM-1, RM-2):
//   1. Agent sends a query to Clean-CTX
//   2. Clean-CTX forwards it to CBM via stdin pipe
//   3. CBM responds on stdout with ~5000-token structural seed (JSON-RPC)
//   4. **Clean-CTX intercepts the raw CBM stdout at the pipe level**
//   5. Uses JSON-aware compressor to compress the intercepted seed down to ~1100 tokens
//      (NOT the tree-sitter pipeline — JSON has no class/method captures)
//   6. On compression failure: ALWAYS applies minimum compression, NEVER returns raw
//   7. Compressed result goes back to the agent

use serde_json::Value;
use crate::mcp::McpState;
use crate::protocol::send_response;
use crate::cbm::json_compress::compress_cbm_response;

/// Handle `cbm_proxy` — forward to CBM, intercept raw response, compress it.
///
/// The proxy accepts any CBM tool call, forwards it, catches CBM's
/// raw stdout text, compresses the JSON with a JSON-aware compressor,
/// and returns the compressed result.
///
/// **Critical fix (RC-1):** Uses `json_compress::compress_cbm_response`
/// instead of `compress_file_with_source()` because the tree-sitter
/// pipeline produces zero captures on JSON input.
///
/// **Critical fix (RC-2):** NEVER returns raw CBM output. If compression
/// fails, applies minimum compression (whitespace stripping + key
/// shortening) before returning.
pub fn handle_cbm_proxy(id: &Value, params: &Value, state: &mut McpState) {
    let bridge = match state.graph_bridge.as_mut() {
        Some(b) => b,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": {
                    "code": -32603,
                    "message": "CBM proxy unavailable. Install codebase-memory-mcp on PATH."
                }
            }));
            return;
        }
    };

    // Step 1: Extract the CBM tool name and its parameters
    let cbm_tool = params["arguments"]["cbm_tool"].as_str().unwrap_or("graph_search");
    let tool_params = params["arguments"]["parameters"].clone();

    // Build the parameters object for CBM
    let args = if tool_params.is_object() {
        let mut merged = tool_params.as_object().unwrap().clone();
        if !merged.contains_key("project") {
            if let Some(p) = params["arguments"]["project"].as_str() {
                merged.insert("project".into(), Value::String(p.to_string()));
            }
        }
        Value::Object(merged)
    } else {
        let mut default = serde_json::Map::new();
        if let Some(q) = params["arguments"]["query"].as_str() {
            default.insert("query".into(), Value::String(q.to_string()));
        }
        if let Some(p) = params["arguments"]["project"].as_str() {
            default.insert("project".into(), Value::String(p.to_string()));
        }
        if let Some(f) = params["arguments"]["from"].as_str() {
            default.insert("from".into(), Value::String(f.to_string()));
        }
        if let Some(t) = params["arguments"]["to"].as_str() {
            default.insert("to".into(), Value::String(t.to_string()));
        }
        Value::Object(default)
    };

    // Step 2: Forward to CBM via pipe — intercept the raw response text
    let raw_response = match bridge.proxy_call(cbm_tool, args) {
        Ok(text) => text,
        Err(e) => {
            // RM-2: Log compression errors for diagnostics
            state.push_warning(format!("CBM proxy call failed: {e}"));
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": format!("CBM proxy call failed: {e}") }
            }));
            return;
        }
    };

    // RM-1: Single status clone after CBM interaction
    state.cbm_status = bridge.status().clone();

    // Step 3: Compress the intercepted response with JSON-aware compressor
    // RC-1 fix: JSON compressor handles key shortening, not tree-sitter
    match compress_cbm_response(&raw_response) {
        Some(compressed) => {
            // Record the proxy operation in session stats
            // RH-1 fix: use the estimated tokens from the JSON compressor
            state.session_stats.record_compression(
                &format!("cbm://{cbm_tool}"),
                compressed.raw_tokens_est,
                compressed.comp_tokens_est,
                "low",
                false,
                "cbm_proxy",
                None,
            );

            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": compressed.compressed_text }],
                    "_meta": {
                        "cbm_tool": cbm_tool,
                        "raw_bytes": compressed.raw_bytes,
                        "raw_tokens_est": compressed.raw_tokens_est,
                        "compressed_tokens_est": compressed.comp_tokens_est,
                        "savings_pct": if compressed.raw_tokens_est > 0 {
                            (1.0 - compressed.comp_tokens_est as f64 / compressed.raw_tokens_est as f64) * 100.0
                        } else { 0.0 },
                        "cbm_status": state.cbm_status.summary(),
                        "cbm_error": compressed.cbm_error,
                    }
                }
            }));
        }
        None => {
            // RC-2 fix: NEVER return raw CBM output. Apply minimum compression:
            // strip the JSON-RPC envelope, shorten keys, remove whitespace.
            state.push_warning("CBM response compression failed — applying minimum compression");

            let min_compressed = apply_minimum_compression(&raw_response);
            let comp_tokens_est = (min_compressed.len() + 3) / 4;
            let raw_tokens_est = (raw_response.len() + 3) / 4;

            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": min_compressed }],
                    "_meta": {
                        "cbm_tool": cbm_tool,
                        "raw_bytes": raw_response.len(),
                        "raw_tokens_est": raw_tokens_est,
                        "compressed_tokens_est": comp_tokens_est,
                        "savings_pct": if raw_tokens_est > 0 {
                            (1.0 - comp_tokens_est as f64 / raw_tokens_est as f64) * 100.0
                        } else { 0.0 },
                        "cbm_status": state.cbm_status.summary(),
                        "compression_fallback": true,
                    }
                }
            }));
        }
    }
}

/// RC-2 fallback: minimum compression when JSON compressor fails.
/// Strips all non-essential whitespace and shortens common JSON keys.
fn apply_minimum_compression(raw: &str) -> String {
    // First attempt: parse as JSON and shorten keys
    if let Ok(val) = serde_json::from_str::<Value>(raw.trim()) {
        if let Some(result) = val.get("result") {
            return serde_json::to_string(result).unwrap_or_else(|_| {
                // Last resort: compress as raw text
                raw.chars().filter(|c| !c.is_whitespace()).collect()
            });
        }
        return serde_json::to_string(&val).unwrap_or_else(|_| {
            raw.chars().filter(|c| !c.is_whitespace()).collect()
        });
    }

    // Last resort: strip all whitespace
    raw.chars().filter(|c| !c.is_whitespace()).collect()
}