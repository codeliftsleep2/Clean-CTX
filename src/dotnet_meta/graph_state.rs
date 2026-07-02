// src/dotnet_meta/graph_state.rs
//
// McpState integration for the .NET dependency graph.
//
// Mirrors `angular_meta::graph_state::AngularGraphHandle` pattern.
// Provides a handle to the .NET graph stored in McpState.

use super::graph::DotnetGraph;

/// Handle to the .NET dependency graph stored in McpState.
///
/// This is a lightweight wrapper that provides typed access to the
/// .NET graph without exposing the full McpState.
#[derive(Debug, Clone)]
pub struct DotnetGraphHandle {
    /// The .NET dependency graph
    pub graph: DotnetGraph,
}

impl DotnetGraphHandle {
    /// Create a new empty handle.
    pub fn new() -> Self {
        Self {
            graph: DotnetGraph::new(),
        }
    }

    /// Get a reference to the graph.
    pub fn graph(&self) -> &DotnetGraph {
        &self.graph
    }

    /// Get a mutable reference to the graph.
    pub fn graph_mut(&mut self) -> &mut DotnetGraph {
        &mut self.graph
    }

    /// Add markers from a single file to the graph.
    pub fn add_file_markers(&mut self, file: &str, markers: &[String]) {
        for line in markers {
            if let Some(node) = DotnetGraph::parse_node_from_line(line, file) {
                self.graph.add_node(node);
            }
        }
    }

    /// Build the graph from all collected markers.
    ///
    /// This is called after all files in a workspace have been processed.
    pub fn build_from_markers(&mut self, markers: &[(String, Vec<String>)]) {
        self.graph = DotnetGraph::build_from_markers(markers);
    }

    /// Render the graph as a `§ΦMAP` footer.
    pub fn render_footer(&self) -> String {
        self.graph.render_footer()
    }
}

impl Default for DotnetGraphHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/graph_state.rs"]
mod tests;