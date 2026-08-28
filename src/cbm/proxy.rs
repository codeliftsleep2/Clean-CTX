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

use crate::cbm::json_compress::compress_cbm_response;
use crate::mcp::McpState;
use crate::mcp::tool_helpers::count_tokens_with_tokenizer;
use crate::mcp::tools::parse_tokenizer_arg;
use crate::protocol::send_response;
use serde_json::Value;
use std::collections::HashSet;

/// The exact set of agent-facing CBM operations that `cbm_proxy` accepts.
///
/// These are CBM's native tool names. The proxy also accepts Clean-CTX
/// compatibility aliases (`graph_search`, `graph_query`, `graph_trace`),
/// which are normalized to their CBM equivalents before this check.
///
/// Internal helpers (`get_symbol_importance`, `get_dead_code`, etc.) are
/// NOT CBM tools and MUST be rejected here.
const CBM_ALLOWED_TOOLS: &[&str] = &[
    "search_graph",
    "query_graph",
    "trace_path",
    "get_architecture",
    "list_projects",
    "index_repository",
];

/// Reject a `cbm_tool` value that is not in the documented whitelist.
///
/// Returns `Some(error_message)` if the tool is disallowed, or `None` if it
/// is a known agent-facing CBM operation. This is factored out for direct
/// unit testing independent of the MCP response pipeline.
pub(crate) fn reject_disallowed_cbm_tool(tool: &str) -> Option<String> {
    let allowed: HashSet<&str> = CBM_ALLOWED_TOOLS.iter().copied().collect();
    if allowed.contains(tool) {
        return None;
    }
    Some(format!(
        "Unsupported CBM operation: '{tool}'. Supported operations: {}. \
         Note: get_symbol_importance, get_dead_code, and other \
         Clean-CTX internal helpers are not CBM proxy tools.",
        CBM_ALLOWED_TOOLS.join(", "),
    ))
}

/// Detect project-not-found errors in a raw JSON-RPC CBM response and
/// enhance the error message with a recovery hint.
///
/// CBM returns JSON-RPC error responses (valid JSON with an `error` key)
/// through the pipe-level proxy as `Ok(raw_text)` — not as `Err`. This
/// function intercepts the raw text before compression, checks for a
/// project-related error signature, and replaces the message with an
/// enhanced version that directs the agent to use `list_projects`.
///
/// Detection is intentionally conservative: the `error.message` must
/// mention "project" AND one of the known-not-found keywords. This avoids
/// adding the hint to unrelated CBM errors.
///
/// If the response is not a JSON-RPC error, or the error is not project-
/// related, the original text is returned unchanged.
pub(crate) fn enhance_project_not_found_error(raw: &str) -> String {
    let val: Value = match serde_json::from_str(raw.trim()) {
        Ok(v) => v,
        Err(_) => return raw.to_string(),
    };
    let error = match val.get("error") {
        Some(e) => e,
        None => return raw.to_string(),
    };
    let msg = match error["message"].as_str() {
        Some(m) => m,
        None => return raw.to_string(),
    };
    let lower = msg.to_lowercase();
    let is_project_error = lower.contains("project")
        && (lower.contains("not found")
            || lower.contains("does not exist")
            || lower.contains("unknown")
            || lower.contains("invalid"));
    if !is_project_error {
        return raw.to_string();
    }
    // Enhance the error message with a recovery hint.
    let hint = format!(
        "{}. Use list_projects to discover available projects and retry with a valid project name.",
        msg.trim_end_matches('.')
    );
    let mut enhanced = val;
    enhanced["error"]["message"] = Value::String(hint);
    serde_json::to_string(&enhanced).unwrap_or_else(|_| raw.to_string())
}

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
pub fn handle_cbm_proxy(id: &Value, params: &Value, state: &McpState) {
    let mut bridge_guard = state.graph_bridge_lock();
    let bridge = match &mut *bridge_guard {
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

    // Step 1: Extract the CBM tool name and its parameters.
    // IMPORTANT: `cbm_tool` is forwarded DIRECTLY to CBM, so it MUST use
    // CBM's actual tool names, NOT Clean-CTX's wrapper names:
    //   Clean-CTX tool   → CBM tool
    //   graph_search     → search_graph
    //   graph_query      → query_graph
    //   graph_trace      → trace_path
    //   get_architecture → get_architecture
    // `get_symbol_importance` and `get_dead_code` are NOT CBM tools — they
    // are implemented in Clean-CTX via query_graph Cypher, so they must
    // NOT be passed as cbm_tool.
    // Normalize Clean-CTX wrapper names to CBM's actual tool names so the
    // proxy is resilient to either convention.
    let raw_tool = params["arguments"]["cbm_tool"]
        .as_str()
        .unwrap_or("search_graph");
    let cbm_tool = match raw_tool {
        "graph_search" => "search_graph",
        "graph_query" => "query_graph",
        "graph_trace" => "trace_path",
        other => other,
    };

    // Whitelist validation: only the 6 documented agent-facing CBM operations
    // are permitted through the proxy. Reject unknown/internal tool names with
    // a clear structured error before any dispatch.
    if let Some(error_msg) = reject_disallowed_cbm_tool(cbm_tool) {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0", "id": id,
            "error": { "code": -32602, "message": error_msg }
        }));
        return;
    }

    // Build the parameters object for CBM.
    // When the caller passes an explicit `parameters` object, it is forwarded
    // as-is (with `project` merged in if missing) — the caller is responsible
    // for using CBM-native parameter names.
    // Otherwise, translate Clean-CTX shorthand args into CBM-native names:
    //   - search_graph:  query → name_pattern
    //   - query_graph:   query → query (same)
    //   - trace_path:    from → function_name, to → direction
    //   - get_architecture: project only
    let tool_params = match params["arguments"]["parameters"].as_object() {
        Some(obj) => {
            let mut merged = obj.clone();
            if !merged.contains_key("project") {
                if let Some(p) = params["arguments"]["project"].as_str() {
                    merged.insert("project".into(), Value::String(p.to_string()));
                }
            }
            Value::Object(merged)
        }
        None => {
            let mut default = serde_json::Map::new();
            match cbm_tool {
                "search_graph" => {
                    // CBM expects `name_pattern`; accept Clean-CTX `query` shorthand.
                    let name_pattern = params["arguments"]["name_pattern"]
                        .as_str()
                        .or_else(|| params["arguments"]["query"].as_str())
                        .unwrap_or("");
                    if !name_pattern.is_empty() {
                        default.insert(
                            "name_pattern".into(),
                            Value::String(name_pattern.to_string()),
                        );
                    }
                }
                "trace_path" => {
                    // CBM expects `function_name` + `direction` (inbound|outbound|both).
                    // Accept Clean-CTX `from`/`to` shorthand: from → function_name.
                    let function_name = params["arguments"]["function_name"]
                        .as_str()
                        .or_else(|| params["arguments"]["from"].as_str())
                        .unwrap_or("");
                    if !function_name.is_empty() {
                        default.insert(
                            "function_name".into(),
                            Value::String(function_name.to_string()),
                        );
                    }
                    if let Some(dir) = params["arguments"]["direction"].as_str() {
                        default.insert("direction".into(), Value::String(dir.to_string()));
                    }
                    if let Some(d) = params["arguments"]["depth"].as_u64() {
                        default.insert("depth".into(), Value::Number(d.into()));
                    }
                }
                _ => {
                    // query_graph, get_architecture, detect_changes, index_repository:
                    // `query` and `project` map directly to CBM names.
                    if let Some(q) = params["arguments"]["query"].as_str() {
                        default.insert("query".into(), Value::String(q.to_string()));
                    }
                }
            }
            if let Some(p) = params["arguments"]["project"].as_str() {
                default.insert("project".into(), Value::String(p.to_string()));
            }
            Value::Object(default)
        }
    };

    let args = tool_params;

    // Step 2: Forward to CBM via pipe — intercept the raw response text.
    //
    // The indexing gate must resolve against the project actually being queried
    // (never a stale active-project entry), and project-independent calls such
    // as `list_projects` must NOT be gated at all.
    if let Some(target_project) = resolve_proxy_target_project(bridge, params, &args) {
        if !crate::cbm::handlers::ensure_indexed_or_error_for(id, bridge, &target_project) {
            return;
        }
    }
    let raw_response = match bridge.proxy_call(cbm_tool, args) {
        Ok(text) => {
            // Intercept before compression: check for project-not-found CBM
            // errors and enhance them with a recovery hint. CBM-level errors
            // arrive as Ok(raw_json) because the pipe I/O succeeded — the
            // error is in the JSON content, not the transport status.
            enhance_project_not_found_error(&text)
        }
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
    // state.cbm_status is Arc - cannot mutate

    // Step 3: Compress the intercepted response with JSON-aware compressor
    // RC-1 fix: JSON compressor handles key shortening, not tree-sitter
    // MEDIUM fix: Use pluggable tokenizer for accurate token counts
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();

    match compress_cbm_response(&raw_response) {
        Some(compressed) => {
            // Use pluggable tokenizer for accurate counts (not byte-based estimate)
            let raw_tokens = count_tokens_with_tokenizer(&raw_response, tokenizer_ref);
            let comp_tokens =
                count_tokens_with_tokenizer(&compressed.compressed_text, tokenizer_ref);

            // Record CBM pipe-level interception savings. This ACCUMULATES
            // across calls (unlike per-file compression which overwrites),
            // so the dashboard reflects total CBM output saved this session.
            state.record_cbm_proxy(cbm_tool, raw_tokens, comp_tokens);

            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": compressed.compressed_text }],
                    "_meta": {
                        "cbm_tool": cbm_tool,
                        "raw_bytes": compressed.raw_bytes,
                        "raw_tokens": raw_tokens,
                        "compressed_tokens": comp_tokens,
                        "savings_pct": if raw_tokens > 0 {
                            (1.0 - comp_tokens as f64 / raw_tokens as f64) * 100.0
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
            let raw_tokens = count_tokens_with_tokenizer(&raw_response, tokenizer_ref);
            let comp_tokens = count_tokens_with_tokenizer(&min_compressed, tokenizer_ref);

            // Record minimum-compression savings too (fallback path previously
            // skipped stats entirely).
            state.record_cbm_proxy(cbm_tool, raw_tokens, comp_tokens);

            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "result": {
                    "content": [{ "type": "text", "text": min_compressed }],
                    "_meta": {
                        "cbm_tool": cbm_tool,
                        "raw_bytes": raw_response.len(),
                        "raw_tokens": raw_tokens,
                        "compressed_tokens": comp_tokens,
                        "savings_pct": if raw_tokens > 0 {
                            (1.0 - comp_tokens as f64 / raw_tokens as f64) * 100.0
                        } else { 0.0 },
                        "cbm_status": state.cbm_status.summary(),
                        "compression_fallback": true,
                    }
                }
            }));
        }
    }
}

/// Resolve the project a `cbm_proxy` call actually targets, or `None` for
/// project-independent tools (e.g. `list_projects`).
///
/// Priority: CBM-native `parameters.project` (already merged into `tool_params`)
/// → Clean-CTX `arguments.project` → `arguments.workspaceRoot` (a repo path).
/// Every value is canonicalized to the authoritative CBM project slug so a
/// proxy call can never gate on an unrelated/raw project name.
pub(crate) fn resolve_proxy_target_project(
    bridge: &crate::cbm::GraphBridge,
    params: &Value,
    tool_params: &Value,
) -> Option<String> {
    if let Some(p) = tool_params.get("project").and_then(|v| v.as_str()) {
        return Some(bridge.resolve_project_id(p));
    }
    if let Some(p) = params["arguments"]["project"].as_str() {
        return Some(bridge.resolve_project_id(p));
    }
    if let Some(root) = params["arguments"]["workspaceRoot"].as_str() {
        return Some(bridge.resolve_project_id(root));
    }
    None
}

/// RC-2 fallback: minimum compression when JSON compressor fails.
/// Strips all non-essential whitespace and shortens common JSON keys.
pub(crate) fn apply_minimum_compression(raw: &str) -> String {
    // First attempt: parse as JSON and shorten keys
    if let Ok(val) = serde_json::from_str::<Value>(raw.trim()) {
        if let Some(result) = val.get("result") {
            return serde_json::to_string(result).unwrap_or_else(|_| {
                // Last resort: compress as raw text
                raw.chars().filter(|c| !c.is_whitespace()).collect()
            });
        }
        return serde_json::to_string(&val)
            .unwrap_or_else(|_| raw.chars().filter(|c| !c.is_whitespace()).collect());
    }

    // Last resort: strip all whitespace
    raw.chars().filter(|c| !c.is_whitespace()).collect()
}
