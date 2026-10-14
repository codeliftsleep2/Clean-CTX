#![cfg(feature = "typescript")]

use super::{ValueDeclarationKind, bind_direct_value_identifier};
use crate::cache::content_hash;
use crate::ir::invocation_evidence::SourceSpan;

fn nth_span(source: &str, needle: &str, occurrence: usize) -> SourceSpan {
    let start = source
        .match_indices(needle)
        .nth(occurrence)
        .map(|(start, _)| start)
        .expect("fixture occurrence");
    SourceSpan::new(start, start + needle.len()).unwrap()
}

fn binding(source: &str, reference: SourceSpan) -> super::ReferenceBinding {
    bind_direct_value_identifier("Fixture.ts", source, reference)
        .expect("direct value identifier must bind")
}

#[test]
fn top_level_local_and_parameter_shadowing_bind_exact_declarations() {
    let source = concat!(
        "const target = 1;\n",
        "function outer() { use(target); }\n",
        "function local() { const target = 2; use(target); }\n",
        "function parameter(target: number) { use(target); }\n",
    );

    let outer = binding(source, nth_span(source, "target", 1));
    assert_eq!(outer.declaration_span(), nth_span(source, "target", 0));
    assert_eq!(outer.declaration_kind(), ValueDeclarationKind::LexicalConst);
    assert_eq!(outer.reference_span(), nth_span(source, "target", 1));
    assert_eq!(outer.source_id(), "Fixture.ts");
    assert_eq!(outer.source_hash(), content_hash(source.as_bytes()));

    let local = binding(source, nth_span(source, "target", 3));
    assert_eq!(local.declaration_span(), nth_span(source, "target", 2));
    assert_eq!(local.declaration_kind(), ValueDeclarationKind::LexicalConst);

    let parameter = binding(source, nth_span(source, "target", 5));
    assert_eq!(parameter.declaration_span(), nth_span(source, "target", 4));
    assert_eq!(parameter.declaration_kind(), ValueDeclarationKind::Parameter);
}

#[test]
fn nested_block_scope_exits_and_separate_functions_do_not_cross_bind() {
    let source = concat!(
        "const target = 0;\n",
        "function a() { { const target = 1; use(target); } use(target); }\n",
        "function b() { const target = 2; use(target); }\n",
    );

    assert_eq!(
        binding(source, nth_span(source, "target", 2)).declaration_span(),
        nth_span(source, "target", 1)
    );
    assert_eq!(
        binding(source, nth_span(source, "target", 3)).declaration_span(),
        nth_span(source, "target", 0)
    );
    assert_eq!(
        binding(source, nth_span(source, "target", 5)).declaration_span(),
        nth_span(source, "target", 4)
    );
}

#[test]
fn only_direct_value_identifier_occurrences_are_admitted() {
    let source = concat!(
        "const target = 1;\n",
        "use(\"target\"); obj.target; wrapper(target); ",
        "condition ? target : other; [target]; () => target;\n",
    );

    for occurrence in [1, 2] {
        assert!(
            bind_direct_value_identifier(
                "Fixture.ts",
                source,
                nth_span(source, "target", occurrence),
            )
            .is_none(),
            "string and member-property occurrences must be refused"
        );
    }
    for expression in [
        "wrapper(target)",
        "condition ? target : other",
        "[target]",
        "() => target",
    ] {
        assert!(
            bind_direct_value_identifier(
                "Fixture.ts",
                source,
                nth_span(source, expression, 0),
            )
            .is_none(),
            "a compound expression span is not a direct identifier occurrence"
        );
    }
    assert!(
        bind_direct_value_identifier("Fixture.ts", source, nth_span(source, "target", 0)).is_none(),
        "a declaration name is not a reference"
    );
}

#[test]
fn let_is_supported_while_var_and_import_bindings_fail_closed() {
    let source = concat!(
        "let local = 1; use(local);\n",
        "var legacy = 2; use(legacy);\n",
        "import { remote } from './remote'; use(remote);\n",
    );

    let local = binding(source, nth_span(source, "local", 1));
    assert_eq!(local.declaration_span(), nth_span(source, "local", 0));
    assert_eq!(local.declaration_kind(), ValueDeclarationKind::LexicalLet);
    assert!(
        bind_direct_value_identifier("Fixture.ts", source, nth_span(source, "legacy", 1))
            .is_none()
    );
    assert!(
        bind_direct_value_identifier("Fixture.ts", source, nth_span(source, "remote", 1))
            .is_none()
    );
}

#[test]
fn same_text_in_another_file_cannot_supply_a_declaration() {
    let file_a = "const target = 1; use(target);";
    let file_b = "use(target);";
    let in_a = binding(file_a, nth_span(file_a, "target", 1));
    assert_eq!(in_a.source_id(), "Fixture.ts");
    assert!(
        bind_direct_value_identifier("Other.ts", file_b, nth_span(file_b, "target", 0)).is_none()
    );
}

#[test]
fn ordinary_framework_shaped_name_binds_without_framework_classification() {
    let source = "const selectAudit = 123; use(selectAudit);";
    let result = binding(source, nth_span(source, "selectAudit", 1));
    assert_eq!(result.declaration_span(), nth_span(source, "selectAudit", 0));
    assert_eq!(result.declaration_kind(), ValueDeclarationKind::LexicalConst);
}
