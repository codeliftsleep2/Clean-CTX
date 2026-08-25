// src/mcp/tool_handlers/registry.rs
//
// Handler registry - maps tool names to boxed handler functions.

use crate::mcp::tool_handlers::traits::BoxedHandlerFn;
use std::collections::HashMap;

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
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &'static str, handler: BoxedHandlerFn) {
        self.handlers.insert(name, HandlerEntry { handler });
    }

    pub fn get(&self, tool_name: &str) -> Option<&HandlerEntry> {
        self.handlers.get(tool_name)
    }

    /// Returns all registered tool names (for tool_list generation).
    /// Used by tests (`src/tests/mcp/tools.rs`) to verify registry/inline
    /// parity; kept for v0.3.0 registry-based dispatch. `#[allow(dead_code)]`
    /// is required because this is only consumed by external test modules.
    #[allow(dead_code)]
    pub fn tool_names(&self) -> Vec<&'static str> {
        self.handlers.keys().copied().collect()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! register_tool {
    ($registry:expr, $name:expr, $handler:expr) => {
        $registry.register($name, $handler);
    };
}

pub fn create_default_registry() -> HandlerRegistry {
    let mut reg = HandlerRegistry::new();

    // Core compression/IR/delta handlers (src/mcp/tool_handlers/core.rs)
    register_tool!(
        reg,
        "compress_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_compress_code_context(id, params, state);
        })
    );
    register_tool!(
        reg,
        "provide_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_provide_code_context(id, params, state);
        })
    );
    register_tool!(
        reg,
        "diff_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_diff_code_context(id, params, state);
        })
    );
    register_tool!(
        reg,
        "delta_code_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_delta_code_context(id, params, state);
        })
    );
    register_tool!(
        reg,
        "delta_text_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_delta_text_context(id, params, state);
        })
    );
    register_tool!(
        reg,
        "apply_delta",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_apply_delta(id, params, state);
        })
    );
    register_tool!(
        reg,
        "restore_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::core::handle_restore_context(id, params, state);
        })
    );

    // apply_edit write path (docs/plans/APPLY_EDIT_PLAN.md Phase 3)
    register_tool!(
        reg,
        "apply_edit",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::edit::handle_apply_edit(id, params, state);
        })
    );

    // Context history handler (src/mcp/tool_handlers/context/mod.rs)
    register_tool!(
        reg,
        "context_history",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::context::handle_context_history(id, params, state);
        })
    );

    // Dashboard/stats handler (src/mcp/tool_handlers/stats/mod.rs)
    register_tool!(
        reg,
        "context_stats",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::stats::handle_context_stats(id, params, state);
        })
    );

    // Persistence handlers (src/mcp/tool_handlers/persistence/mod.rs)
    register_tool!(
        reg,
        "save_context",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_save_context(id, params, state);
        })
    );
    register_tool!(
        reg,
        "list_sessions",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_list_sessions(id, params, state);
        })
    );
    register_tool!(
        reg,
        "replay_history",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_replay_history(id, params, state);
        })
    );
    register_tool!(
        reg,
        "purge_old_deltas",
        Box::new(|id, params, state| {
            crate::mcp::tool_handlers::persistence::handle_purge_old_deltas(id, params, state);
        })
    );

    reg
}
