// src/mcp/tools.rs
//
// Tool definitions and dispatch for the MCP server.
//
// Phase 1 (FAANG audit F-03): the three `Fidelity::parse` call sites
// used to silently down-typo'd input to `Low`. They now use
// [`parse_fidelity_arg`], which returns a `-32602 Invalid params`
// JSON-RPC error on unrecognised input.
//
// Phase 2 (FAANG audit F-05): the dispatcher now takes `&mut McpState`
// (which bundles the path dict, cache, and project config) instead of
// separate dict/cache arguments. Tool handlers consult the user's
// `exclude_patterns` (via `is_excluded`) and `fidelity_overrides`
// (via `get_fidelity_for_extension`) before any file I/O.

use std::path::PathBuf;
use serde_json::Value;
use crate::compressor::{compress_file, Fidelity};
use crate::decompressor::Decompressor;
use crate::diff::{build_snapshot, diff_snapshots, format_diff, diff_summary};
use crate::mcp::McpState;
use crate::mcp::workspace;
use crate::protocol::send_response;

/// Return the list of tool definitions (for `tools/list`).
pub(crate) fn tool_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "compress_code_context",
            "description": "High-speed local AST compilation, hash-caching, and variable mapping tool.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts or .cs file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low' (max compression, ~85% reduction), 'medium' (balanced, preserves fields/async/markers, ~70-80%), 'high' (minimal compression, preserves most semantic depth, ~50-60%). Default: 'low'." }
                },
                "required": ["filePath"]
            }
        }),
        serde_json::json!({
            "name": "decompress_code_context",
            "description": "Expands a compressed structural skeleton back into human-readable format. Reverses opcodes ($c→class), path aliases, and behavior markers (⊕guard→'// conditional branch').",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "compressedText": { "type": "string", "description": "The compressed output from compress_code_context to expand." }
                },
                "required": ["compressedText"]
            }
        }),
        serde_json::json!({
            "name": "compress_workspace",
            "description": "Compresses all TypeScript/C# files in a directory tree. Outputs a manifest of compressed file signatures with shared opcode dictionary.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "directoryPath": { "type": "string", "description": "Absolute path to the project directory to scan." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'. Default: 'low'." }
                },
                "required": ["directoryPath"]
            }
        }),
        serde_json::json!({
            "name": "diff_code_context",
            "description": "AST-level diff compression. Returns only the structural deltas (added/removed/modified classes, methods, fields, imports) between the file's previous in-session snapshot and its current state. First call with no baseline stores the snapshot; subsequent calls emit a compact change-set using + / - / ~ / = markers.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "filePath": { "type": "string", "description": "Absolute path to .ts or .cs file." },
                    "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'. Default: 'low'." }
                },
                "required": ["filePath"]
            }
        }),
    ]
}

/// Parse the `fidelity` arg from a `tools/call` params object. On
/// success, returns the resolved `Fidelity`. On a parse error, sends
/// a `-32602 Invalid params` JSON-RPC response with the parse error
/// message and returns `Err(())` — the caller MUST then `return` from
/// the dispatch.
pub(crate) fn parse_fidelity_arg(id: &Value, params: &Value) -> Result<Fidelity, ()> {
    let fidelity_str = params["arguments"]["fidelity"].as_str().unwrap_or("low");
    match Fidelity::parse(fidelity_str) {
        Ok(f) => Ok(f),
        Err(e) => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": e.to_string() }
            }));
            Err(())
        }
    }
}

/// Resolve the effective fidelity for a `(explicit_arg, file_extension)`
/// pair, consulting the project config for an extension override.
///
/// F-05: the resolution order is:
///   1. The explicit `fidelity` arg (if present and valid).
///   2. `config.fidelity_overrides[ext]` (if present and valid).
///   3. The config's `default_fidelity` (if present and valid).
///   4. `Fidelity::Low` (hard fallback).
///
/// Steps 2 and 3 use `parse_or_default` (not `parse`) because a
/// typo in `.clean-ctx.json` should not be a hard error — it just
/// falls back to Low with a stderr warning.
pub(crate) fn resolve_fidelity(
    explicit: Option<&str>,
    ext: Option<&str>,
    config: &crate::config::CleanCtxConfig,
) -> Fidelity {
    if let Some(s) = explicit
        && let Ok(f) = Fidelity::parse(s)
    {
        return f;
    }
    if let Some(e) = ext
        && let Some(s) = config.get_fidelity_for_extension(e)
    {
        return Fidelity::parse_or_default(s);
    }
    Fidelity::parse_or_default(&config.default_fidelity)
}

/// Dispatch a `tools/call` request for the given tool name.
///
/// F-05: takes `&mut McpState` instead of separate dict/cache args.
pub(crate) fn dispatch_tools_call(
    id: &Value,
    tool_name: &str,
    params: &Value,
    state: &mut McpState,
) {
    match tool_name {
        "compress_code_context" => {
            let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
            let fidelity = match parse_fidelity_arg(id, params) {
                Ok(f) => f,
                Err(()) => return,
            };

            // F-05: consult `is_excluded` *before* any file I/O.
            if state.config.is_excluded(file_path_str) {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!("File excluded by config: {}", file_path_str)
                    }
                }));
                return;
            }

            // F-05: extension-based fidelity override (only kicks in
            // when the caller didn't pass an explicit `fidelity`).
            let explicit = params["arguments"]["fidelity"].as_str();
            let path_buf = PathBuf::from(file_path_str);
            let ext = path_buf.extension().and_then(|e| e.to_str());
            let effective_fidelity = if explicit.is_some() {
                fidelity
            } else {
                resolve_fidelity(explicit, ext, &state.config)
            };

            match compress_file(
                PathBuf::from(file_path_str),
                &mut state.dict,
                &mut state.cache,
                effective_fidelity,
            ) {
                Ok(mut compressed_text) => {
                    compressed_text.push_str(&state.dict.format_footer());

                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "content": [{ "type": "text", "text": compressed_text }] }
                    }));
                }
                Err(e) => {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32603, "message": e.to_string() }
                    }));
                }
            }
        }
        "decompress_code_context" => {
            let compressed_text = params["arguments"]["compressedText"].as_str().unwrap_or("");

            let mut decompressor = Decompressor::new();
            let decompressed = decompressor.quick_decompress(compressed_text);

            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "content": [{ "type": "text", "text": decompressed }] }
            }));
        }
        "compress_workspace" => {
            let dir_path = params["arguments"]["directoryPath"].as_str().unwrap_or(".");
            let fidelity = match parse_fidelity_arg(id, params) {
                Ok(f) => f,
                Err(()) => return,
            };

            match workspace::compress_workspace_dir(dir_path, fidelity, state) {
                Ok(manifest) => {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "content": [{ "type": "text", "text": manifest }] }
                    }));
                }
                Err(e) => {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32603, "message": e.to_string() }
                    }));
                }
            }
        }
        "diff_code_context" => {
            let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
            let fidelity = match parse_fidelity_arg(id, params) {
                Ok(f) => f,
                Err(()) => return,
            };

            // F-05: same exclusion check as `compress_code_context`.
            if state.config.is_excluded(file_path_str) {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!("File excluded by config: {}", file_path_str)
                    }
                }));
                return;
            }

            match diff_code_context_handler(
                PathBuf::from(file_path_str),
                &mut state.cache,
                fidelity,
            ) {
                Ok(output) => {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "content": [{ "type": "text", "text": output }] }
                    }));
                }
                Err(e) => {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32603, "message": e.to_string() }
                    }));
                }
            }
        }
        _ => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Tool not found: {}", tool_name) }
            }));
        }
    }
}

/// Compute an AST-level diff between the file's in-session baseline and
/// its current on-disk state.
fn diff_code_context_handler(
    file: PathBuf,
    cache: &mut crate::cache::LocalStateCache,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    let absolute_path = match std::fs::canonicalize(&file) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => file.to_string_lossy().into_owned(),
    };
    let cache_key = format!("{}::{}", absolute_path, fidelity as u8);

    let source = std::fs::read_to_string(&file)?;
    let current = build_snapshot(&source, fidelity)?;

    let baseline = cache.get_baseline(&cache_key).cloned();
    let body = match baseline {
        None => {
            let class_count = current.classes.len();
            cache.store_baseline(cache_key, current);
            format!(
                "// --- AST Diff ---\n// No baseline snapshot for this file yet.\n// Current state stored as baseline ({} classes).\n// Call diff_code_context again after the file changes to see the delta.",
                class_count
            )
        }
        Some(baseline_snap) => {
            let actions = diff_snapshots(&baseline_snap, &current);
            let (added, removed, modified, unchanged) = diff_summary(&actions);
            let header = format!(
                "// --- AST Diff: {} ---\n// +{} -{} ~{} ={} (classes/methods/fields/imports)\n",
                absolute_path, added, removed, modified, unchanged
            );
            let body = format_diff(&actions, fidelity);
            cache.store_baseline(cache_key, current);
            format!("{}{}", header, body)
        }
    };
    Ok(body)
}

#[cfg(test)]
#[path = "../tests/mcp/tools.rs"]
mod tests;
