// src/tests/ir/observable_pattern_semantics.rs
//
// Semantic separation between Observable and Promise pattern classifications.

use crate::ir::opcodes::{CoreOp, DeclarationModifier};
use crate::ir::patterns::{CompressingPatternRecognizer, PatternOp};

fn method_with_return(return_type: &str) -> Vec<CoreOp> {
    vec![
        CoreOp::DefMethod("C1".into(), "M1".into(), "load".into()),
        CoreOp::Return("M1".into(), return_type.into()),
    ]
}

#[test]
fn observable_semantics_consumptive_recognizes_declared_observable_without_async() {
    let operations = method_with_return("Observable<User>");
    let (patterns, _) = CompressingPatternRecognizer::new().compress(&operations);

    assert!(
        matches!(patterns.as_slice(), [PatternOp::Observable { .. }]),
        "an Observable return must not be classified as Promise: {patterns:?}"
    );
}

#[test]
fn observable_semantics_consumptive_keeps_async_promise_as_promise() {
    let mut operations = method_with_return("$P");
    operations.push(CoreOp::MethodModifiers(
        "M1".into(),
        vec![DeclarationModifier::Async],
    ));
    let (patterns, _) = CompressingPatternRecognizer::new().compress(&operations);

    assert!(
        matches!(patterns.as_slice(), [PatternOp::Promise { .. }]),
        "async does not change a Promise return into an Observable: {patterns:?}"
    );
}
