// src/cbm/mod.rs
//
// Codebase-Memory-MCP (CBM) integration module.
//
// This entire module is self-contained — it owns its own config, client,
// graph bridge, MCP tool definitions, and tool handlers. The rest of
// Clean-CTX knows about CBM through exactly three hooks:
//
//   1. `crate::config::CleanCtxConfig::cbm` — the CBM config field
//   2. `crate::mcp::state::McpState::graph_bridge` — Option<GraphBridge>
//   3. Tool dispatch in `crate::mcp::tools` calls into `crate::cbm::handlers`
//
// To install CBM: `codebase-memory-mcp` binary on PATH.
// To disable:     set `cbm.enabled = false` in `.clean-ctx.json`.

pub mod bridge;
pub mod cache_store;
pub mod client;
pub mod config;
pub mod handlers;
pub mod json_compress;
pub mod proxy;
pub mod setup;
pub mod tools;

// Re-export the public API for external consumers.
pub use bridge::{
    ArchitectureOverview, ChangeSet, DeadCodeEntry, GraphBridge, GraphEdge, GraphNode, QueryResult,
    SymbolImportance,
};
pub use config::{CbmConfig, CbmStatus};
pub use handlers::{
    handle_get_architecture, handle_get_cbm_status, handle_graph_query, handle_graph_search,
    handle_graph_trace,
};
pub use setup::{CbmSetupInfo, cbm_setup_check};
pub use tools::cbm_tool_list;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
