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

mod handlers;
pub(crate) mod prompts;
mod router;
mod server;
pub(crate) mod tools;
pub(crate) mod workspace;

/// Run the MCP server. This is the entry point called from `main()`.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    server::run()
}