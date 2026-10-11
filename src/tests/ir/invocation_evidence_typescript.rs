//! Phase 0 RED contract for TypeScript invocation-occurrence evidence.
//!
//! These tests use the existing single-parse capture boundary. They do not
//! name the future evidence-set type, so RED is a runtime assertion failure.

#![cfg(feature = "typescript")]

use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{CapturedNode, run_capture_pipeline_nodes};
use crate::queries::{TS_CALL_QUERY, TS_QUERY};

fn captures(source: &str) -> Vec<CapturedNode> {
    let language = crate::compression::language::safe_typescript_language()
        .expect("typescript grammar enabled");
    let query = format!("{TS_QUERY}\n{TS_CALL_QUERY}");
    run_capture_pipeline_nodes(language, &query, source, Fidelity::High, |_, raw, _| {
        Some(raw.to_string())
    })
    .expect("the production capture query must compile")
}

fn raw<'a>(captures: &'a [CapturedNode], name: &str) -> Vec<&'a str> {
    captures
        .iter()
        .filter(|capture| capture.name == name)
        .map(|capture| capture.raw_text.as_str())
        .collect()
}

fn unique_raw<'a>(captures: &'a [CapturedNode], name: &str) -> Vec<&'a str> {
    let mut spans = std::collections::BTreeSet::new();
    captures
        .iter()
        .filter(|capture| capture.name == name)
        .filter(|capture| spans.insert((capture.start_byte, capture.end_byte)))
        .map(|capture| capture.raw_text.as_str())
        .collect()
}

#[test]
fn member_calls_preserve_root_receiver_terminal_arguments_and_utf8_byte_spans() {
    let source = "const café = this.userService.save(user);";
    let captures = captures(source);

    assert_eq!(unique_raw(&captures, "invoke.root"), ["this.userService.save(user)"]);
    assert_eq!(raw(&captures, "invoke.receiver"), ["this.userService"]);
    assert_eq!(unique_raw(&captures, "invoke.terminal"), ["save"]);
    assert_eq!(raw(&captures, "invoke.argument"), ["user"]);

    let root = captures
        .iter()
        .find(|capture| capture.name == "invoke.root")
        .expect("invocation root");
    assert_eq!(
        &source.as_bytes()[root.start_byte..root.end_byte],
        root.raw_text.as_bytes()
    );
}

#[test]
fn nested_and_chained_calls_expose_distinct_contained_roots_in_source_order() {
    let source = "foo(bar(baz())); this.userService.getAll().pipe(map(x => x.id));";
    let captures = captures(source);
    let roots = unique_raw(&captures, "invoke.root");

    assert_eq!(roots.len(), 6, "every written invocation must have one root");
    assert!(roots.contains(&"foo(bar(baz()))"));
    assert!(roots.contains(&"bar(baz())"));
    assert!(roots.contains(&"baz()"));
    assert!(roots.contains(&"this.userService.getAll().pipe(map(x => x.id))"));
    assert!(roots.contains(&"this.userService.getAll()"));
    assert!(roots.contains(&"map(x => x.id)"));
}

#[test]
fn generic_spread_initializer_and_deferred_contexts_remain_structural() {
    let source = r#"
const result = http.post<User>(url, ...items);
async function save() { await service.save(user); return service.load(); }
"#;
    let captures = captures(source);

    assert!(raw(&captures, "invoke.type_argument").contains(&"User"));
    assert!(raw(&captures, "invoke.spread").contains(&"...items"));
    assert!(
        raw(&captures, "invoke.declaration")
            .iter()
            .any(|text| text.contains("result"))
    );
    assert!(raw(&captures, "invoke.declaration_name").contains(&"result"));
    assert_eq!(raw(&captures, "invoke.await_context"), Vec::<&str>::new());
    assert_eq!(raw(&captures, "invoke.return_context"), Vec::<&str>::new());
}

#[test]
fn repeated_and_dynamic_calls_retain_occurrences_without_fabricated_terminals() {
    let captures = captures("foo(); foo(); handlers[name](x);");
    let roots = unique_raw(&captures, "invoke.root");
    let terminals = unique_raw(&captures, "invoke.terminal");

    assert_eq!(roots, ["foo()", "foo()", "handlers[name](x)"]);
    assert_eq!(terminals, ["foo", "foo"]);
}


#[test]
fn canonical_call_and_semantic_evidence_remain_unchanged() {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::layers::typescript::TypeScriptLayer;
    use crate::ir::patterns::CompressingPatternRecognizer;
    use crate::layers::meta::semantic::SemanticRelation;

    let language = crate::compression::language::safe_typescript_language()
        .expect("typescript grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    let ir = compiler
        .compile(
            "class Example { run(): void { save(item); } }",
            "Example.ts",
            language,
            TS_QUERY,
            Fidelity::Medium,
            None,
        )
        .expect("production compilation must succeed");

    let call = ir
        .instructions
        .iter()
        .find_map(|op| op.call_parts())
        .expect("CoreOp::Call must remain present");
    assert_eq!(call.1, "save");
    assert_eq!(call.2, 1);
    assert!(!call.3);

    let edges =
        crate::ir::semantic_projection::project_generic_facts(&ir.instructions, "Example.ts");
    let edge = edges
        .iter()
        .find(|edge| edge.relation == SemanticRelation::Calls)
        .expect("Calls projection must remain present");
    let evidence = edge.call_evidence.expect("CallEvidence must remain present");
    assert_eq!(evidence.explicit_arg_count, 1);
    assert!(!evidence.has_spread);
}
