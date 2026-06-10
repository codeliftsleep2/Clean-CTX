// src/mcp/handlers.rs
//
// JSON-RPC method handlers for the MCP server.
// Each function corresponds to a single method and sends its own response.

use serde_json::Value;
use crate::protocol::send_response;
use crate::mcp::tools;
use crate::mcp::prompts;

/// Handle `initialize` — returns server capabilities and protocol version.
pub(crate) fn handle_initialize(id: &Value) {
    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2025-11-25",
            "capabilities": { "tools": {}, "prompts": {} },
            "serverInfo": { "name": "clean-ctx", "version": "1.0.0" }
        }
    }));
}

/// Handle `tools/list` — returns the list of available tools.
pub(crate) fn handle_tools_list(id: &Value) {
    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": tools::tool_list()
        }
    }));
}

/// Handle `prompts/list` — returns the list of available prompts.
pub(crate) fn handle_prompts_list(id: &Value) {
    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "prompts": prompts::prompt_list()
        }
    }));
}

/// Handle `prompts/get` — returns the content of a specific prompt.
pub(crate) fn handle_prompts_get(id: &Value, prompt_name: &str) {
    if prompt_name == "cleanctx-notation" {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "description": "System instructions for reading and writing Clean-CTX compressed notation",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": prompts::SYSTEM_PROMPT
                        }
                    }
                ]
            }
        }));
    } else if prompt_name == "dashboard" {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "description": "View the Clean-CTX token savings dashboard",
                "messages": [
                    {
                        "role": "user",
                        "content": {
                            "type": "text",
                            "text": concat!(
                                "You have access to the Clean-CTX Dashboard. To view token savings and compression stats:\n\n",
                                "- Call `context_stats` with no arguments to see the full session dashboard.\n",
                                "- Call `context_stats` with a `filePath` to see stats for a specific file.\n",
                                "- Use `format: \"json\"` for structured data, or `format: \"text\"` for human-readable output.\n\n",
                                "The dashboard shows:\n",
                                "- Session duration and file count\n",
                                "- Total raw vs compressed tokens and savings percentage\n",
                                "- Full compression vs delta operation counts\n",
                                "- Per-file breakdown with version history and delta counts\n",
                                "- Delta hit rate (how often deltas were used instead of full re-compression)\n",
                            )
                        }
                    }
                ]
            }
        }));
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Prompt not found: {}", prompt_name) }
        }));
    }
}