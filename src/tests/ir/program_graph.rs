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

    assert_eq!(
        graph.nodes.len(),
        4,
        "should have 4 nodes (class, method, field, interface)"
    );
    assert_eq!(
        graph.edges.len(),
        4,
        "should have 4 edges (extends, implements, injects, dataflow)"
    );
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

// ── RED-CALL20: native call facts preserve arity ─────────────────────

fn call_ir() -> CompiledIR {
    CompiledIR {
        file_id: "Example.cs".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Example".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "Process".into()),
            CoreOp::Call("M1".into(), "OrderBy".into(), 1, false),
            CoreOp::Call("M1".into(), "OrderBy".into(), 2, false),
        ],
        version: 1,
    }
}

fn calls_of(graph: &crate::ir::program_graph::ProgramGraph) -> Vec<(String, String, usize)> {
    graph
        .edges_of_type("calls")
        .iter()
        .map(|edge| match edge {
            crate::ir::program_graph::GraphEdge::Calls {
                from,
                to,
                explicit_arg_count,
            } => (from.clone(), to.clone(), *explicit_arg_count),
            other => panic!("unexpected edge type: {other:?}"),
        })
        .collect()
}

#[test]
fn test_graph_call_edges_preserve_arity_build() {
    let graph = GraphBuilder::build(&[call_ir()], &GlobalSymbolTable::new());
    assert_eq!(
        calls_of(&graph),
        vec![
            ("M1".to_string(), "OrderBy".to_string(), 1),
            ("M1".to_string(), "OrderBy".to_string(), 2),
        ],
        "the local graph must carry the explicit argument count verbatim"
    );
}

#[test]
fn test_graph_call_edges_preserve_arity_build_from_instructions() {
    let graph = GraphBuilder::build_from_instructions(&call_ir().instructions);
    assert_eq!(
        calls_of(&graph),
        vec![
            ("M1".to_string(), "OrderBy".to_string(), 1),
            ("M1".to_string(), "OrderBy".to_string(), 2),
        ],
        "both builders must map CoreOp::Call identically"
    );
    assert_eq!(
        graph.fan_in("OrderBy"),
        2,
        "two arity-distinct facts are two call occurrences"
    );
    assert_eq!(graph.fan_out("M1"), 2);
}
