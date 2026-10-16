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

    assert_eq!(
        unique_raw(&captures, "invoke.root"),
        ["this.userService.save(user)"]
    );
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

    assert_eq!(
        roots.len(),
        6,
        "every written invocation must have one root"
    );
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
    let evidence = edge
        .call_evidence
        .expect("CallEvidence must remain present");
    assert_eq!(evidence.explicit_arg_count, 1);
    assert!(!evidence.has_spread);
}

fn compiled_evidence(
    source: &str,
    fidelity: Fidelity,
) -> crate::ir::invocation_evidence::InvocationEvidenceSet {
    use crate::ir::compiler::IRCompiler;
    use crate::ir::layers::typescript::TypeScriptLayer;
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler
        .compile(
            source,
            "Example.ts",
            crate::compression::language::safe_typescript_language().unwrap(),
            TS_QUERY,
            fidelity,
            None,
        )
        .expect("production compilation succeeds");
    compiler
        .invocation_evidence
        .take()
        .expect("successful compile publishes evidence")
}

#[test]
fn phase2_production_member_generics_spread_and_field_context() {
    use crate::ir::invocation_evidence::{
        ArgumentQualifier, DeclarationContextKind, WrittenCallee,
    };
    let source = "class Example { readonly café = this.service.save<User>(user, ...items); }";
    let set = compiled_evidence(source, Fidelity::High);
    let view = set.view("Example.ts", source).unwrap();
    assert_eq!(view.occurrences().len(), 1);
    let call = &view.occurrences()[0];
    assert_eq!(
        view.slice(call.span()).unwrap(),
        "this.service.save<User>(user, ...items)"
    );
    match call.callee() {
        WrittenCallee::Member { receiver, terminal } => {
            assert_eq!(view.slice(*receiver).unwrap(), "this.service");
            assert_eq!(view.slice(*terminal).unwrap(), "save");
        }
        other => panic!("expected written member evidence: {other:?}"),
    }
    assert_eq!(call.arguments().len(), 2);
    assert_eq!(view.slice(call.arguments()[0].span()).unwrap(), "user");
    assert_eq!(call.arguments()[0].qualifier(), &ArgumentQualifier::Plain);
    assert_eq!(view.slice(call.arguments()[1].span()).unwrap(), "...items");
    assert_eq!(call.arguments()[1].qualifier(), &ArgumentQualifier::Spread);
    assert_eq!(call.type_arguments().len(), 1);
    assert_eq!(view.slice(call.type_arguments()[0]).unwrap(), "User");
    let context = call
        .declaration_context()
        .expect("field initializer context");
    assert_eq!(context.kind(), DeclarationContextKind::FieldInitializer);
    assert_eq!(view.slice(context.written_name().unwrap()).unwrap(), "café");
    assert!(call.parent().is_none());
}

#[test]
fn phase2_production_immediate_argument_and_receiver_parents() {
    use crate::ir::invocation_evidence::{DeclarationContextKind, ParentSite};
    let source = "const result = this.api.load().pipe(map(value => format(value)));";
    let set = compiled_evidence(source, Fidelity::High);
    let view = set.view("Example.ts", source).unwrap();
    let calls = view.occurrences();
    assert_eq!(calls.len(), 4);
    assert_eq!(
        view.slice(calls[0].span()).unwrap(),
        "this.api.load().pipe(map(value => format(value)))"
    );
    assert!(calls[0].parent().is_none());
    for (index, parent_index, site) in [
        (1, 0, ParentSite::Receiver),
        (2, 0, ParentSite::Argument { index: 0 }),
        (3, 2, ParentSite::Argument { index: 0 }),
    ] {
        let parent = calls[index].parent().expect("immediate structural parent");
        assert_eq!(parent.invocation(), calls[parent_index].span());
        assert_eq!(parent.site(), site);
    }
    for call in calls {
        let context = call.declaration_context().unwrap();
        assert_eq!(context.kind(), DeclarationContextKind::VariableInitializer);
        assert_eq!(
            view.slice(context.written_name().unwrap()).unwrap(),
            "result"
        );
    }
}

#[test]
fn phase2_production_callable_context_and_one_parse() {
    use crate::compression::capture_pipeline::{parse_count, reset_parse_count};
    use crate::ir::invocation_evidence::DeclarationContextKind;
    let source = "class Example { async run() { await service.save(user); return service.load(); } refresh = () => service.refresh(); }";
    reset_parse_count();
    let set = compiled_evidence(source, Fidelity::High);
    assert_eq!(
        parse_count(),
        1,
        "the evidence collector must reuse the production parse"
    );
    let view = set.view("Example.ts", source).unwrap();
    assert_eq!(view.occurrences().len(), 3);
    for (call, name) in view.occurrences().iter().zip(["run", "run", "refresh"]) {
        let context = call.declaration_context().unwrap();
        assert_eq!(context.kind(), DeclarationContextKind::Callable);
        assert_eq!(view.slice(context.written_name().unwrap()).unwrap(), name);
    }
}

#[test]
fn phase2_production_dynamic_repeated_and_lexical_decoys() {
    use crate::ir::invocation_evidence::WrittenCallee;
    let source =
        "const café = 'fake()'; /* comment() */ if (a) foo(); if (b) foo(); handlers[name](x);";
    let set = compiled_evidence(source, Fidelity::High);
    let view = set.view("Example.ts", source).unwrap();
    let calls = view.occurrences();
    assert_eq!(calls.len(), 3);
    assert_eq!(view.slice(calls[0].span()).unwrap(), "foo()");
    assert_eq!(view.slice(calls[1].span()).unwrap(), "foo()");
    assert_ne!(calls[0].span(), calls[1].span());
    for call in &calls[..2] {
        assert!(matches!(call.callee(), WrittenCallee::Free { .. }));
        assert!(call.declaration_context().is_none());
    }
    match calls[2].callee() {
        WrittenCallee::Unsupported {
            function: Some(span),
        } => {
            assert_eq!(view.slice(*span).unwrap(), "handlers[name]");
        }
        other => panic!("dynamic callee must not invent a terminal: {other:?}"),
    }
}

#[test]
fn phase2_production_argument_boundaries_are_fidelity_independent() {
    let source = "const result = pipe({ text: 'a,b' }, [1, 2], `c,${value}`, x => nested(x, 2), ...operators);";
    let high = compiled_evidence(source, Fidelity::High);
    let low = compiled_evidence(source, Fidelity::Low);
    assert_eq!(
        high, low,
        "structural evidence must not depend on presentation fidelity"
    );
    let view = high.view("Example.ts", source).unwrap();
    let calls = view.occurrences();
    assert_eq!(calls.len(), 2);
    let args = calls[0]
        .arguments()
        .iter()
        .map(|arg| view.slice(arg.span()).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        args,
        [
            "{ text: 'a,b' }",
            "[1, 2]",
            "`c,${value}`",
            "x => nested(x, 2)",
            "...operators"
        ]
    );
    assert_eq!(
        calls[1].parent().unwrap().site(),
        crate::ir::invocation_evidence::ParentSite::Argument { index: 3 }
    );
}
