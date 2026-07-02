// src/mcp/mod.rs
//
// MCP server module. Re-exports the top-level `run()` entry point.
//
// Sub-modules:
//   - server     : stdin/stdout read loop
//   - router     : JSON-RPC method dispatch
//   - handlers   : method handler implementations (initialize, tools/list, etc.)
//   - tools      : tool definitions and tools/call dispatch
//   - prompts    : prompt definitions and content
//   - workspace  : workspace-level operations (compress_workspace_dir, collect_source_files)
//   - state      : per-session state shared by all tool handlers (F-05)

pub mod dispatcher;
mod handlers;
pub(crate) mod tool_handlers;
pub(crate) mod tool_helpers;
pub(crate) mod cache_hints;
pub(crate) mod buffered_store;
pub(crate) mod context_store;
pub(crate) mod heuristics;
pub(crate) mod prompts;
pub(crate) mod proxy_stats;
mod router;
mod server;
pub(crate) mod session_stats;
pub(crate) mod sqlite_store;
pub(crate) mod state;
pub(crate) mod tools;
pub(crate) mod workspace;
pub(crate) mod workspace_util;

pub use state::McpState;

/// Run the MCP server. This is the entry point called from `main()`.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    server::run()
}

// Regression and audit-fix tests compress src/main.rs and other .rs files,
// so they require the "rust" feature to be enabled.
#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/regression.rs"]
mod regression;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/audit_fixes.rs"]
mod audit_fixes;
