// src/angular_meta/graph_state.rs
//
// McpState integration for the Angular cross-file dependency graph
// (Tier 3 of the Meta-Layer).
//
// The `AngularGraph` is built once per `compress_workspace` call
// and lives in `McpState.angular_graph`. It is wrapped in
// `Option<Arc<Mutex<AngularGraph>>>` so that:
//
// 1. It can be shared across the single-threaded MCP dispatch chain
// 2. It is `None` when no workspace has been compressed yet
// 3. It is replaced (not mutated) on each new workspace compression
//
// # Lifecycle
//
// 1. `McpState::new()` → `angular_graph: None`
// 2. `compress_workspace_dir`:
//    a. Creates a fresh `GraphCollector`
//    b. Passes it through the per-file compression pass (each file
//       that produces Angular decorators pushes its graph entry)
//    c. Calls `collector.build_graph()` to produce the `AngularGraph`
//    d. Stores the result in `McpState.angular_graph`
// 3. The graph is read-only for the rest of the session until the
//    next `compress_workspace` call replaces it.

use crate::angular_meta::graph::AngularGraph;
use std::sync::{Arc, Mutex};

/// Thread-safe wrapper for the Angular graph.
///
/// The graph is built once per workspace compression and then shared
/// immutably via `Arc<Mutex<…>>`. The mutex lock is never contended
/// in practice (single-threaded MCP server), but the wrapper exists
/// to satisfy the type system for future multi-threaded scenarios.
#[derive(Debug, Clone)]
pub struct AngularGraphHandle {
    inner: Arc<Mutex<Option<AngularGraph>>>,
}

impl Default for AngularGraphHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl AngularGraphHandle {
    /// Create a new empty handle (no graph built yet).
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Store a new graph, replacing any previous one.
    pub fn set(&self, graph: AngularGraph) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(graph);
        }
    }

    /// Clear the graph (set to `None`).
    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = None;
        }
    }

    /// Execute a read-only callback with the graph.
    ///
    /// Returns `None` if no graph has been built yet or the lock
    /// is poisoned.
    pub fn with_graph<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&AngularGraph) -> R,
    {
        if let Ok(guard) = self.inner.lock() {
            guard.as_ref().map(f)
        } else {
            None
        }
    }

    /// Execute a mutable callback with the graph.
    ///
    /// Returns `None` if no graph has been built yet or the lock
    /// is poisoned.
    pub fn with_graph_mut<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut AngularGraph) -> R,
    {
        if let Ok(mut guard) = self.inner.lock() {
            guard.as_mut().map(f)
        } else {
            None
        }
    }

    /// Check whether a graph has been built.
    pub fn is_present(&self) -> bool {
        if let Ok(guard) = self.inner.lock() {
            guard.is_some()
        } else {
            false
        }
    }
}
