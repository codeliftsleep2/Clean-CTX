use crate::compression::Fidelity;
use crate::compression::language::detect_language;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::opcodes::CoreOp;

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
        .compile(source, "test_medium", language, query, Fidelity::Medium, None)
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
    let ir1 = c1.compile(source, "f", language.clone(), query, Fidelity::Low, None).unwrap();

    let mut c2 = IRCompiler::new();
    let ir2 = c2.compile(source, "f", language, query, Fidelity::Low, None).unwrap();

    assert_eq!(ir1.instructions.len(), ir2.instructions.len());
    for (a, b) in ir1.instructions.iter().zip(ir2.instructions.iter()) {
        assert_eq!(a, b);
    }
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