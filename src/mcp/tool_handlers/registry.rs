// src/mcp/tool_handlers/registry.rs
//
// Handler registry - maps tool names to boxed handler functions.

use std::collections::HashMap;
use crate::mcp::tool_handlers::traits::BoxedHandlerFn;

/// Handler metadata
pub struct HandlerEntry {
    pub handler: BoxedHandlerFn,
}

/// Handler registry - maps tool names to handler functions
pub struct HandlerRegistry {
    handlers: HashMap<&'static str, HandlerEntry>,
}

impl HandlerRegistry {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }
    
    pub fn register(&mut self, name: &'static str, handler: BoxedHandlerFn) {
        self.handlers.insert(name, HandlerEntry { handler });
    }
    
    pub fn get(&self, tool_name: &str) -> Option<&HandlerEntry> {
        self.handlers.get(tool_name)
    }
    
    /// Returns all registered tool names (for tool_list generation).
    /// Currently unused but kept for v0.3.0 registry-based dispatch.
    #[allow(dead_code)]
    pub fn tool_names(&self) -> Vec<&'static str> {
        self.handlers.keys().copied().collect()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self { Self::new() }
}

pub fn create_default_registry() -> HandlerRegistry {
    let mut reg = HandlerRegistry::new();

    // Core compression/IR/delta handlers (src/mcp/tool_handlers/core.rs)
    reg.register(
        "compress_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_compress_code_context(id, params, state);
        }),
    );






    reg.register(
        "provide_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_provide_code_context(id, params, state);
        }),
    );






    reg.register(
        "diff_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_diff_code_context(id, params, state);
        }),
    );






    reg.register(
        "delta_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_delta_code_context(id, params, state);
        }),
    );






    reg.register(
        "delta_text_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_delta_text_context(id, params, state);
        }),
    );






    reg.register(
        "apply_delta",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_apply_delta(id, params, state);
        }),
    );






    reg.register(
        "restore_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_restore_context(id, params, state);
        }),
    );







    // Context history handler (src/mcp/tool_handlers/context/mod.rs)
    reg.register(
        "context_history",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::context::handle_context_history(id, params, state);
        }),
    );







    // Dashboard/stats handler (src/mcp/tool_handlers/stats/mod.rs)
    reg.register(
        "context_stats",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::stats::handle_context_stats(id, params, state);
        }),
    );







    // Persistence handlers (src/mcp/tool_handlers/persistence/mod.rs)
    reg.register(
        "save_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_save_context(id, params, state);
        }),
    );






    reg.register(
        "list_sessions",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_list_sessions(id, params, state);
        }),
    );






    reg.register(
        "replay_history",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_replay_history(id, params, state);
        }),
    );






    reg.register(
        "purge_old_deltas",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_purge_old_deltas(id, params, state);
        }),
    );







    reg
}
