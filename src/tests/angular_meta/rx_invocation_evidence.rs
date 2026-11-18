//! Phase 0 RED plus preservation oracle for the RxJS pipe proof slice.

#![cfg(feature = "typescript")]

use super::extract_rx_shape;
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{CapturedNode, run_capture_pipeline_nodes};
use crate::queries::{TS_CALL_QUERY, TS_QUERY};

fn invocation_captures(source: &str) -> Vec<CapturedNode> {
    let language = crate::compression::language::safe_typescript_language()
        .expect("typescript grammar enabled");
    let query = format!("{TS_QUERY}\n{TS_CALL_QUERY}");
    run_capture_pipeline_nodes(language, &query, source, Fidelity::High, |_, raw, _| {
        Some(raw.to_string())
    })
    .expect("the production capture query must compile")
}

#[test]
fn pipe_occurrence_boundaries_preserve_the_existing_rxjs_shape() {
    let source = r#"
import { map, switchMap, tap } from 'rxjs/operators';
class Example {
  result$ = this.source.pipe(
    map(value => ({ value, label: 'item,' + value })),
    switchMap(value => this.service.load(value, { retry: [1, 2] })),
    tap(value => console.log(value))
  );
}
"#;

    let shape = extract_rx_shape(source, Fidelity::High).expect("RxJS shape");
    assert_eq!(shape.pipes.len(), 1);
    assert_eq!(shape.pipes[0].owner, "result$");
    assert_eq!(
        shape.pipes[0]
            .operators
            .iter()
            .map(|operator| operator.operator_name.as_str())
            .collect::<Vec<_>>(),
        ["map", "switchMap", "tap"],
        "this is the frozen legacy result the structural migration must preserve"
    );

    let captures = invocation_captures(source);
    let pipe_roots = captures
        .iter()
        .filter(|capture| capture.name == "invoke.root" && capture.raw_text.contains(".pipe("))
        .collect::<Vec<_>>();
    let pipe = pipe_roots
        .first()
        .expect("the shared occurrence authority must expose the pipe invocation root");
    let pipe_matches = pipe_roots
        .iter()
        .filter(|root| root.start_byte == pipe.start_byte && root.end_byte == pipe.end_byte)
        .map(|root| root.match_index)
        .collect::<std::collections::BTreeSet<_>>();
    let mut argument_spans = std::collections::BTreeSet::new();
    let arguments = captures
        .iter()
        .filter(|capture| {
            capture.name == "invoke.argument"
                && pipe_matches.contains(&capture.match_index)
                && argument_spans.insert((capture.start_byte, capture.end_byte))
        })
        .map(|capture| capture.raw_text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        arguments.len(),
        3,
        "grammar boundaries must preserve three top-level operators"
    );
    assert!(arguments[0].starts_with("map("));
    assert!(arguments[1].starts_with("switchMap("));
    assert!(arguments[2].starts_with("tap("));
}

#[test]
fn comments_and_strings_containing_pipe_text_create_no_occurrence_or_rxjs_chain() {
    let source = r#"
import { of } from 'rxjs';
const text = ".pipe(map(fake))";
// source.pipe(tap(fake));
const value$ = of(1);
"#;
    let shape = extract_rx_shape(source, Fidelity::High).expect("RxJS shape");
    assert!(
        shape.pipes.is_empty(),
        "legacy extraction baseline must ignore fake pipe text"
    );

    let captures = invocation_captures(source);
    assert!(
        captures
            .iter()
            .filter(|capture| capture.name == "invoke.root")
            .all(|capture| !capture.raw_text.contains(".pipe(")),
        "parser-proven occurrences must ignore string/comment lookalikes"
    );
}

#[test]
fn phase5_structural_pipe_accepts_written_whitespace_and_comments() {
    let source = "import { map, tap } from 'rxjs/operators';\nconst value$ = source . pipe /* boundary */ (map(x => ({ x, y: [1, 2] })), tap(x => log(x)));";
    let shape = extract_rx_shape(source, Fidelity::High).unwrap();
    assert_eq!(shape.pipes.len(), 1);
    assert_eq!(shape.pipes[0].owner, "value$");
    assert_eq!(
        shape.pipes[0]
            .operators
            .iter()
            .map(|o| o.operator_name.as_str())
            .collect::<Vec<_>>(),
        ["map", "tap"]
    );
}

#[test]
fn phase5_source_only_collector_parses_once_and_keeps_legacy_shape() {
    use crate::compression::capture_pipeline::{parse_count, reset_parse_count};
    let source = "import { map, tap } from 'rxjs/operators';\nconst value$ = source.pipe(map(x => x), tap(log));";
    reset_parse_count();
    let shape = extract_rx_shape(source, Fidelity::High).unwrap();
    assert_eq!(
        parse_count(),
        1,
        "standalone API must use the shared structural collector"
    );
    assert_eq!(shape.pipes[0].owner, "value$");
    assert_eq!(shape.pipes[0].operators.len(), 2);
}

#[test]
fn phase5_production_meta_reuses_compiler_parse_for_structural_pipe() {
    use crate::compression::capture_pipeline::{parse_count, reset_parse_count};
    use crate::ir::{IRCompiler, opcodes::CoreOp};
    let source = "import { map, tap } from 'rxjs/operators';\nclass C { value$ = source . pipe (map(x => x), tap(log)); }";
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(
        crate::ir::layers::typescript::TypeScriptLayer::new(),
    ));
    reset_parse_count();
    let ir = compiler
        .compile(
            source,
            "Example.ts",
            crate::compression::language::safe_typescript_language().unwrap(),
            TS_QUERY,
            Fidelity::High,
            None,
        )
        .unwrap();
    assert_eq!(parse_count(), 1, "production meta must not reparse source");
    assert!(
        ir.instructions
            .iter()
            .any(|op| matches!(op, CoreOp::TypeAlias(name, _) if name == "@pipeRx")),
        "structural pipe must reach production markers"
    );
}

#[test]
fn audit_invocation_unicode_operator_summary_keeps_utf8_boundaries() {
    let prefix = format!("x => \"{}", "a".repeat(30));
    assert_eq!(prefix.len(), 36);
    let source = format!(
        "import {{ map, tap }} from 'rxjs/operators';\nconst value$ = source.pipe(map({prefix}élong-text\"), tap(log));"
    );
    let shape = extract_rx_shape(&source, Fidelity::High).unwrap();
    assert_eq!(
        shape.pipes[0].operators[0].arg_summary.as_deref(),
        Some(format!("{prefix}...").as_str())
    );
}

#[test]
fn audit_invocation_high_pipe_operator_details_reach_canonical_ir() {
    use crate::ir::{IRCompiler, opcodes::CoreOp};
    let source = "import { map, switchMap, tap } from 'rxjs/operators'; class C { value$ = source.pipe(map(x => x), switchMap(x => load(x)), tap(log)); }";
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(
        crate::ir::layers::typescript::TypeScriptLayer::new(),
    ));
    let ir = compiler
        .compile(
            source,
            "Example.ts",
            crate::compression::language::safe_typescript_language().unwrap(),
            TS_QUERY,
            Fidelity::High,
            None,
        )
        .unwrap();
    let marker = ir
        .instructions
        .iter()
        .find_map(|op| match op {
            CoreOp::TypeAlias(name, value) if name == "@pipeRx" => Some(value),
            _ => None,
        })
        .unwrap();
    assert!(
        marker.contains("map:x => x"),
        "High operator arguments must survive the production handoff"
    );
    assert!(marker.contains("switchMap:x => load(x"));
    assert!(marker.contains("tap:log"));
    assert!(marker.ends_with(')'));
    assert!(marker.find("map:").unwrap() < marker.find("switchMap:").unwrap());
}
