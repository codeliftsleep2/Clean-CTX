// Nested-type hierarchy rendering regression split from `spans.rs` to keep
// the active test modules within the repository file-size ceiling.

use super::{Fidelity, nested_service_source};

#[test]
fn nested_enum_outer_class_renders_without_static_and_enum_stays_nested() {
    use crate::compression::language::detect_language;
    use crate::ir::compiler::IRCompiler;
    use crate::ir::hierarchical::ir_to_hierarchical;
    use crate::ir::render_llm::render_hierarchical_for_llm;

    // Medium fidelity so the nested enum's two members survive as fields
    // (`extract_field` suppresses fields at Low). The C# language layer
    // must be registered: `ClassFlags` (EXPORT/STATIC) are emitted by the
    // layer, not Core IR — a bare `IRCompiler::new()` yields no flags.
    let source = nested_service_source();
    let (language, query) = detect_language(&source);
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
    let ir = compiler
        .compile(
            &source,
            "nested_service_render",
            language,
            query,
            Fidelity::Medium,
            None,
        )
        .expect("compilation should succeed");
    let hir = ir_to_hierarchical(&ir);
    let rendered = render_hierarchical_for_llm(&hir, Fidelity::Medium);

    let service = hir
        .classes
        .iter()
        .find(|c| c.name == "SomeService")
        .expect("outer class must exist in hierarchical IR");
    let flags = service
        .class_flags
        .iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        flags.iter().any(|f| f == "EXPORT"),
        "outer class keeps EXPORT, got: {flags:?}"
    );
    assert!(
        !flags.iter().any(|f| f == "STATIC"),
        "non-static outer class must never render STATIC, got: {flags:?}"
    );
    assert!(
        rendered.contains("cl: EXPORT\n"),
        "rendered output must show `cl: EXPORT` without STATIC, got:\n{rendered}"
    );
    assert!(
        !rendered.contains("STATIC"),
        "no STATIC anywhere for this fixture, got:\n{rendered}"
    );

    let method_names: Vec<&str> = service.methods.iter().map(|m| m.name.as_str()).collect();
    assert!(
        method_names.contains(&"Before") && method_names.contains(&"After"),
        "outer class must own both methods, got: {method_names:?}"
    );
    let enum_node = hir
        .classes
        .iter()
        .find(|c| c.name == "SomeStatus")
        .expect("nested enum must render as its own section");
    assert!(
        enum_node.methods.is_empty(),
        "nested enum must own zero ordinary methods"
    );
    assert_eq!(
        enum_node.fields.len(),
        2,
        "nested enum keeps its two members, got: {:?}",
        enum_node.fields
    );
}
