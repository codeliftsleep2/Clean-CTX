//! Fresh compilation state and snapshot ownership at the production boundary.
#![cfg(feature = "typescript")]
use super::*;
use crate::compression::{
    Fidelity,
    capture_pipeline::{parse_count, reset_parse_count},
};
use crate::ir::{IRCompiler, opcodes::CoreOp};

fn compiler() -> IRCompiler {
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(
        crate::ir::layers::typescript::TypeScriptLayer::new(),
    ));
    compiler
}

fn compile(compiler: &mut IRCompiler, source: &str) -> crate::ir::CompiledIR {
    compiler
        .compile(
            source,
            "Example.ts",
            crate::compression::language::safe_typescript_language().unwrap(),
            crate::queries::TS_QUERY,
            Fidelity::High,
            None,
        )
        .unwrap()
}

fn has_pipe(ir: &crate::ir::CompiledIR) -> bool {
    ir.instructions
        .iter()
        .any(|op| matches!(op, CoreOp::TypeAlias(name, _) if name == "@pipeRx"))
}

#[test]
fn invocation_fresh_state_edit_and_empty_replacement_clear_meta_and_occurrences() {
    let original = "import { map, tap } from 'rxjs/operators'; class C { value$ = source.pipe(map(x => x), tap(log)); }";
    let edited = original.replace(".pipe(", ".skip(");
    let mut compiler = compiler();
    reset_parse_count();
    assert!(has_pipe(&compile(&mut compiler, original)));
    let snapshot = compiler.invocation_evidence.take().unwrap();
    assert!(!has_pipe(&compile(&mut compiler, &edited)));
    assert_eq!(
        snapshot.view("Example.ts", &edited).err(),
        Some(EvidenceMismatch::SourceHashMismatch)
    );
    let current = compiler.invocation_evidence.as_ref().unwrap();
    let view = current.view("Example.ts", &edited).unwrap();
    assert!(view.occurrences().iter().any(|call| matches!(call.callee(), WrittenCallee::Member { terminal, .. } if view.slice(*terminal).unwrap() == "skip")));
    assert!(!has_pipe(&compile(&mut compiler, "")));
    assert!(
        compiler
            .invocation_evidence
            .as_ref()
            .unwrap()
            .view("Example.ts", "")
            .unwrap()
            .occurrences()
            .is_empty()
    );
    assert_eq!(
        snapshot.view("Example.ts", "").err(),
        Some(EvidenceMismatch::SourceHashMismatch)
    );
    assert_eq!(
        parse_count(),
        2,
        "nonempty compilations parse once; empty source keeps the zero-parse fast path"
    );
}

#[test]
fn invocation_fresh_state_failure_clears_a_populated_snapshot() {
    let source = "class C { run() { service.save(item); } }";
    let mut compiler = compiler();
    compile(&mut compiler, source);
    assert_eq!(
        compiler
            .invocation_evidence
            .as_ref()
            .unwrap()
            .view("Example.ts", source)
            .unwrap()
            .occurrences()
            .len(),
        1
    );
    let failure = compiler.compile(
        source,
        "Example.ts",
        crate::compression::language::safe_typescript_language().unwrap(),
        "(",
        Fidelity::High,
        None,
    );
    assert!(failure.is_err());
    assert!(compiler.invocation_evidence.is_none());
    compile(&mut compiler, "class C { run() {} }");
    assert!(
        compiler
            .invocation_evidence
            .as_ref()
            .unwrap()
            .view("Example.ts", "class C { run() {} }")
            .unwrap()
            .occurrences()
            .is_empty()
    );
}
