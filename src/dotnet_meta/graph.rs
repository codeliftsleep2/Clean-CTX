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

use super::markers::PhiLineKind;
use std::collections::HashMap;

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

/// The .NET dependency graph.
#[derive(Debug, Clone, Default)]
pub struct DotnetGraph {
    /// Nodes indexed by ID
    nodes: HashMap<String, GraphNode>,
    /// Edges
    edges: Vec<GraphEdge>,
    /// Reverse index: file → node IDs
    file_index: HashMap<String, Vec<String>>,
}

impl DotnetGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: GraphNode) {
        let id = node.id.clone();
        let file = node.file.clone();
        self.nodes.insert(id.clone(), node);
        self.file_index.entry(file).or_default().push(id);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    /// Get all nodes.
    pub fn nodes(&self) -> Vec<&GraphNode> {
        self.nodes.values().collect()
    }

    /// Get all edges.
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
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

    /// Build a graph from extracted markers across multiple files.
    ///
    /// This is called after all files in a workspace have been processed.
    pub fn build_from_markers(
        markers: &[(String, Vec<String>)], // (file_path, marker_lines)
    ) -> Self {
        let mut graph = Self::new();

        // First pass: add all nodes
        for (file, lines) in markers {
            for line in lines {
                if let Some(node) = Self::parse_node_from_line(line, file) {
                    graph.add_node(node);
                }
            }
        }

        // Second pass: add edges based on relationships
        for (file, lines) in markers {
            for line in lines {
                graph.add_edges_from_line(line, file, markers);
            }
        }

        graph
    }

    /// Parse a graph node from a marker line.
    pub fn parse_node_from_line(line: &str, file: &str) -> Option<GraphNode> {
        // Match patterns like:
        //   Φctrl:UserController [api/users]
        //   Φhub:NotificationHub [INotificationClient]
        //   Φef:AppDbContext
        //   Φsvc:UserService
        if let Some((kind_str, rest)) = line.split_once(':') {
            // kind_str already includes the Φ prefix (e.g. "Φctrl")
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
    fn add_edges_from_line(&mut self, line: &str, _file: &str, _all_markers: &[(String, Vec<String>)]) {
        // Parse Φdi: lines to create service → implementation edges
        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φdi" {
                // Format: Φdi:ServiceName → RegistrationType
                if let Some((from, to)) = rest.split("→").map(|s| s.trim()).collect::<Vec<_>>().split_first() {
                    if let Some(target) = to.first() {
                        self.add_edge(GraphEdge {
                            from: from.to_string(),
                            to: target.to_string(),
                            kind: "di".to_string(),
                        });
                    }
                }
            }
        }

        // Parse Φaction: lines to create controller → action edges
        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φaction" {
                // Format: Φaction:Verb Name(params) → ReturnType
                // The class name is the first word before the verb
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    let class_name = parts[0].trim();
                    let action_sig = parts[1].trim();
                    self.add_edge(GraphEdge {
                        from: class_name.to_string(),
                        to: action_sig.to_string(),
                        kind: "action".to_string(),
                    });
                }
            }
        }

        // Parse Φhub: lines to create hub → client interface edges
        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φhub" {
                // Format: Φhub:ClassName [ClientInterface]
                if let Some(bracket_start) = rest.find('[') {
                    if let Some(bracket_end) = rest[bracket_start..].find(']') {
                        let class_name = rest[..bracket_start].trim();
                        let client_iface = rest[bracket_start + 1..bracket_start + bracket_end].trim();
                        self.add_edge(GraphEdge {
                            from: class_name.to_string(),
                            to: client_iface.to_string(),
                            kind: "hub_client".to_string(),
                        });
                    }
                }
            }
        }

        // Parse Φrel: lines to create entity → entity edges
        if let Some((kind_str, rest)) = line.split_once(':') {
            if kind_str == "Φrel" {
                // Format: Φrel:Name → Target
                if let Some((from, to)) = rest.split("→").map(|s| s.trim()).collect::<Vec<_>>().split_first() {
                    if let Some(target) = to.first() {
                        self.add_edge(GraphEdge {
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

        // Group nodes by kind
        let mut by_kind: HashMap<PhiLineKind, Vec<&GraphNode>> = HashMap::new();
        for node in self.nodes.values() {
            by_kind.entry(node.kind).or_default().push(node);
        }

        // Render each group
        for (kind, nodes) in &by_kind {
            let prefix = kind.marker_prefix();
            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
            footer.push_str(&format!("{} {}\n", prefix, node_ids.join(", ")));
        }

        // Render edges
        if !self.edges.is_empty() {
            footer.push_str("§EDGES\n");
            for edge in &self.edges {
                footer.push_str(&format!("{} → {}\n", edge.from, edge.to));
            }
        }

        footer
    }
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/graph.rs"]
mod tests;