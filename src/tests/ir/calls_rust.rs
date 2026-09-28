//! Native call facts for Rust through the production compilation path.

#![cfg(feature = "rust")]

use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::layers::rust::RustLayer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::queries::RS_QUERY;

#[test]
fn rust_native_calls_preserve_caller_callee_and_written_arity() {
    let source = r#"
        struct Worker;

        impl Worker {
            fn run(&self) {
                self.save(1);
                helper(2, 3);
            }
        }
    "#;
    let language =
        crate::compression::language::safe_rust_language().expect("rust grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(RustLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    let ir = compiler
        .compile(
            source,
            "worker.rs",
            language,
            RS_QUERY,
            Fidelity::Medium,
            None,
        )
        .expect("production Rust compilation must succeed");

    let run_id = ir
        .instructions
        .iter()
        .find_map(|op| match op {
            CoreOp::DefMethod(_, method_id, name) if name == "run" => Some(method_id.as_str()),
            _ => None,
        })
        .expect("run method identity");
    let calls: Vec<_> = ir
        .instructions
        .iter()
        .filter_map(|op| op.call_parts())
        .collect();

    assert_eq!(
        calls,
        vec![(run_id, "save", 1, false), (run_id, "helper", 2, false)],
        "Rust calls must use the shared caller ownership and written-arity contract"
    );
}
