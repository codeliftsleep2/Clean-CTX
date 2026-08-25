// src/mcp/tool_handlers/mod.rs
//
// Tool handler module - registry-based dispatch with modular handler
// implementations split across submodules for Single Responsibility.
//
// Submodules:
//   core/        — Compression, IR, delta, provide, restore handlers
//   context/     — Context history queries
//   persistence/ — SQLite/BufferedStore CRUD tools
//   stats/       — Dashboard rendering
//   traits.rs    — ToolHandlerDef trait (v0.3.0 placeholder)
//   registry.rs  — HandlerRegistry + wiring

pub mod context;
pub mod core;
pub mod persistence;
pub mod registry;
pub mod stats;
pub mod traits;

// Re-exports for API consumers (v0.3.0+)
#[allow(unused_imports)]
pub use registry::HandlerRegistry;
#[allow(unused_imports)]
pub use traits::BoxedHandlerFn;

#[cfg(all(test, feature = "rust"))]
#[path = "../../tests/mcp/tool_handlers.rs"]
pub(crate) mod tool_handlers_tests;
