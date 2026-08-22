use crate::compression::Fidelity;
use crate::compression::language::detect_language;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::opcodes::CoreOp;
use std::collections::HashSet;

/// Extract the set of method IDs that have a `CoreOp::Body` in the IR.
fn body_method_ids(ir: &CompiledIR) -> HashSet<&str> {
    ir.instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::Body(mid, _) = op {
                Some(mid.as_str())
            } else {
                None
            }
        })
        .collect()
}

/// Extract the `(method_name, method_id)` map from DefMethod ops so tests
/// can correlate body presence with method names.
fn method_names_by_id(ir: &CompiledIR) -> std::collections::HashMap<&str, &str> {
    ir.instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefMethod(_cid, mid, name) = op {
                Some((mid.as_str(), name.as_str()))
            } else {
                None
            }
        })
        .collect()
}

/// Compile the sample service at Edit fidelity.
fn compile_edit() -> CompiledIR {
    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    compiler
        .compile(source, "test_edit", language, query, Fidelity::Edit, None)
        .expect("compilation should succeed")
}

/// Compile the sample service at Edit fidelity with `focus`.
fn compile_edit_focused(focus: Option<&HashSet<String>>) -> CompiledIR {
    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "test_edit_focused",
            language,
            query,
            Fidelity::Edit,
            None,
            focus,
        )
        .expect("compilation should succeed")
}

fn compile_sample() -> CompiledIR {
    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);

    let mut compiler = IRCompiler::new();
    compiler
        .compile(source, "test_sample", language, query, Fidelity::Low, None)
        .expect("compilation should succeed")
}

#[test]
fn compile_sample_produces_instructions() {
    let ir = compile_sample();
    assert!(
        !ir.instructions.is_empty(),
        "compiled IR should have instructions"
    );
}

#[test]
fn compile_sample_version_is_one() {
    let ir = compile_sample();
    assert_eq!(ir.version, 1);
}

#[test]
fn compile_sample_file_id_matches() {
    let ir = compile_sample();
    assert_eq!(ir.file_id, "test_sample");
}

#[test]
fn compile_sample_has_def_class() {
    let ir = compile_sample();
    let classes: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefClass(..)))
        .collect();
    assert!(
        !classes.is_empty(),
        "should have at least one DefClass instruction"
    );
}

#[test]
fn compile_sample_has_def_methods() {
    let ir = compile_sample();
    let methods: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefMethod(..)))
        .collect();
    assert!(
        methods.len() >= 2,
        "sample_service.ts has 2 methods, got {}",
        methods.len()
    );
}

#[test]
fn compile_sample_has_flags() {
    let ir = compile_sample();
    let flags: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::Flags(..)))
        .collect();
    assert!(
        !flags.is_empty(),
        "sample_service.ts has if/for/return/throw -> should produce FLAGS"
    );
}

#[test]
fn compile_sample_class_name_correct() {
    let ir = compile_sample();
    let class = ir
        .instructions
        .iter()
        .find_map(|op| {
            if let CoreOp::DefClass(_, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .expect("should have a class");
    assert_eq!(class, "SampleService");
}

#[test]
fn compile_sample_method_names_correct() {
    let ir = compile_sample();
    let method_names: Vec<&str> = ir
        .instructions
        .iter()
        .filter_map(|op| {
            if let CoreOp::DefMethod(_, _, name) = op {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();
    assert!(
        method_names.contains(&"processComplexData"),
        "should have processComplexData method, got: {:?}",
        method_names
    );
    assert!(
        method_names.contains(&"healthCheck"),
        "should have healthCheck method, got: {:?}",
        method_names
    );
}

#[test]
fn compile_empty_source_produces_no_instructions() {
    let source = "";
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(source, "empty", language, query, Fidelity::Low, None)
        .expect("compilation should succeed");
    assert!(
        ir.instructions.is_empty(),
        "empty source should produce no instructions"
    );
}

#[test]
fn compile_with_medium_fidelity() {
    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(
            source,
            "test_medium",
            language,
            query,
            Fidelity::Medium,
            None,
        )
        .expect("compilation should succeed");
    assert!(
        !ir.instructions.is_empty(),
        "medium fidelity compilation should produce instructions"
    );
}

#[test]
fn compiler_counter_is_deterministic() {
    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);

    let mut c1 = IRCompiler::new();
    let ir1 = c1
        .compile(source, "f", language.clone(), query, Fidelity::Low, None)
        .unwrap();

    let mut c2 = IRCompiler::new();
    let ir2 = c2
        .compile(source, "f", language, query, Fidelity::Low, None)
        .unwrap();

    assert_eq!(ir1.instructions.len(), ir2.instructions.len());
    for (a, b) in ir1.instructions.iter().zip(ir2.instructions.iter()) {
        assert_eq!(a, b);
    }
}

// ── Symbol targeting (compile-time optimization) ──────────────────

#[test]
fn compile_focused_none_matches_compile() {
    // Byte-identical to plain `compile` when focus is None (backward compat).
    let plain = compile_edit();
    let focused = compile_edit_focused(None);
    assert_eq!(plain.instructions, focused.instructions);
}

#[test]
fn compile_focused_none_emits_all_bodies() {
    let ir = compile_edit_focused(None);
    let names = method_names_by_id(&ir);
    let bodies = body_method_ids(&ir);
    // sample_service.ts has 3 methods: constructor, processComplexData, healthCheck.
    // All should have bodies at Edit fidelity when focus is None.
    assert_eq!(
        bodies.len(),
        3,
        "all 3 methods should have bodies, got {:?}",
        bodies
    );
    for mid in names.keys() {
        assert!(bodies.contains(mid), "method {} should have a body", mid);
    }
}

#[test]
fn compile_focused_some_extracts_only_named_bodies() {
    let focus: HashSet<String> = ["processComplexData".into()].into_iter().collect();
    let ir = compile_edit_focused(Some(&focus));
    let names = method_names_by_id(&ir);
    let bodies = body_method_ids(&ir);

    // Only processComplexData's body should be extracted.
    let expected_mid = names
        .iter()
        .find_map(|(mid, name)| (*name == "processComplexData").then_some(*mid))
        .expect("processComplexData should exist in IR");
    assert_eq!(
        bodies.len(),
        1,
        "only 1 body should be extracted, got {:?}",
        bodies
    );
    assert!(
        bodies.contains(expected_mid),
        "processComplexData body missing"
    );
}

#[test]
fn compile_focused_no_match_extracts_no_bodies() {
    let focus: HashSet<String> = ["NoSuchMethod".into()].into_iter().collect();
    let ir = compile_edit_focused(Some(&focus));
    let bodies = body_method_ids(&ir);
    assert!(
        bodies.is_empty(),
        "no methods should have bodies when focus matches nothing, got {:?}",
        bodies
    );
}

#[test]
fn compile_focused_empty_set_extracts_no_bodies() {
    let focus: HashSet<String> = HashSet::new();
    let ir = compile_edit_focused(Some(&focus));
    let bodies = body_method_ids(&ir);
    assert!(
        bodies.is_empty(),
        "empty focus set should extract no bodies, got {:?}",
        bodies
    );
}

#[test]
fn compile_focused_multiple_names_extracts_only_named_bodies() {
    let focus: HashSet<String> = ["processComplexData".into(), "healthCheck".into()]
        .into_iter()
        .collect();
    let ir = compile_edit_focused(Some(&focus));
    let names = method_names_by_id(&ir);
    let bodies = body_method_ids(&ir);
    let expected_mids: HashSet<&str> = names
        .iter()
        .filter_map(|(mid, name)| {
            (*name == "processComplexData" || *name == "healthCheck").then_some(*mid)
        })
        .collect();
    assert_eq!(
        bodies.len(),
        2,
        "2 methods should have bodies, got {:?}",
        bodies
    );
    assert_eq!(bodies, expected_mids);
}

#[test]
fn compile_focused_non_edit_fidelity_ignores_focus() {
    // Focus is silently ignored at non-Edit fidelities — no bodies are
    // extracted regardless (Edit is the only fidelity that emits bodies).
    let focus: HashSet<String> = ["processComplexData".into()].into_iter().collect();
    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile_focused(
            source,
            "test_high_focused",
            language,
            query,
            Fidelity::High,
            None,
            Some(&focus),
        )
        .expect("compilation should succeed");
    let bodies = body_method_ids(&ir);
    assert!(
        bodies.is_empty(),
        "non-Edit fidelity should produce no bodies regardless of focus, got {:?}",
        bodies
    );
}

#[test]
fn parse_method_sig_simple() {
    let source = r#"
        class Foo {
            public doStuff(name: string): boolean {
                return true;
            }
        }
    "#;
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(source, "test", language, query, Fidelity::Low, None)
        .expect("compilation should succeed");

    let params: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::Param(..)))
        .collect();
    assert!(
        !params.is_empty(),
        "should have Param instruction for doStuff"
    );

    let returns: Vec<_> = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::Return(..)))
        .collect();
    assert!(
        !returns.is_empty(),
        "should have Return instruction for doStuff"
    );
}
