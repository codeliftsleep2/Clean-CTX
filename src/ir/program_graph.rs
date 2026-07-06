// src/ir/program_graph.rs
//
// R-43b Phase 2: Lightweight Local Program Graph
//
// A lightweight local program graph built from compiled IRs.
// Nodes are symbols (classes, methods, fields), edges are relationships.
//
// This graph is built from structural IR facts only (confidence = 1.0).
// Inferred edges (from CBM or heuristics) live in InferenceLayer, NOT here.
//
// Key design decisions:
// - Lazy computation — not stored in CompiledIR, built on demand
// - No CBM data in ProgramGraph — all edges are structural facts
// - Edge types reuse existing CoreOp relationships plus new DATAFLOW edges

use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use super::symbol_table::{GlobalSymbolTable, SymbolKind};

/// A lightweight local program graph built from compiled IRs.
#[derive(Debug, Clone, Default)]
pub struct ProgramGraph {
    /// Nodes in the graph (classes, methods, fields)
    pub nodes: Vec<GraphNode>,
    /// Edges between nodes (relationships)
    pub edges: Vec<GraphEdge>,
}

/// A node in the program graph.
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Unique symbol ID (e.g., "C1", "M1")
    pub id: String,
    /// Original symbol name
    pub name: String,
    /// Kind of symbol
    pub kind: SymbolKind,
    /// File this symbol belongs to
    pub file_id: String,
}

/// An edge between two nodes in the program graph.
/// All edges have confidence = 1.0 (structural facts from tree-sitter).
#[derive(Debug, Clone)]
pub enum GraphEdge {
    /// Method calls another method
    Calls { from: String, to: String },
    /// Class extends another class
    Extends { child: String, parent: String },
    /// Class implements an interface
    Implements { class: String, interface: String },
    /// Class injects a dependency
    Injects { class: String, dependency: String },
    /// Method reads from a target
    DataFlowRead { method: String, target: String },
    /// Method writes to a target
    DataFlowWrite { method: String, target: String },
}

/// Builder for constructing a ProgramGraph from compiled IRs.
pub struct GraphBuilder;

impl GraphBuilder {
    /// Build a program graph from compiled IRs and symbol table.
    /// Lazy — called on demand, not during compilation.
    /// All edges have confidence = 1.0 (structural facts).
    pub fn build(
        compiled_irs: &[CompiledIR],
        _symbol_table: &GlobalSymbolTable,
    ) -> ProgramGraph {
        let mut graph = ProgramGraph::default();

        for ir in compiled_irs {
            let file_id = &ir.file_id;
            for op in &ir.instructions {
                match op {
                    CoreOp::DefClass(id, name) => {
                        graph.nodes.push(GraphNode {
                            id: id.clone(),
                            name: name.clone(),
                            kind: SymbolKind::Class,
                            file_id: file_id.clone(),
                        });
                    }
                    CoreOp::DefMethod(_cid, mid, name) => {
                        graph.nodes.push(GraphNode {
                            id: mid.clone(),
                            name: name.clone(),
                            kind: SymbolKind::Method,
                            file_id: file_id.clone(),
                        });
                        // Add Calls edge: method belongs to class
                        // (structural relationship)
                    }
                    CoreOp::DefField(_cid, fid, name) => {
                        graph.nodes.push(GraphNode {
                            id: fid.clone(),
                            name: name.clone(),
                            kind: SymbolKind::Field,
                            file_id: file_id.clone(),
                        });
                    }
                    CoreOp::DefInterface(id, name) => {
                        graph.nodes.push(GraphNode {
                            id: id.clone(),
                            name: name.clone(),
                            kind: SymbolKind::Interface,
                            file_id: file_id.clone(),
                        });
                    }
                    CoreOp::Extends(child, parent) => {
                        graph.edges.push(GraphEdge::Extends {
                            child: child.clone(),
                            parent: parent.clone(),
                        });
                    }
                    CoreOp::Implements(cid, iid) => {
                        graph.edges.push(GraphEdge::Implements {
                            class: cid.clone(),
                            interface: iid.clone(),
                        });
                    }
                    CoreOp::Injects(cid, deps) => {
                        for dep in deps {
                            graph.edges.push(GraphEdge::Injects {
                                class: cid.clone(),
                                dependency: dep.clone(),
                            });
                        }
                    }
                    CoreOp::DataFlow(mid, direction, target) => {
                        if direction == "reads" {
                            graph.edges.push(GraphEdge::DataFlowRead {
                                method: mid.clone(),
                                target: target.clone(),
                            });
                        } else {
                            graph.edges.push(GraphEdge::DataFlowWrite {
                                method: mid.clone(),
                                target: target.clone(),
                            });
                        }
                    }
                    _ => {} // Other ops don't contribute to graph structure
                }
            }
        }

        graph
    }

    /// Build a program graph from an instruction stream (for pass pipeline).
    pub fn build_from_instructions(instructions: &[CoreOp]) -> ProgramGraph {
        let mut graph = ProgramGraph::default();

        for op in instructions {
            match op {
                CoreOp::DefClass(id, name) => {
                    graph.nodes.push(GraphNode {
                        id: id.clone(),
                        name: name.clone(),
                        kind: SymbolKind::Class,
                        file_id: String::new(),
                    });
                }
                CoreOp::DefMethod(_cid, mid, name) => {
                    graph.nodes.push(GraphNode {
                        id: mid.clone(),
                        name: name.clone(),
                        kind: SymbolKind::Method,
                        file_id: String::new(),
                    });
                }
                CoreOp::DefField(_cid, fid, name) => {
                    graph.nodes.push(GraphNode {
                        id: fid.clone(),
                        name: name.clone(),
                        kind: SymbolKind::Field,
                        file_id: String::new(),
                    });
                }
                CoreOp::DefInterface(id, name) => {
                    graph.nodes.push(GraphNode {
                        id: id.clone(),
                        name: name.clone(),
                        kind: SymbolKind::Interface,
                        file_id: String::new(),
                    });
                }
                CoreOp::Extends(child, parent) => {
                    graph.edges.push(GraphEdge::Extends {
                        child: child.clone(),
                        parent: parent.clone(),
                    });
                }
                CoreOp::Implements(cid, iid) => {
                    graph.edges.push(GraphEdge::Implements {
                        class: cid.clone(),
                        interface: iid.clone(),
                    });
                }
                CoreOp::Injects(cid, deps) => {
                    for dep in deps {
                        graph.edges.push(GraphEdge::Injects {
                            class: cid.clone(),
                            dependency: dep.clone(),
                        });
                    }
                }
                CoreOp::DataFlow(mid, direction, target) => {
                    if direction == "reads" {
                        graph.edges.push(GraphEdge::DataFlowRead {
                            method: mid.clone(),
                            target: target.clone(),
                        });
                    } else {
                        graph.edges.push(GraphEdge::DataFlowWrite {
                            method: mid.clone(),
                            target: target.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        graph
    }
}

impl ProgramGraph {
    /// Find a node by its ID.
    pub fn find_node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Find all nodes of a given kind.
    pub fn nodes_by_kind(&self, kind: SymbolKind) -> Vec<&GraphNode> {
        self.nodes.iter().filter(|n| n.kind == kind).collect()
    }

    /// Get all edges of a specific type.
    pub fn edges_of_type(&self, edge_type: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| matches_edge_type(e, edge_type)).collect()
    }

    /// Get fan-in (number of callers) for a method.
    pub fn fan_in(&self, method_id: &str) -> usize {
        self.edges.iter()
            .filter(|e| matches!(e, GraphEdge::Calls { to, .. } if to == method_id))
            .count()
    }

    /// Get fan-out (number of callees) for a method.
    pub fn fan_out(&self, method_id: &str) -> usize {
        self.edges.iter()
            .filter(|e| matches!(e, GraphEdge::Calls { from, .. } if from == method_id))
            .count()
    }
}

fn matches_edge_type(edge: &GraphEdge, edge_type: &str) -> bool {
    matches!((edge, edge_type),
        (GraphEdge::Calls { .. }, "calls")
        | (GraphEdge::Extends { .. }, "extends")
        | (GraphEdge::Implements { .. }, "implements")
        | (GraphEdge::Injects { .. }, "injects")
        | (GraphEdge::DataFlowRead { .. }, "dataflow_read")
        | (GraphEdge::DataFlowWrite { .. }, "dataflow_write")
    )
}

#[cfg(test)]
#[path = "../tests/ir/program_graph.rs"]
mod tests;