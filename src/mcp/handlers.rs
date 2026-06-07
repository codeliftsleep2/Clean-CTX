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
    } else {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("Prompt not found: {}", prompt_name) }
        }));
    }
}