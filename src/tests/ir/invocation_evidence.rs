use super::*;
use crate::compression::Fidelity;
use crate::compression::language::detect_language;
use crate::ir::compiler::IRCompiler;

fn span(start_byte: usize, end_byte: usize) -> SourceSpan {
    SourceSpan::new(start_byte, end_byte).expect("valid test span")
}

fn occurrence(start_byte: usize, end_byte: usize) -> InvocationOccurrence {
    InvocationOccurrence::new(
        span(start_byte, end_byte),
        WrittenCallee::Unsupported { function: None },
        Vec::new(),
        Vec::new(),
        None,
        None,
    )
}

#[test]
fn evidence_set_sorts_by_start_then_containing_span_first() {
    let set = InvocationEvidenceSet::new(
        "Example.ts".into(),
        "abcdef",
        vec![occurrence(2, 3), occurrence(0, 2), occurrence(0, 5)],
    );
    let view = set.view("Example.ts", "abcdef").expect("matching source");
    let spans = view
        .occurrences()
        .iter()
        .map(InvocationOccurrence::span)
        .collect::<Vec<_>>();

    assert_eq!(spans, [span(0, 5), span(0, 2), span(2, 3)]);
}

#[test]
fn evidence_view_rejects_other_identity_and_edited_bytes() {
    let set = InvocationEvidenceSet::new("Example.ts".into(), "foo()", Vec::new());

    assert_eq!(
        set.view("Other.ts", "foo()").err(),
        Some(EvidenceMismatch::SourceIdentityMismatch)
    );
    assert_eq!(
        set.view("Example.ts", "bar()").err(),
        Some(EvidenceMismatch::SourceHashMismatch)
    );
}

#[test]
fn validated_view_slices_utf8_bytes_and_rejects_invalid_boundaries() {
    let source = "éfoo()";
    let set = InvocationEvidenceSet::new("Example.ts".into(), source, Vec::new());
    let view = set.view("Example.ts", source).expect("matching source");

    assert_eq!(view.slice(span(2, 7)).expect("ASCII suffix"), "foo()");
    assert!(matches!(
        view.slice(span(1, 2)),
        Err(EvidenceSliceError::NotUtf8Boundary { .. })
    ));
    assert!(matches!(
        view.slice(span(0, 8)),
        Err(EvidenceSliceError::OutOfBounds { .. })
    ));
}

#[test]
fn constructors_preserve_the_frozen_structural_contract() {
    let argument = ArgumentOccurrence::new(
        span(10, 14),
        ArgumentQualifier::Named { name: span(10, 11) },
    );
    let context = DeclarationContext::new(
        DeclarationContextKind::VariableInitializer,
        span(0, 20),
        Some(span(6, 9)),
    );
    let parent = InvocationParent::new(span(0, 30), ParentSite::Argument { index: 1 });
    let occurrence = InvocationOccurrence::new(
        span(5, 18),
        WrittenCallee::Member {
            receiver: span(5, 9),
            terminal: span(10, 14),
        },
        vec![argument],
        vec![span(14, 17)],
        Some(context),
        Some(parent),
    );

    assert_eq!(occurrence.span(), span(5, 18));
    assert_eq!(occurrence.arguments()[0].span(), span(10, 14));
    assert!(matches!(
        occurrence.arguments()[0].qualifier(),
        ArgumentQualifier::Named { name } if *name == span(10, 11)
    ));
    assert_eq!(occurrence.type_arguments(), [span(14, 17)]);
    assert_eq!(
        occurrence.declaration_context().expect("context").kind(),
        DeclarationContextKind::VariableInitializer
    );
    assert_eq!(
        occurrence.parent().expect("parent").site(),
        ParentSite::Argument { index: 1 }
    );
}

#[test]
fn successful_compile_publishes_empty_phase_one_side_output() {
    let source = "class Example { run(): void {} }";
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();

    compiler
        .compile_focused(
            source,
            "alias",
            Some("C:/repo/Example.ts"),
            language,
            query,
            Fidelity::Low,
            None,
            None,
        )
        .expect("compilation succeeds");

    let evidence = compiler
        .invocation_evidence
        .as_ref()
        .expect("successful compile publishes its side output");
    assert_eq!(evidence.source_id(), "C:/repo/Example.ts");
    assert_eq!(
        evidence.source_hash(),
        crate::cache::content_hash(source.as_bytes())
    );
    assert!(
        evidence
            .view("C:/repo/Example.ts", source)
            .expect("bound view")
            .occurrences()
            .is_empty(),
        "Phase 1 establishes lifecycle but does not populate language evidence"
    );
}

#[test]
fn failed_compile_clears_previous_invocation_evidence() {
    let source = "class Example { run(): void {} }";
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    compiler
        .compile(
            source,
            "Example.ts",
            language.clone(),
            query,
            Fidelity::Low,
            None,
        )
        .expect("initial compilation succeeds");
    assert!(compiler.invocation_evidence.is_some());

    let result = compiler.compile(source, "Example.ts", language, "(", Fidelity::Low, None);
    assert!(result.is_err(), "invalid query must fail compilation");
    assert!(
        compiler.invocation_evidence.is_none(),
        "failed compilation must not retain stale source-bound evidence"
    );
}
