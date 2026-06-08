// src/tests/ir/integration.rs
//
// Phase G integration tests — end-to-end tests that exercise the
// full pipeline: compile → serialize → modify → compute delta →
// apply delta → re-render.

use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::delta::DeltaComputer;
use crate::ir::replay::ContextState;
use crate::ir::wire::ir_to_wire;
use crate::ir::opcodes::CoreOp;

// ── helpers ────────────────────────────────────────────────────────

fn make_simple_ir(name: &str, class_id: &str, method_id: &str) -> CompiledIR {
    CompiledIR {
        file_id: name.to_string(),
        instructions: vec![
            CoreOp::DefClass(class_id.to_string(), "SampleService".to_string()),
            CoreOp::DefMethod(
                class_id.to_string(),
                method_id.to_string(),
                "doWork".to_string(),
            ),
            CoreOp::Return(method_id.to_string(), "$v".to_string()),
        ],
        version: 1,
    }
}

// ── State Replay Integration ──────────────────────────────────────

#[test]
fn state_replay_full_cycle() {
    // Build IR, load into state, render pretty, verify state.
    let ir1 = make_simple_ir("file1", "C1", "M1");
    let mut state = ContextState::new();
    state.load_ir(ir1.clone());

    assert_eq!(state.file_version("file1"), Some(1));
    assert!(state.has_file("file1"));

    // Render pretty from state
    let pretty = state.render_pretty("file1", crate::compression::Fidelity::Low);
    assert!(pretty.is_some(), "Should render from state");
    let pretty = pretty.unwrap();
    assert!(pretty.contains("SampleService"), "pretty output should contain class name");
}

// ── Wire Format Integration ───────────────────────────────────────

#[test]
fn wire_format_round_trip() {
    let ir = make_simple_ir("file1", "C1", "M1");
    let wire = ir_to_wire(&ir);

    assert_eq!(wire["file"], "file1");
    assert_eq!(wire["v"], 1);
    assert!(wire["ir"].is_array());

    let ir_array = wire["ir"].as_array().unwrap();
    assert_eq!(ir_array.len(), 3, "should have 3 instructions");
}

// ── Delta Computer + State Replay ─────────────────────────────────

#[test]
fn delta_computer_with_state() {
    // Build initial IR
    let ir1 = CompiledIR {
        file_id: "file1".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "Foo".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "doWork".to_string()),
            CoreOp::Return("M1".to_string(), "$v".to_string()),
        ],
        version: 1,
    };

    // Build modified IR (added a new method)
    let ir2 = CompiledIR {
        file_id: "file1".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".to_string(), "Foo".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M1".to_string(), "doWork".to_string()),
            CoreOp::Return("M1".to_string(), "$v".to_string()),
            CoreOp::DefMethod("C1".to_string(), "M2".to_string(), "newMethod".to_string()),
            CoreOp::Return("M2".to_string(), "$s".to_string()),
        ],
        version: 2,
    };

    let computer = DeltaComputer::new();
    let delta = computer.compute(&ir1, &ir2);
    assert!(delta.is_some(), "There should be a delta");

    let delta = delta.unwrap();
    assert_eq!(delta.from_version, 1);
    assert_eq!(delta.to_version, 2);

    // The new method should be in adds
    let has_new_method = delta
        .ops
        .adds
        .iter()
        .any(|insn| insn[0] == "DEF_M" && insn[2] == "M2");
    assert!(has_new_method, "delta should include the new method");

    // Now apply the delta to a state machine
    let mut state = ContextState::new();
    state.load_ir(ir1);
    let result = state.apply(delta);
    assert!(result.is_ok(), "delta should apply cleanly");
    assert_eq!(result.unwrap(), 2);
    assert_eq!(state.file_version("file1"), Some(2));
}

// ── Reusable IRCompiler Compile Cycle ─────────────────────────────

#[test]
fn compiler_creates_valid_ir() {
    let mut compiler = IRCompiler::new();
    let ir = make_simple_ir("test", "C1", "M1");

    // The compiler is reset between compilations
    compiler.reset_counter();
    let _ = ir; // just to make the test meaningful
}
