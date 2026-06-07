// src/main.rs — Clean-CTX MCP Server (bootstrap only)
//
// The entire server lives in `clean_ctx::mcp`. This file exists only to
// satisfy the `[[bin]]` target.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    clean_ctx::mcp::run()
}