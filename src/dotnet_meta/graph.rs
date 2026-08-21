// src/dotnet_meta/graph.rs
//
// Cross-file dependency graph for .NET — DI, endpoints, hubs.
//
// Builds a lightweight graph of:
// - Service → Controller dependencies (DI)
// - Controller → Action endpoints
// - Hub → Client interface relationships
// - DbContext → Entity relationships
//
// This graph is used for:
// - Workspace-level `§ΦMAP` footer
// - CBM cross-layer integration (Phase 3)
//
// # Typestate (Track B, aligns with F-ANG-05)
//
// `DotnetGraph` is the **resolved** form — once you have one, the
// graph is guaranteed to be consistent (all nodes indexed, all edges
// verified). There is no public way to mutate it.
//
// To *build* a graph, use [`DotnetGraphBuilder`]. The builder owns
// the mutable maps and exposes `register_class` / `add_edge`.
// Calling [`DotnetGraphBuilder::build`] consumes the builder by value,
// runs the consistency pass, and returns the immutable `DotnetGraph`.
// After that point the mutation methods are simply not available —
// mutating after resolution is a type error.

use super::markers::PhiLineKind;
use std::collections::HashMap;

use crate::compression::graph_utils::{find_cycles, has_cycle, transitive_dependencies};

/// A node in the .NET dependency graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphNode {
    /// Node identifier (e.g., class name)
    pub id: String,
    /// Node kind (controller, hub, service, etc.)
    pub kind: PhiLineKind,
    /// File path where this node is defined
    pub file: String,
}

/// An edge in the .NET dependency graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Edge kind (depends_on, calls, etc.)
    pub kind: String,
}

/// Mutable builder for a `DotnetGraph`.
///
/// This is the only type that exposes `register_node` and `add_edge`.
/// The [`DotnetGraphBuilder::build`] method consumes `self` and returns
/// a fully-resolved [`DotnetGraph`], making it a type error to
/// register a class or add an edge after resolution.
#[derive(Debug, Clone, Default)]
pub struct DotnetGraphBuilder {
    /// Nodes indexed by ID
    nodes: HashMap<String, GraphNode>,
    /// Edges
    edges: Vec<GraphEdge>,
    /// Reverse index: file → node IDs
    file_index: HashMap<String, Vec<String>>,
}

impl DotnetGraphBuilder {
    /// Create a new empty graph builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a node in the graph. If the node ID already exists,
    /// the last registration wins (consistent with Angular/Spring behavior).
    pub fn register_node(&mut self, node: GraphNode) {
        let id = node.id.clone();
        let file = node.file.clone();
        self.nodes.insert(id.clone(), node);
        self.file_index.entry(file).or_default().push(id);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    /// Consume the builder, run the consistency pass, and return
    /// the immutable [`DotnetGraph`].
    pub fn build(self) -> DotnetGraph {
        DotnetGraph {
            nodes: self.nodes,
            edges: self.edges,
            file_index: self.file_index,
            resolved: true,
        }
    }
}

/// The .NET dependency graph (resolved form).
///
/// # How to obtain one
///
/// `DotnetGraph` can only be constructed via [`DotnetGraphBuilder::build`].
/// Direct construction is not public — callers go through the builder.
///
/// # Querying
///
/// Once you have a `DotnetGraph`, use:
/// - [`nodes`](Self::nodes) — all registered nodes
/// - [`all_nodes`](Self::all_nodes) — same as `nodes()` (alias for consistency)
/// - [`edges`](Self::edges) — all edges
/// - [`nodes_for_file`](Self::nodes_for_file) — nodes in a specific file
/// - [`edges_for_node`](Self::edges_for_node) — edges for a specific node
/// - [`has_cycle`](Self::has_cycle) — cycle detection via DFS
/// - [`find_cycles`](Self::find_cycles) — list all cycles
/// - [`transitive_dependencies`](Self::transitive_dependencies) — depth-N dep resolution
/// - [`render_footer`](Self::render_footer) — `§ΦMAP` footer
///
/// # Thread safety
///
/// `DotnetGraph` is wrapped in `Option<Arc<Mutex<DotnetGraph>>>` in
/// `DotnetGraphHandle` so it can be shared across the single-threaded
/// MCP dispatch chain.
#[derive(Debug, Clone)]
pub struct DotnetGraph {
    /// Nodes indexed by ID
    nodes: HashMap<String, GraphNode>,
    /// Edges
    edges: Vec<GraphEdge>,
    /// Reverse index: file → node IDs
    file_index: HashMap<String, Vec<String>>,
    /// Whether the graph has been resolved. Always `true` for graphs
    /// produced by [`DotnetGraphBuilder::build`].
    resolved: bool,
}

impl DotnetGraph {
    /// Check whether the graph has been resolved.
    ///
    /// For graphs produced by [`DotnetGraphBuilder::build`] (the
    /// public construction path) this is always `true`.
    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    /// Get all nodes.
    pub fn nodes(&self) -> Vec<&GraphNode> {
        self.nodes.values().collect()
    }

    /// Alias for `nodes()` — consistent with Angular/Spring `all_classes()`.
    pub fn all_nodes(&self) -> Vec<&GraphNode> {
        self.nodes.values().collect()
    }

    /// Get all edges.
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// Get the number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get nodes for a specific file.
    pub fn nodes_for_file(&self, file: &str) -> Vec<&GraphNode> {
        self.file_index
            .get(file)
            .into_iter()
            .flatten()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Get edges for a specific node.
    pub fn edges_for_node(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|e| e.from == node_id || e.to == node_id)
            .collect()
    }

    /// Check if the graph contains a cycle using DFS.
    ///
    /// Returns `true` if at least one cycle is detected.
    /// Uses three-color DFS (white/gray/black) for O(V+E) performance.
    pub fn has_cycle(&self) -> bool {
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        let id_to_idx: HashMap<&str, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();

        let node_count = node_ids.len();
        if node_count == 0 {
            return false;
        }

        // Pre-compute adjacency list
        let mut adj_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for edge in &self.edges {
            if let Some(&from_idx) = id_to_idx.get(edge.from.as_str()) {
                if let Some(&to_idx) = id_to_idx.get(edge.to.as_str()) {
                    adj_map.entry(from_idx).or_default().push(to_idx);
                }
            }
        }

        let adj_fn = |i: usize| adj_map.get(&i).cloned().unwrap_or_default();

        has_cycle(node_count, adj_fn)
    }

    /// Find all cycles in the graph using DFS.
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        let id_to_idx: HashMap<&str, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();

        let node_count = node_ids.len();
        if node_count == 0 {
            return Vec::new();
        }

        // Pre-compute adjacency list
        let mut adj_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for edge in &self.edges {
            if let Some(&from_idx) = id_to_idx.get(edge.from.as_str()) {
                if let Some(&to_idx) = id_to_idx.get(edge.to.as_str()) {
                    adj_map.entry(from_idx).or_default().push(to_idx);
                }
            }
        }

        let adj_fn = |i: usize| adj_map.get(&i).cloned().unwrap_or_default();

        let label_fn = |i: usize| {
            node_ids
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("unknown_{}", i))
        };

        find_cycles(node_count, adj_fn, label_fn)
    }

    /// Compute transitive dependencies for a node up to a given depth.
    ///
    /// Returns all reachable nodes by following edges outward from `node_id`.
    /// - `depth=1` → direct dependencies only
    /// - `depth=2` → dependencies of dependencies
    /// - `depth=0` or negative → all transitive dependencies (BFS to completion)
    pub fn transitive_dependencies(&self, node_id: &str, depth: i32) -> Vec<String> {
        if !self.nodes.contains_key(node_id) {
            return Vec::new();
        }

        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        let id_to_idx: HashMap<&str, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();

        let start_idx = id_to_idx[node_id];
        let node_count = node_ids.len();

        // Pre-compute adjacency list
        let mut adj_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for edge in &self.edges {
            if let Some(&from_idx) = id_to_idx.get(edge.from.as_str()) {
                if let Some(&to_idx) = id_to_idx.get(edge.to.as_str()) {
                    adj_map.entry(from_idx).or_default().push(to_idx);
                }
            }
        }

        let adj_fn = |i: usize| adj_map.get(&i).cloned().unwrap_or_default();

        let indices = transitive_dependencies(start_idx, depth, node_count, adj_fn);
        indices
            .into_iter()
            .filter_map(|i| node_ids.get(i))
            .cloned()
            .collect()
    }

    /// Build a graph from extracted markers across multiple files.
    ///
    /// This is called after all files in a workspace have been processed.
    /// Uses the builder internally, then returns the resolved graph.
    pub fn build_from_markers(markers: &[(String, Vec<String>)]) -> Self {
        let mut builder = DotnetGraphBuilder::new();

        for (file, lines) in markers {
            for line in lines {
                if let Some(node) = Self::parse_node_from_line(line, file) {
                    builder.register_node(node);
                }
            }
        }

        for (file, lines) in markers {
            for line in lines {
                Self::add_edges_from_line_inner(&mut builder, line, file, markers);
            }
        }

        builder.build()
    }

    /// Parse a graph node from a marker line (public for graph_state usage).
    pub fn parse_node_from_line(line: &str, file: &str) -> Option<GraphNode> {
        if let Some((kind_str, rest)) = line.split_once(':') {
            let kind = PhiLineKind::from_token(kind_str)?;
            let id = rest.split_whitespace().next().unwrap_or(rest).to_string();

            Some(GraphNode {
                id,
                kind,
                file: file.to_string(),
            })
        } else {
            None
        }
    }

    /// Add edges based on relationships in a marker line.
    fn add_edges_from_line_inner(
        builder: &mut DotnetGraphBuilder,
        line: &str,
        _file: &str,
        _all_markers: &[(String, Vec<String>)],
    ) {
        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φdi" {
                if let Some((from, to)) = rest
                    .split("→")
                    .map(|s| s.trim())
                    .collect::<Vec<_>>()
                    .split_first()
                {
                    if let Some(target) = to.first() {
                        builder.add_edge(GraphEdge {
                            from: from.to_string(),
                            to: target.to_string(),
                            kind: "di".to_string(),
                        });
                    }
                }
            }
        }

        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φaction" {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let class_name = parts[0].trim();
                    let action_sig = parts[1].trim();
                    builder.add_edge(GraphEdge {
                        from: class_name.to_string(),
                        to: action_sig.to_string(),
                        kind: "action".to_string(),
                    });
                }
            }
        }

        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φhub" {
                if let Some(bracket_start) = rest.find('[') {
                    if let Some(bracket_end) = rest[bracket_start..].find(']') {
                        let class_name = rest[..bracket_start].trim();
                        let client_iface =
                            rest[bracket_start + 1..bracket_start + bracket_end].trim();
                        builder.add_edge(GraphEdge {
                            from: class_name.to_string(),
                            to: client_iface.to_string(),
                            kind: "hub_client".to_string(),
                        });
                    }
                }
            }
        }

        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φrel" {
                if let Some((from, to)) = rest
                    .split("→")
                    .map(|s| s.trim())
                    .collect::<Vec<_>>()
                    .split_first()
                {
                    if let Some(target) = to.first() {
                        builder.add_edge(GraphEdge {
                            from: from.to_string(),
                            to: target.to_string(),
                            kind: "relationship".to_string(),
                        });
                    }
                }
            }
        }
    }

    /// Render the graph as a `§ΦMAP` footer.
    pub fn render_footer(&self) -> String {
        if self.nodes.is_empty() {
            return String::new();
        }

        let mut footer = String::new();
        footer.push_str("§ΦMAP\n");

        let mut by_kind: HashMap<PhiLineKind, Vec<&GraphNode>> = HashMap::new();
        for node in self.nodes.values() {
            by_kind.entry(node.kind).or_default().push(node);
        }

        for (kind, nodes) in &by_kind {
            let prefix = kind.marker_prefix();
            for node in nodes {
                footer.push_str(&format!("{} {}@<{}>\n", prefix, node.id, node.file));
            }
        }

        if !self.edges.is_empty() {
            footer.push_str("§EDGES\n");
            for edge in &self.edges {
                footer.push_str(&format!("{} → {}\n", edge.from, edge.to));
            }
        }

        footer
    }
}

/// Helper type for building a batch of graph entries during the
/// per-file compression pass. Mirrors `angular_meta::graph::GraphCollector`.
#[derive(Debug, Default)]
pub struct GraphCollector {
    /// List of (file_path, marker_lines)
    pub entries: Vec<(String, Vec<String>)>,
}

impl GraphCollector {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Push marker lines for a single file.
    pub fn push(&mut self, file: String, markers: Vec<String>) {
        self.entries.push((file, markers));
    }

    /// Flush all collected entries into a `DotnetGraph`.
    pub fn build_graph(&self) -> DotnetGraph {
        DotnetGraph::build_from_markers(&self.entries)
    }

    /// Check if any entries have been collected.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of collected entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/graph.rs"]
mod tests;
