// src/tests/ir/program_graph.rs
//
// Tests for R-43b Phase 2: Lightweight Local Program Graph

use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::CoreOp;
use crate::ir::program_graph::GraphBuilder;
use crate::ir::symbol_table::{GlobalSymbolTable, SymbolKind};

fn sample_ir() -> CompiledIR {
    CompiledIR {
        file_id: "test.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "UserService".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "getUser".into()),
            CoreOp::DefField("C1".into(), "F1".into(), "userRepo".into()),
            CoreOp::DefInterface("I1".into(), "IUserRepo".into()),
            CoreOp::Extends("C1".into(), "BaseService".into()),
            CoreOp::Implements("C1".into(), "IUserService".into()),
            CoreOp::Injects("C1".into(), vec!["IUserRepo".into()]),
            CoreOp::DataFlow("M1".into(), "reads".into(), "userRepo".into()),
        ],
        version: 1,
    }
}

#[test]
fn test_graph_build_from_ir() {
    let ir = sample_ir();
    let symbol_table = GlobalSymbolTable::new();
    let graph = GraphBuilder::build(&[ir], &symbol_table);

    assert_eq!(graph.nodes.len(), 4, "should have 4 nodes (class, method, field, interface)");
    assert_eq!(graph.edges.len(), 4, "should have 4 edges (extends, implements, injects, dataflow)");
}

#[test]
fn test_graph_build_from_instructions() {
    let ir = sample_ir();
    let graph = GraphBuilder::build_from_instructions(&ir.instructions);

    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.edges.len(), 4);
}

#[test]
fn test_find_node() {
    let ir = sample_ir();
    let graph = GraphBuilder::build_from_instructions(&ir.instructions);

    let node = graph.find_node("C1").expect("should find C1");
    assert_eq!(node.name, "UserService");
    assert_eq!(node.kind, SymbolKind::Class);
}

#[test]
fn test_nodes_by_kind() {
    let ir = sample_ir();
    let graph = GraphBuilder::build_from_instructions(&ir.instructions);

    let methods = graph.nodes_by_kind(SymbolKind::Method);
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].name, "getUser");
}

#[test]
fn test_edges_of_type() {
    let ir = sample_ir();
    let graph = GraphBuilder::build_from_instructions(&ir.instructions);

    let extends = graph.edges_of_type("extends");
    assert_eq!(extends.len(), 1);

    let dataflow_read = graph.edges_of_type("dataflow_read");
    assert_eq!(dataflow_read.len(), 1);
}

#[test]
fn test_empty_graph() {
    let graph = GraphBuilder::build_from_instructions(&[]);
    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_graph_node_fields() {
    let ir = sample_ir();
    let graph = GraphBuilder::build(&[ir], &GlobalSymbolTable::new());

    let class_node = graph.find_node("C1").unwrap();
    assert_eq!(class_node.file_id, "test.ts");
}