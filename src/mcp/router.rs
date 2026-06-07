// src/mcp/router.rs
//
// JSON-RPC method dispatcher. Routes incoming requests to the appropriate
// handler based on the method name.

use crate::cache::LocalStateCache;
use crate::dictionary::PathDictionary;
use crate::mcp::handlers;
use crate::mcp::tools;
use crate::protocol::send_response;

/// Dispatch an incoming JSON-RPC request to the appropriate handler.
pub(crate) fn dispatch(
    req: &crate::protocol::JsonRpcRequest,
    structural_dict: &mut PathDictionary,
    session_cache: &mut LocalStateCache,
) {
    match req.method.as_str() {
        "initialize" => {
            if let Some(ref id) = req.id {
                handlers::handle_initialize(id);
            }
        }
        "tools/list" => {
            if let Some(ref id) = req.id {
                handlers::handle_tools_list(id);
            }
        }
        "prompts/list" => {
            if let Some(ref id) = req.id {
                handlers::handle_prompts_list(id);
            }
        }
        "prompts/get" => {
            if let Some(ref id) = req.id {
                match req.params.as_ref() {
                    Some(params) => {
                        let prompt_name = params["name"].as_str().unwrap_or("");
                        handlers::handle_prompts_get(id, prompt_name);
                    }
                    None => {
                        send_response(&serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": { "code": -32602, "message": "Missing params" }
                        }));
                    }
                }
            }
        }
        "tools/call" => {
            if let (Some(id), Some(params)) = (req.id.as_ref(), req.params.as_ref()) {
                let tool_name = params["name"].as_str().unwrap_or("");
                tools::dispatch_tools_call(id, tool_name, params, structural_dict, session_cache);
            }
        }
        _ => {
            if let Some(ref id) = req.id {
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                }));
            }
        }
    }
}
