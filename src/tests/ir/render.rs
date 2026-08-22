use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;
use crate::ir::render::ir_to_text;
use crate::ir::wire::op_to_tuple;

fn simple_ir() -> Vec<Vec<String>> {
    vec![
        op_to_tuple(&CoreOp::DefClass("C1".into(), "MyClass".into())),
        op_to_tuple(&CoreOp::DefMethod(
            "C1".into(),
            "M1".into(),
            "doWork".into(),
        )),
        op_to_tuple(&CoreOp::Param(
            "M1".into(),
            "P1".into(),
            "$s".into(),
            "input".into(),
        )),
        op_to_tuple(&CoreOp::Return("M1".into(), "$b".into())),
        op_to_tuple(&CoreOp::Flags("M1".into(), vec!["IF".into()])),
    ]
}

#[test]
fn low_fidelity_renders_class() {
    let output = ir_to_text(&simple_ir(), Fidelity::Low);
    assert!(
        output.contains("$c MyClass"),
        "low fidelity should use $c prefix: {}",
        output
    );
}

#[test]
fn low_fidelity_renders_method() {
    let output = ir_to_text(&simple_ir(), Fidelity::Low);
    assert!(
        output.contains("doWork();"),
        "low fidelity should render method with semicolon: {}",
        output
    );
}

#[test]
fn low_fidelity_renders_flags() {
    let output = ir_to_text(&simple_ir(), Fidelity::Low);
    assert!(
        output.contains("⊕guard"),
        "low fidelity should render ⊕guard: {}",
        output
    );
}

#[test]
fn medium_fidelity_renders_class() {
    let output = ir_to_text(&simple_ir(), Fidelity::Medium);
    assert!(
        output.contains("class MyClass {"),
        "medium fidelity should use 'class' keyword: {}",
        output
    );
}

#[test]
fn medium_fidelity_renders_method() {
    let output = ir_to_text(&simple_ir(), Fidelity::Medium);
    assert!(
        output.contains("doWork("),
        "medium fidelity should render method name: {}",
        output
    );
}

#[test]
fn high_fidelity_renders_indented_method() {
    let output = ir_to_text(&simple_ir(), Fidelity::High);
    assert!(
        output.contains("  doWork("),
        "high fidelity should indent methods: {}",
        output
    );
}

#[test]
fn high_fidelity_renders_flags_in_braces() {
    let output = ir_to_text(&simple_ir(), Fidelity::High);
    assert!(
        output.contains("{ ⊕guard }"),
        "high fidelity should wrap flags in braces: {}",
        output
    );
}

#[test]
fn render_with_return_flag() {
    let ir = vec![
        op_to_tuple(&CoreOp::DefClass("C1".into(), "Foo".into())),
        op_to_tuple(&CoreOp::DefMethod("C1".into(), "M1".into(), "bar".into())),
        op_to_tuple(&CoreOp::Return("M1".into(), "$v".into())),
        op_to_tuple(&CoreOp::Flags("M1".into(), vec!["RET".into()])),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(
        output.contains("⊕⇒"),
        "should render return flag: {}",
        output
    );
}

#[test]
fn render_with_throw_flag() {
    let ir = vec![
        op_to_tuple(&CoreOp::DefClass("C1".into(), "Foo".into())),
        op_to_tuple(&CoreOp::DefMethod("C1".into(), "M1".into(), "baz".into())),
        op_to_tuple(&CoreOp::Flags("M1".into(), vec!["THROW".into()])),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(
        output.contains("⊕!"),
        "should render throw flag: {}",
        output
    );
}

#[test]
fn render_empty_ir() {
    let output = ir_to_text(&[], Fidelity::Low);
    assert!(output.is_empty(), "empty IR should produce empty output");
}

#[test]
fn render_import_low_fidelity() {
    let ir = vec![op_to_tuple(&CoreOp::Import(
        "IM1".into(),
        "rxjs".into(),
        "map".into(),
    ))];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(
        output.contains("$im") && output.contains("$fm"),
        "low fidelity import should use opcodes: {}",
        output
    );
}

#[test]
fn render_import_medium_fidelity() {
    let ir = vec![op_to_tuple(&CoreOp::Import(
        "IM1".into(),
        "rxjs".into(),
        "map".into(),
    ))];
    let output = ir_to_text(&ir, Fidelity::Medium);
    assert!(
        output.contains("import") && output.contains("from"),
        "medium fidelity import should use keywords: {}",
        output
    );
}

#[test]
fn render_def_field_low() {
    let ir = vec![op_to_tuple(&CoreOp::DefField(
        "C1".into(),
        "F1".into(),
        "count".into(),
    ))];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(output.contains("count;"), "low fidelity field: {}", output);
}

#[test]
fn render_def_interface_low() {
    let ir = vec![op_to_tuple(&CoreOp::DefInterface(
        "I1".into(),
        "IMyService".into(),
    ))];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(
        output.contains("$if IMyService"),
        "low fidelity interface: {}",
        output
    );
}

#[test]
fn render_extends() {
    let ir = vec![
        op_to_tuple(&CoreOp::DefClass("C1".into(), "Child".into())),
        op_to_tuple(&CoreOp::Extends("C1".into(), "Parent".into())),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(
        output.contains("$x Parent"),
        "should render extends: {}",
        output
    );
}

#[test]
fn render_type_alias() {
    let ir = vec![op_to_tuple(&CoreOp::TypeAlias(
        "T1".into(),
        "UserId".into(),
    ))];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(
        output.contains("$ty T1=UserId"),
        "should render type alias: {}",
        output
    );
}

/// Round-trip test: compile sample TypeScript -> render at Low fidelity ->
/// verify the output matches expected compressed format.
#[test]
fn round_trip_compile_and_render_low() {
    use crate::compression::language::detect_language;
    use crate::ir::compiler::IRCompiler;

    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);

    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(source, "roundtrip", language, query, Fidelity::Low, None)
        .expect("compilation should succeed");

    // Render back to text
    let tuples: Vec<Vec<String>> = ir.instructions.iter().map(op_to_tuple).collect();
    let output = ir_to_text(&tuples, Fidelity::Low);

    // Every non-empty IR should produce some output
    assert!(
        !output.is_empty(),
        "round-trip low fidelity should produce non-empty output"
    );

    // Should contain the class name in compact opcode format
    assert!(
        output.contains("$c SampleService"),
        "round-trip low should contain $c SampleService: {}",
        output
    );

    // Should contain method with semicolons (low fidelity style)
    assert!(
        output.contains("processComplexData();") || output.contains("healthCheck();"),
        "round-trip low should contain method names with semicolons: {}",
        output
    );

    // Should contain ⊕guard if there are conditionals (sample has 'if')
    assert!(
        output.contains("⊕guard"),
        "round-trip low should contain ⊕guard for if statements: {}",
        output
    );
}

/// Round-trip test: compile sample -> render at Medium fidelity ->
/// verify the output uses natural language keywords.
#[test]
fn round_trip_compile_and_render_medium() {
    use crate::compression::language::detect_language;
    use crate::ir::compiler::IRCompiler;

    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);

    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(source, "roundtrip", language, query, Fidelity::Medium, None)
        .expect("compilation should succeed");

    let tuples: Vec<Vec<String>> = ir.instructions.iter().map(op_to_tuple).collect();
    let output = ir_to_text(&tuples, Fidelity::Medium);

    assert!(
        !output.is_empty(),
        "round-trip medium fidelity should produce non-empty output"
    );

    // Medium fidelity should use 'class' keyword
    assert!(
        output.contains("class SampleService"),
        "round-trip medium should contain 'class SampleService': {}",
        output
    );
}

/// Round-trip test: compile sample -> render at High fidelity ->
/// verify the output uses indentation and behavior markers in braces.
#[test]
fn round_trip_compile_and_render_high() {
    use crate::compression::language::detect_language;
    use crate::ir::compiler::IRCompiler;

    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);

    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(source, "roundtrip", language, query, Fidelity::High, None)
        .expect("compilation should succeed");

    let tuples: Vec<Vec<String>> = ir.instructions.iter().map(op_to_tuple).collect();
    let output = ir_to_text(&tuples, Fidelity::High);

    assert!(
        !output.is_empty(),
        "round-trip high fidelity should produce non-empty output"
    );

    // High fidelity should produce some output with the class name
    assert!(
        output.contains("class SampleService"),
        "round-trip high should contain 'class SampleService': {}",
        output
    );
}

/// Fidelity comparison: same IR compiled once, rendered at Low vs Medium vs High.
/// Verifies that progressive detail is shown across fidelity levels.
#[test]
fn fidelity_comparison_shows_progressive_detail() {
    use crate::compression::language::detect_language;
    use crate::ir::compiler::IRCompiler;

    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);

    // Compile once with Low fidelity (the IR is the same regardless)
    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(
            source,
            "fidelity_test",
            language,
            query,
            Fidelity::Low,
            None,
        )
        .expect("compilation should succeed");

    let tuples: Vec<Vec<String>> = ir.instructions.iter().map(op_to_tuple).collect();

    let low = ir_to_text(&tuples, Fidelity::Low);
    let medium = ir_to_text(&tuples, Fidelity::Medium);
    let high = ir_to_text(&tuples, Fidelity::High);

    // All should be non-empty
    assert!(!low.is_empty(), "low fidelity output should not be empty");
    assert!(
        !medium.is_empty(),
        "medium fidelity output should not be empty"
    );
    assert!(!high.is_empty(), "high fidelity output should not be empty");

    // Low should use compact opcodes ($c prefix for class)
    assert!(low.contains("$c"), "low fidelity should contain $c opcode");

    // Medium should use natural language keywords ('class')
    assert!(
        medium.contains("class"),
        "medium fidelity should contain 'class' keyword"
    );

    // High should also use 'class' keyword
    assert!(
        high.contains("class"),
        "high fidelity should contain 'class' keyword"
    );

    // Low should NOT contain 'class' keyword (uses opcodes instead)
    assert!(
        !low.contains("class "),
        "low fidelity should NOT contain 'class' keyword (uses $c): {}",
        low
    );

    // Low should have semicolons after methods
    assert!(
        low.contains(");"),
        "low fidelity methods should end with semicolons: {}",
        low
    );

    // Medium should NOT have semicolons after methods
    assert!(
        !medium.contains(");"),
        "medium fidelity methods should NOT end with semicolons: {}",
        medium
    );

    // High should be longer than medium (more whitespace/indentation)
    assert!(
        high.len() >= medium.len(),
        "high fidelity output should be at least as long as medium (more indentation). high={}, medium={}",
        high.len(),
        medium.len()
    );
}

/// Verify that compiling the same source at different fidelities
/// produces instruction streams with similar structure (not necessarily
/// identical, because fidelity can affect capture names like `async`).
#[test]
fn compilation_fidelity_produces_comparable_ir() {
    use crate::compression::language::detect_language;
    use crate::ir::compiler::IRCompiler;

    let source = include_str!("../../test_files/sample_service.ts");
    let (language, query) = detect_language(source);

    let mut c_low = IRCompiler::new();
    let ir_low = c_low
        .compile(source, "test", language.clone(), query, Fidelity::Low, None)
        .expect("low fidelity compilation should succeed");

    let mut c_high = IRCompiler::new();
    let ir_high = c_high
        .compile(source, "test", language, query, Fidelity::High, None)
        .expect("high fidelity compilation should succeed");

    // Both should produce at least some DefClass, DefMethod instructions
    let low_classes: usize = ir_low
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefClass(..)))
        .count();
    let high_classes: usize = ir_high
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefClass(..)))
        .count();
    assert_eq!(
        low_classes, high_classes,
        "both fidelities should produce the same number of class definitions"
    );

    let low_methods: usize = ir_low
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefMethod(..)))
        .count();
    let high_methods: usize = ir_high
        .instructions
        .iter()
        .filter(|op| matches!(op, CoreOp::DefMethod(..)))
        .count();
    assert_eq!(
        low_methods, high_methods,
        "both fidelities should produce the same number of method definitions"
    );

    // Filename and version should match
    assert_eq!(ir_low.file_id, ir_high.file_id);
    assert_eq!(ir_low.version, ir_high.version);
}
