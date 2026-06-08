use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;
use crate::ir::wire::op_to_tuple;
use crate::ir::render::ir_to_text;

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
    assert!(output.contains("$c MyClass"), "low fidelity should use $c prefix: {}", output);
}

#[test]
fn low_fidelity_renders_method() {
    let output = ir_to_text(&simple_ir(), Fidelity::Low);
    assert!(output.contains("doWork();"), "low fidelity should render method with semicolon: {}", output);
}

#[test]
fn low_fidelity_renders_flags() {
    let output = ir_to_text(&simple_ir(), Fidelity::Low);
    assert!(output.contains("⊕guard"), "low fidelity should render ⊕guard: {}", output);
}

#[test]
fn medium_fidelity_renders_class() {
    let output = ir_to_text(&simple_ir(), Fidelity::Medium);
    assert!(output.contains("class MyClass {"), "medium fidelity should use 'class' keyword: {}", output);
}

#[test]
fn medium_fidelity_renders_method() {
    let output = ir_to_text(&simple_ir(), Fidelity::Medium);
    assert!(output.contains("doWork()"), "medium fidelity should render method: {}", output);
}

#[test]
fn high_fidelity_renders_indented_method() {
    let output = ir_to_text(&simple_ir(), Fidelity::High);
    assert!(output.contains("  doWork()"), "high fidelity should indent methods: {}", output);
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
    assert!(output.contains("⊕⇒"), "should render return flag: {}", output);
}

#[test]
fn render_with_throw_flag() {
    let ir = vec![
        op_to_tuple(&CoreOp::DefClass("C1".into(), "Foo".into())),
        op_to_tuple(&CoreOp::DefMethod("C1".into(), "M1".into(), "baz".into())),
        op_to_tuple(&CoreOp::Flags("M1".into(), vec!["THROW".into()])),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(output.contains("⊕!"), "should render throw flag: {}", output);
}

#[test]
fn render_empty_ir() {
    let output = ir_to_text(&[], Fidelity::Low);
    assert!(output.is_empty(), "empty IR should produce empty output");
}

#[test]
fn render_import_low_fidelity() {
    let ir = vec![
        op_to_tuple(&CoreOp::Import(
            "IM1".into(),
            "rxjs".into(),
            "map".into(),
        )),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(
        output.contains("$im") && output.contains("$fm"),
        "low fidelity import should use opcodes: {}",
        output
    );
}

#[test]
fn render_import_medium_fidelity() {
    let ir = vec![
        op_to_tuple(&CoreOp::Import(
            "IM1".into(),
            "rxjs".into(),
            "map".into(),
        )),
    ];
    let output = ir_to_text(&ir, Fidelity::Medium);
    assert!(
        output.contains("import") && output.contains("from"),
        "medium fidelity import should use keywords: {}",
        output
    );
}

#[test]
fn render_def_field_low() {
    let ir = vec![
        op_to_tuple(&CoreOp::DefField("C1".into(), "F1".into(), "count".into())),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(output.contains("count;"), "low fidelity field: {}", output);
}

#[test]
fn render_def_interface_low() {
    let ir = vec![
        op_to_tuple(&CoreOp::DefInterface("I1".into(), "IMyService".into())),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(output.contains("$if IMyService"), "low fidelity interface: {}", output);
}

#[test]
fn render_extends() {
    let ir = vec![
        op_to_tuple(&CoreOp::DefClass("C1".into(), "Child".into())),
        op_to_tuple(&CoreOp::Extends("C1".into(), "Parent".into())),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(output.contains("$x Parent"), "should render extends: {}", output);
}

#[test]
fn render_type_alias() {
    let ir = vec![
        op_to_tuple(&CoreOp::TypeAlias("T1".into(), "UserId".into())),
    ];
    let output = ir_to_text(&ir, Fidelity::Low);
    assert!(output.contains("$ty T1=UserId"), "should render type alias: {}", output);
}