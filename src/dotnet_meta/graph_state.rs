// src/dotnet_meta/graph_state.rs
//
// McpState integration for the .NET dependency graph.
//
// Mirrors `angular_meta::graph_state::AngularGraphHandle` pattern.
// Provides a handle to the .NET graph stored in McpState.
//
// The `DotnetGraph` is built once per `compress_workspace` call
// and lives in `McpState.dotnet_graph`. It is wrapped in
// `Option<Arc<Mutex<DotnetGraph>>>` so that:
//
// 1. It can be shared across the single-threaded MCP dispatch chain
// 2. It is `None` when no workspace has been compressed yet
// 3. It is replaced (not mutated) on each new workspace compression

use crate::dotnet_meta::graph::DotnetGraph;
use std::sync::{Arc, Mutex};

/// Thread-safe wrapper for the .NET graph.
///
/// The graph is built once per workspace compression and then shared
/// immutably via `Arc<Mutex<…>>`. The mutex lock is never contended
/// in practice (single-threaded MCP server), but the wrapper exists
/// to satisfy the type system for future multi-threaded scenarios.
#[derive(Debug, Clone)]
pub struct DotnetGraphHandle {
    inner: Arc<Mutex<Option<DotnetGraph>>>,
}

impl Default for DotnetGraphHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl DotnetGraphHandle {
    /// Create a new empty handle (no graph built yet).
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Store a new graph, replacing any previous one.
    pub fn set(&self, graph: DotnetGraph) {
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
        F: FnOnce(&DotnetGraph) -> R,
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
        F: FnOnce(&mut DotnetGraph) -> R,
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

#[cfg(test)]
#[path = "../tests/dotnet_meta/graph_state.rs"]
mod tests;