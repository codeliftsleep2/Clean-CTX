// src/tests/ir/diagnose_m6.rs
//
// Reproduction and trace of the [E003] FLAGS references unknown method 'M6'
// error on src/test_files/angular/user-card.component.ts.
//
// Run: cargo test --release --all-features -- ir::diagnose_m6 2>&1
// 
// This test exercises the exact production IR compilation path and dumps
// the full instruction stream for analysis.

use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::layers::csharp::CSharpLayer;
use crate::ir::layers::rust::RustLayer;
use crate::ir::layers::java::JavaLayer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::ir::opcodes::CoreOp;
use std::path::Path;

const FIXTURE_PATH: &str = "src/test_files/angular/user-card.component.ts";

#[test]
fn trace_e003_m6_instruction_stream() {
    // 1. Read the fixture
