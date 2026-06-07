// src/mcp/tools.rs
//
// Tool definitions and dispatch for the MCP server.

use std::path::PathBuf;
use serde_json::Value;
use crate::compressor::{compress_file, Fidelity};
use crate::dictionary::PathDictionary;
use crate::cache::LocalStateCache;
use crate::decompressor::Decompressor;
use crate::diff::{build_snapshot, diff_snapshots, format_diff, diff_summary};
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

/// Dispatch a `tools/call` request for the given tool name.
pub(crate) fn dispatch_tools_call(
    id: &Value,
    tool_name: &str,
    params: &Value,
    structural_dict: &mut PathDictionary,
    session_cache: &mut LocalStateCache,
) {
    match tool_name {
        "compress_code_context" => {
            let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
            let fidelity_str = params["arguments"]["fidelity"].as_str().unwrap_or("low");
            let fidelity = Fidelity::from_str(fidelity_str);

            match compress_file(PathBuf::from(file_path_str), structural_dict, session_cache, fidelity) {
                Ok(mut compressed_text) => {
                    compressed_text.push_str(&structural_dict.format_footer());

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
            let fidelity_str = params["arguments"]["fidelity"].as_str().unwrap_or("low");
            let fidelity = Fidelity::from_str(fidelity_str);

            match crate::mcp::workspace::compress_workspace_dir(dir_path, fidelity) {
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
            let fidelity_str = params["arguments"]["fidelity"].as_str().unwrap_or("low");
            let fidelity = Fidelity::from_str(fidelity_str);

            match diff_code_context_handler(
                PathBuf::from(file_path_str),
                session_cache,
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
    cache: &mut LocalStateCache,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    let absolute_path = match std::fs::canonicalize(&file) {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => file.to_string_lossy().into_owned(),
    };
    let cache_key = format!("{}::{:?}", absolute_path, fidelity);

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