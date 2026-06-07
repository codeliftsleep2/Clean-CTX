// src/main.rs — Clean-CTX MCP Server
// Enterprise Token Waste Reducer & Context Compiler

use serde_json::json;
use std::io::{self, BufRead};
use std::path::PathBuf;
use clean_ctx::protocol::{JsonRpcRequest, send_response};
use clean_ctx::compressor::{compress_file, Fidelity};
use clean_ctx::dictionary::PathDictionary;
use clean_ctx::cache::LocalStateCache;
use clean_ctx::decompressor::Decompressor;
use clean_ctx::config::CleanCtxConfig;
use clean_ctx::diff::{build_snapshot, diff_snapshots, format_diff, diff_summary};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = String::new();

    // Persistent state tracking registries
    let mut structural_dict = PathDictionary::new();
    let mut session_cache = LocalStateCache::new();
    // Load project config (best-effort, falls back to defaults)
    let _config = CleanCtxConfig::load(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    while handle.read_line(&mut buffer)? > 0 {
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            buffer.clear();
            continue;
        }

        if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
            match req.method.as_str() {
                "initialize" => {
                    send_response(&json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "protocolVersion": "2025-11-25",
                            "capabilities": { "tools": {}, "prompts": {} },
                            "serverInfo": { "name": "clean-ctx", "version": "1.0.0" }
                        }
                    }));
                }
                "tools/list" => {
                    send_response(&json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "tools": [
                                {
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
                                },
                                {
                                    "name": "decompress_code_context",
                                    "description": "Expands a compressed structural skeleton back into human-readable format. Reverses opcodes ($c→class), path aliases, and behavior markers (⊕guard→'// conditional branch').",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "compressedText": { "type": "string", "description": "The compressed output from compress_code_context to expand." }
                                        },
                                        "required": ["compressedText"]
                                    }
                                },
                                {
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
                                },
                                {
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
                                }
                            ]
                        }
                    }));
                }
                "prompts/list" => {
                    send_response(&json!({
                        "jsonrpc": "2.0",
                        "id": req.id,
                        "result": {
                            "prompts": [
                                {
                                    "name": "cleanctx-notation",
                                    "description": "System instructions for reading and writing Clean-CTX compressed notation",
                                    "arguments": []
                                }
                            ]
                        }
                    }));
                }
                "prompts/get" => {
                    let prompt_name = req.params.as_ref()
                        .and_then(|p| p["name"].as_str())
                        .unwrap_or("");

                    if prompt_name == "cleanctx-notation" {
                        let system_prompt = concat!(
                            "# Clean-CTX Notation Guide\n\n",
                            "You are working with Clean-CTX compressed code notation. ",
                            "This is an AST-based compression format that strips implementation details while preserving structural signatures.\n\n",
                            "## Opcode Reference\n\n",
                            "### Built-in Primitives (always available)\n",
                            "| Opcode | Token | Opcode | Token | Opcode | Token |\n",
                            "|--------|-------|--------|-------|--------|-------|\n",
                            "| $c | class | $s | string | $b | boolean |\n",
                            "| $n | number | $v | void | $a | async |\n",
                            "| $e | export | $r | return | $t | throw |\n",
                            "| $T | true | $F | false | $P | Promise |\n",
                            "| $ctor | constructor | $fn | function | $E | Error |\n",
                            "| $nw | new | $i | if | $fr | for |\n",
                            "| $w | while | $h | this | $k | const |\n",
                            "| $l | let | $pu | public | $pv | private |\n",
                            "| $st | static | $x | extends | $m | implements |\n",
                            "| $if | interface | $ty | type | $nl | null |\n",
                            "| $ud | undefined | $fm | from | $im | import |\n\n",
                            "### Custom Opcodes\n",
                            "Custom opcodes ($1, $2, ...) are auto-assigned to tokens appearing 2+ times in the session. ",
                            "Check the §SYM footer for custom opcode definitions.\n\n",
                            "### Path Aliases\n",
                            "File paths are compressed to α1, α2, β1, etc. Check §MAP footer for path mappings.\n\n",
                            "## Behavior Markers\n",
                            "| Marker | Meaning |\n",
                            "|--------|---------|\n",
                            "| ⊕guard | Conditional branch (if statement) |\n",
                            "| ⊕loop | Iteration (for/while) |\n",
                            "| ⊕⇒ | Return value follows |\n",
                            "| ⊕! | Throws error |\n",
                            "| ⊕export | Module export |\n\n",
                            "## Diff Markers (from diff_code_context)\n",
                            "| Marker | Meaning |\n",
                            "|--------|---------|\n",
                            "| + | Added (new class, method, field, or import) |\n",
                            "| - | Removed |\n",
                            "| ~ | Modified (signature or markers changed) |\n",
                            "| = | Unchanged (included for scope context) |\n\n",
                            "## Rules for Using Compressed Notation\n",
                            "1. When reading compressed context, interpret opcodes using the tables above\n",
                            "2. When writing code in compressed form, use the opcodes and markers\n",
                            "3. NEVER output raw opcode tables or §MAP/§SYM footers — those are internal metadata\n",
                            "4. When asked to expand, use the decompress_code_context tool\n",
                            "5. When asked for changes between versions, use the diff_code_context tool — it returns only the deltas\n",
                            "6. Preserve the semantic meaning — compressed ≠ less accurate\n",
                            "7. Use the same fidelity level as the compressed context you received\n\n",
                            "## Example\n",
                            "Compressed: `$c UserService;$ctor();$a process(payload: $s[]): $P<$b>`\n",
                            "Interpreted: `class UserService; constructor(); async process(payload: string[]): Promise<boolean>`\n",
                            "Write back as: `$c UserService { $ctor() $a process(payload: $s[]): $P<$b> }`\n",
                            "Diff:        `~ class UserService\\n  ~ method process: process(payload: $s[]): $P<$b>\\n    was: process(payload: $s[]): $P<$s>`\n",
                        );
                        send_response(&json!({
                            "jsonrpc": "2.0",
                            "id": req.id,
                            "result": {
                                "description": "System instructions for reading and writing Clean-CTX compressed notation",
                                "messages": [
                                    {
                                        "role": "user",
                                        "content": {
                                            "type": "text",
                                            "text": system_prompt
                                        }
                                    }
                                ]
                            }
                        }));
                    } else {
                        send_response(&json!({
                            "jsonrpc": "2.0",
                            "id": req.id,
                            "error": { "code": -32601, "message": format!("Prompt not found: {}", prompt_name) }
                        }));
                    }
                }
                "tools/call" => {
                    if let (Some(ref id), Some(params)) = (req.id, req.params) {
                        let tool_name = params["name"].as_str().unwrap_or("");

                        match tool_name {
                            "compress_code_context" => {
                                let file_path_str = params["arguments"]["filePath"].as_str().unwrap_or("");
                                let fidelity_str = params["arguments"]["fidelity"].as_str().unwrap_or("low");
                                let fidelity = Fidelity::from_str(fidelity_str);

                                match compress_file(PathBuf::from(file_path_str), &mut structural_dict, &mut session_cache, fidelity) {
                                    Ok(mut compressed_text) => {
                                        compressed_text.push_str(&structural_dict.format_footer());

                                        send_response(&json!({
                                            "jsonrpc": "2.0",
                                            "id": id,
                                            "result": { "content": [{ "type": "text", "text": compressed_text }] }
                                        }));
                                    }
                                    Err(e) => {
                                        send_response(&json!({
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

                                send_response(&json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "content": [{ "type": "text", "text": decompressed }] }
                                }));
                            }
                            "compress_workspace" => {
                                let dir_path = params["arguments"]["directoryPath"].as_str().unwrap_or(".");
                                let fidelity_str = params["arguments"]["fidelity"].as_str().unwrap_or("low");
                                let fidelity = Fidelity::from_str(fidelity_str);

                                match compress_workspace_dir(dir_path, fidelity) {
                                    Ok(manifest) => {
                                        send_response(&json!({
                                            "jsonrpc": "2.0",
                                            "id": id,
                                            "result": { "content": [{ "type": "text", "text": manifest }] }
                                        }));
                                    }
                                    Err(e) => {
                                        send_response(&json!({
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

                                match diff_code_context(
                                    PathBuf::from(file_path_str),
                                    &mut session_cache,
                                    fidelity,
                                ) {
                                    Ok(output) => {
                                        send_response(&json!({
                                            "jsonrpc": "2.0",
                                            "id": id,
                                            "result": { "content": [{ "type": "text", "text": output }] }
                                        }));
                                    }
                                    Err(e) => {
                                        send_response(&json!({
                                            "jsonrpc": "2.0",
                                            "id": id,
                                            "error": { "code": -32603, "message": e.to_string() }
                                        }));
                                    }
                                }
                            }
                            _ => {
                                send_response(&json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "error": { "code": -32601, "message": format!("Tool not found: {}", tool_name) }
                                }));
                            }
                        }
                    }
                }
                _ => {
                    if let Some(id) = req.id {
                        send_response(&json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "Method not found" } }));
                    }
                }
            }
        }
        buffer.clear();
    }
    Ok(())
}

/// Compute an AST-level diff between the file's in-session baseline and
/// its current on-disk state. If no baseline exists, the current state is
/// stored as the new baseline and the response indicates that. Otherwise
/// the previous baseline is compared against the current snapshot, the
/// change-set is rendered, and the baseline is rotated to the current
/// snapshot for the next call.
fn diff_code_context(
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
            // First call for this file at this fidelity: no comparison to
            // make. Persist the snapshot and tell the caller to call again
            // after editing the file.
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
            // Rotate baseline so the next call diffs against *this* version.
            cache.store_baseline(cache_key, current);
            format!("{}{}", header, body)
        }
    };
    Ok(body)
}

/// Scan a directory for .ts/.cs files and compress each one
fn compress_workspace_dir(
    dir_path: &str,
    fidelity: Fidelity,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut dict = PathDictionary::new();
    let mut cache = LocalStateCache::new();
    let mut manifest = String::new();
    manifest.push_str("// Clean-CTX Workspace Manifest\n");
    manifest.push_str(&format!("// Directory: {}\n", dir_path));
    manifest.push_str(&format!("// Fidelity: {:?}\n\n", fidelity));

    let mut entries: Vec<String> = Vec::new();

    // Collect all .ts and .cs files recursively
    collect_source_files(dir_path, &mut entries);
    entries.sort();

    for entry in &entries {
        match compress_file(PathBuf::from(entry), &mut dict, &mut cache, fidelity) {
            Ok(mut compressed) => {
                compressed.push_str(&dict.format_footer());
                manifest.push_str(&format!("// ===== FILE: {} =====\n", entry));
                manifest.push_str(&compressed);
                manifest.push('\n');
            }
            Err(e) => {
                manifest.push_str(&format!("// ERROR compressing {}: {}\n\n", entry, e));
            }
        }
    }

    // Append the global path map
    manifest.push_str(&dict.format_footer());

    Ok(manifest)
}

/// Recursively collect .ts and .cs files from a directory
fn collect_source_files(dir: &str, entries: &mut Vec<String>) {
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();

            // Skip hidden dirs, node_modules, target, etc.
            if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" {
                continue;
            }

            if path.is_dir() {
                collect_source_files(&path.to_string_lossy(), entries);
            } else if path.is_file() {
                let ext = path.extension().unwrap_or_default().to_string_lossy();
                if ext == "ts" || ext == "js" || ext == "cs" {
                    entries.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }
}
