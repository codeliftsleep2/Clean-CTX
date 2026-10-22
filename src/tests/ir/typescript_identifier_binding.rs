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

fn canonical_binding(source: &str, reference: SourceSpan) -> super::ReferenceBinding {
    let language = crate::compression::language::safe_typescript_language().unwrap();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(source, None).unwrap();
    super::bind_value_identifier_in_tree("Fixture.ts", source, reference, tree.root_node())
        .expect("canonical declaration binding")
}

#[test]
fn issue_162_catch_parameter_shadows_only_its_catch_and_keeps_legacy_role_boundary() {
    let source = "const error = 1; try {} catch (error) { use(error); } use(error);";
    let inside = nth_span(source, "error", 2);
    let outside = nth_span(source, "error", 3);
    let canonical = canonical_binding(source, inside);
    assert_eq!(
        canonical.declaration_kind(),
        ValueDeclarationKind::CatchParameter
    );
    assert_eq!(canonical.declaration_span(), nth_span(source, "error", 1));
    assert!(bind_direct_value_identifier("Fixture.ts", source, inside).is_none());
    assert_eq!(
        binding(source, outside).declaration_span(),
        nth_span(source, "error", 0)
    );
}

#[test]
fn issue_162_optional_catch_binding_does_not_invent_a_shadow() {
    let source = "const error = 1; try {} catch { use(error); }";
    assert_eq!(
        canonical_binding(source, nth_span(source, "error", 1)).declaration_span(),
        nth_span(source, "error", 0)
    );
}

#[test]
fn issue_162_var_binding_hoists_through_blocks_but_not_nested_functions() {
    let source = "function f() { if (flag) { var item = 1; } use(item); } function g() { var item = 2; use(item); }";
    let first = canonical_binding(source, nth_span(source, "item", 1));
    let second = canonical_binding(source, nth_span(source, "item", 3));
    assert_eq!(first.declaration_span(), nth_span(source, "item", 0));
    assert_eq!(second.declaration_span(), nth_span(source, "item", 2));
    assert_eq!(first.declaration_kind(), ValueDeclarationKind::FunctionVar);
    assert_eq!(second.declaration_kind(), ValueDeclarationKind::FunctionVar);
}

#[test]
fn issue_162_block_let_shadows_function_var_without_retargeting_outer_uses() {
    let source = "function f() { var item = 1; { let item = 2; use(item); } use(item); }";
    assert_eq!(
        canonical_binding(source, nth_span(source, "item", 2)).declaration_kind(),
        ValueDeclarationKind::LexicalLet
    );
    assert_eq!(
        canonical_binding(source, nth_span(source, "item", 3)).declaration_span(),
        nth_span(source, "item", 0)
    );
}

#[test]
fn issue_162_loop_header_let_does_not_shadow_an_outer_reference_after_the_loop() {
    let source = "const item = 1; for (let item = 0; item < 2; item++) { use(item); } use(item);";
    let outer = binding(source, nth_span(source, "item", 5));
    assert_eq!(outer.declaration_span(), nth_span(source, "item", 0));
    let inner = binding(source, nth_span(source, "item", 4));
    assert_eq!(inner.declaration_span(), nth_span(source, "item", 1));
    assert_eq!(inner.declaration_kind(), ValueDeclarationKind::LexicalLet);
}

#[test]
fn issue_162_expression_arrow_parameter_keeps_its_own_lexical_scope() {
    let source = "const item = 1; const f = (item: number) => item; use(item);";
    let inner = binding(source, nth_span(source, "item", 2));
    assert_eq!(inner.declaration_span(), nth_span(source, "item", 1));
    assert_eq!(inner.declaration_kind(), ValueDeclarationKind::Parameter);
    assert_eq!(
        binding(source, nth_span(source, "item", 3)).declaration_span(),
        nth_span(source, "item", 0)
    );
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
    assert_eq!(
        parameter.declaration_kind(),
        ValueDeclarationKind::Parameter
    );
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
            bind_direct_value_identifier("Fixture.ts", source, nth_span(source, expression, 0),)
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
        bind_direct_value_identifier("Fixture.ts", source, nth_span(source, "legacy", 1)).is_none()
    );
    assert!(
        bind_direct_value_identifier("Fixture.ts", source, nth_span(source, "remote", 1)).is_none()
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
    assert_eq!(
        result.declaration_span(),
        nth_span(source, "selectAudit", 0)
    );
    assert_eq!(
        result.declaration_kind(),
        ValueDeclarationKind::LexicalConst
    );
}

#[test]
fn sibling_blocks_do_not_leak_and_parameter_returns_after_inner_shadow() {
    let source = concat!(
        "function sibling(target: number) {\n",
        "  { const target = 1; use(target); }\n",
        "  { const target = 2; use(target); }\n",
        "  use(target);\n",
        "}\n",
    );

    assert_eq!(
        binding(source, nth_span(source, "target", 2)).declaration_span(),
        nth_span(source, "target", 1)
    );
    assert_eq!(
        binding(source, nth_span(source, "target", 4)).declaration_span(),
        nth_span(source, "target", 3)
    );
    let post_block = binding(source, nth_span(source, "target", 5));
    assert_eq!(post_block.declaration_span(), nth_span(source, "target", 0));
    assert_eq!(
        post_block.declaration_kind(),
        ValueDeclarationKind::Parameter
    );
}

#[test]
fn differently_named_declarations_bind_only_the_referenced_identity() {
    let source = "const first = 1; const second = 2; use(second); use(first);";
    assert_eq!(
        binding(source, nth_span(source, "second", 1)).declaration_span(),
        nth_span(source, "second", 0)
    );
    assert_eq!(
        binding(source, nth_span(source, "first", 1)).declaration_span(),
        nth_span(source, "first", 0)
    );
}

#[test]
fn duplicate_declarations_malformed_source_and_type_positions_fail_closed() {
    let duplicate = "const target = 1; const target = 2; use(target);";
    assert!(
        bind_direct_value_identifier("Fixture.ts", duplicate, nth_span(duplicate, "target", 2),)
            .is_none()
    );

    let malformed = "const target = ; use(target);";
    assert!(
        bind_direct_value_identifier("Fixture.ts", malformed, nth_span(malformed, "target", 1),)
            .is_none()
    );

    let type_position = "type Alias = target; const target = 1; use(target);";
    assert!(
        bind_direct_value_identifier(
            "Fixture.ts",
            type_position,
            nth_span(type_position, "target", 0),
        )
        .is_none()
    );
    assert_eq!(
        binding(type_position, nth_span(type_position, "target", 2)).declaration_span(),
        nth_span(type_position, "target", 1)
    );
}

#[test]
fn lexical_identity_is_scope_wide_while_tdz_runtime_access_is_out_of_scope() {
    let source = "function f() { use(target); const target = 1; }";
    let result = binding(source, nth_span(source, "target", 0));
    assert_eq!(result.declaration_span(), nth_span(source, "target", 1));
    assert_eq!(
        result.declaration_kind(),
        ValueDeclarationKind::LexicalConst
    );
}

#[test]
fn equivalent_inputs_are_deterministic_and_source_bound() {
    let source = "const target = 1; use(target);";
    let reference = nth_span(source, "target", 1);
    let first = bind_direct_value_identifier("Fixture.ts", source, reference);
    let unrelated = bind_direct_value_identifier(
        "Other.ts",
        "const other = 2; use(other);",
        SourceSpan::new(21, 26).unwrap(),
    );
    let second = bind_direct_value_identifier("Fixture.ts", source, reference);

    assert!(unrelated.is_some());
    assert_eq!(first, second);
    assert_eq!(
        first.unwrap().source_hash(),
        content_hash(source.as_bytes())
    );
}

#[test]
fn invocation_argument_span_is_directly_compatible_with_the_binder() {
    let source = "const target = 1; consume(target);";
    let evidence = crate::ir::invocation_evidence::collect_typescript_source(source).unwrap();
    let view = evidence.view("standalone-typescript", source).unwrap();
    let argument_span = view
        .occurrences()
        .iter()
        .flat_map(|occurrence| occurrence.arguments())
        .find(|argument| view.slice(argument.span()).unwrap() == "target")
        .expect("consume argument")
        .span();

    let result = bind_direct_value_identifier("standalone-typescript", source, argument_span)
        .expect("exact invocation argument span must bind");
    assert_eq!(result.reference_span(), argument_span);
    assert_eq!(result.declaration_span(), nth_span(source, "target", 0));
    assert_eq!(result.source_id(), evidence.source_id());
    assert_eq!(result.source_hash(), evidence.source_hash());
}
