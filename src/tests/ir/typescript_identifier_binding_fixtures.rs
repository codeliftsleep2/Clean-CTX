#![cfg(feature = "typescript")]

use super::{ValueDeclarationKind, bind_direct_value_identifier};
use crate::cache::content_hash;
use crate::ir::invocation_evidence::{SourceSpan, collect_typescript_source};

const ORACLE: &str = include_str!("../../../verification/typescript-identifier-binding/oracle.tsv");

fn fixture(name: &str) -> &'static str {
    match name {
        "shadowing.ts" => include_str!(
            "../../../verification/typescript-identifier-binding/fixtures/shadowing.ts"
        ),
        "removal-parameter.ts" => include_str!(
            "../../../verification/typescript-identifier-binding/fixtures/removal-parameter.ts"
        ),
        "removal-top.ts" => include_str!(
            "../../../verification/typescript-identifier-binding/fixtures/removal-top.ts"
        ),
        "unsupported.ts" => include_str!(
            "../../../verification/typescript-identifier-binding/fixtures/unsupported.ts"
        ),
        "file-a.ts" => {
            include_str!("../../../verification/typescript-identifier-binding/fixtures/file-a.ts")
        }
        "file-b.ts" => {
            include_str!("../../../verification/typescript-identifier-binding/fixtures/file-b.ts")
        }
        _ => panic!("unknown fixture {name}"),
    }
}

fn marker_end(source: &str, label: &str) -> usize {
    let marker = format!("/*{label}*/");
    source.find(&marker).expect("oracle marker") + marker.len()
}

fn marked_name_span(source: &str, label: &str, name: &str) -> SourceSpan {
    let search_start = marker_end(source, label);
    let relative = source[search_start..]
        .find(name)
        .expect("name after oracle marker");
    let start = search_start + relative;
    SourceSpan::new(start, start + name.len()).unwrap()
}

fn invocation_argument_span(source: &str, label: &str, text: &str) -> SourceSpan {
    let evidence = collect_typescript_source(source).expect("TypeScript invocation evidence");
    let view = evidence
        .view("standalone-typescript", source)
        .expect("current source view");
    let after = marker_end(source, label);
    view.occurrences()
        .iter()
        .flat_map(|occurrence| occurrence.arguments())
        .filter(|argument| argument.span().start_byte() >= after)
        .filter(|argument| view.slice(argument.span()).ok() == Some(text))
        .min_by_key(|argument| argument.span().start_byte())
        .unwrap_or_else(|| panic!("argument {text:?} after marker {label}"))
        .span()
}

fn expected_kind(value: &str) -> ValueDeclarationKind {
    match value {
        "LexicalConst" => ValueDeclarationKind::LexicalConst,
        "LexicalLet" => ValueDeclarationKind::LexicalLet,
        "Parameter" => ValueDeclarationKind::Parameter,
        _ => panic!("unknown declaration kind {value}"),
    }
}

#[test]
fn tracked_fixtures_match_independent_oracle_through_real_invocation_spans() {
    for line in ORACLE
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let columns: Vec<_> = line.split('|').collect();
        assert_eq!(columns.len(), 6, "oracle row shape: {line}");
        let [
            file,
            reference_label,
            argument_text,
            declaration_label,
            declaration_name,
            kind,
        ] = columns.as_slice()
        else {
            unreachable!("column count checked")
        };
        let source = fixture(file);
        let reference_span = invocation_argument_span(source, reference_label, argument_text);
        let result = bind_direct_value_identifier(file, source, reference_span);

        if *declaration_label == "REFUSE" {
            assert!(result.is_none(), "{file}:{reference_label} must refuse");
            continue;
        }

        let result = result.unwrap_or_else(|| panic!("{file}:{reference_label} must bind"));
        assert_eq!(
            result.reference_span(),
            reference_span,
            "{file}:{reference_label}"
        );
        assert_eq!(
            result.declaration_span(),
            marked_name_span(source, declaration_label, declaration_name),
            "{file}:{reference_label}"
        );
        assert_eq!(result.declaration_kind(), expected_kind(kind));
        assert_eq!(result.source_id(), *file);
        assert_eq!(result.source_hash(), content_hash(source.as_bytes()));
    }
}

#[test]
fn marked_non_invocation_positions_refuse_through_real_typescript_parse() {
    let source = fixture("unsupported.ts");
    for (label, name) in [
        ("marked:type", "target"),
        ("marked:declaration", "declared"),
    ] {
        assert!(
            bind_direct_value_identifier(
                "unsupported.ts",
                source,
                marked_name_span(source, label, name),
            )
            .is_none(),
            "{label} must not be admitted as a value reference"
        );
    }
}

#[test]
fn fixture_order_does_not_change_binding_truth() {
    let rows: Vec<_> = ORACLE
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    let evaluate = |line: &&str| {
        let columns: Vec<_> = line.split('|').collect();
        let source = fixture(columns[0]);
        let reference = invocation_argument_span(source, columns[1], columns[2]);
        bind_direct_value_identifier(columns[0], source, reference)
            .map(|binding| (binding.declaration_span(), binding.declaration_kind()))
    };
    let forward: Vec<_> = rows.iter().map(evaluate).collect();
    let mut reverse: Vec<_> = rows.iter().rev().map(evaluate).collect();
    reverse.reverse();
    assert_eq!(forward, reverse);
}
