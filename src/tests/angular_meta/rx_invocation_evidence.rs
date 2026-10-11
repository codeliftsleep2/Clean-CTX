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
