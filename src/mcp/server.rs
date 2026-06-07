// src/mcp/server.rs
//
// MCP server main loop: reads JSON-RPC requests from stdin, dispatches
// them via the router, and writes responses to stdout.

use std::io::{self, BufRead};
use std::path::PathBuf;
use crate::dictionary::PathDictionary;
use crate::cache::LocalStateCache;
use crate::config::CleanCtxConfig;
use crate::protocol::JsonRpcRequest;

/// Run the MCP server loop, processing incoming JSON-RPC requests until
/// stdin is exhausted.
pub(crate) fn run() -> Result<(), Box<dyn std::error::Error>> {
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
            crate::mcp::router::dispatch(&req, &mut structural_dict, &mut session_cache);
        }
        buffer.clear();
    }
    Ok(())
}