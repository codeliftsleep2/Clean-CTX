//! Phase 0 RED contract for C# invocation-occurrence evidence.

#![cfg(feature = "csharp")]

use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{CapturedNode, run_capture_pipeline_nodes};
use crate::queries::{CS_CALL_QUERY, CS_QUERY};

fn captures(source: &str) -> Vec<CapturedNode> {
    let language =
        crate::compression::language::safe_csharp_language().expect("csharp grammar enabled");
    let query = format!("{CS_QUERY}\n{CS_CALL_QUERY}");
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
fn member_generic_and_named_arguments_preserve_written_structure() {
    let source = "class C { void M() { service.Save<User>(user: dto); } }";
    let captures = captures(source);

    assert_eq!(
        unique_raw(&captures, "invoke.root"),
        ["service.Save<User>(user: dto)"]
    );
    assert_eq!(raw(&captures, "invoke.receiver"), ["service"]);
    assert_eq!(unique_raw(&captures, "invoke.terminal"), ["Save"]);
    assert_eq!(raw(&captures, "invoke.argument"), ["user: dto"]);
    assert_eq!(raw(&captures, "invoke.argument_name"), ["user"]);
    assert_eq!(raw(&captures, "invoke.type_argument"), ["User"]);
}

#[test]
fn nested_calls_expose_contained_roots_and_argument_order() {
    let source = "class C { void M() { Foo(Bar(Baz()), new Pair(1, 2)); } }";
    let captures = captures(source);
    let roots = unique_raw(&captures, "invoke.root");

    assert!(roots.contains(&"Foo(Bar(Baz()), new Pair(1, 2))"));
    assert!(roots.contains(&"Bar(Baz())"));
    assert!(roots.contains(&"Baz()"));
    assert_eq!(
        raw(&captures, "invoke.argument")
            .into_iter()
            .filter(|argument| *argument == "Bar(Baz())" || *argument == "new Pair(1, 2)")
            .collect::<Vec<_>>(),
        ["Bar(Baz())", "new Pair(1, 2)"]
    );
}

#[test]
fn initializer_and_unsupported_targets_do_not_invent_semantic_ownership() {
    let source = "class C { object Value = Factory.Create(); void M() { handlers[name](x); } }";
    let captures = captures(source);

    assert!(
        raw(&captures, "invoke.declaration")
            .iter()
            .any(|text| text.contains("Value"))
    );
    assert!(raw(&captures, "invoke.declaration_name").contains(&"Value"));
    assert!(unique_raw(&captures, "invoke.root").contains(&"handlers[name](x)"));
    assert!(!unique_raw(&captures, "invoke.terminal").contains(&"name"));
}

fn compiled(source: &str) -> crate::ir::invocation_evidence::InvocationEvidenceSet {
    let mut compiler = crate::ir::compiler::IRCompiler::new();
    compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
    compiler
        .compile(
            source,
            "Example.cs",
            crate::compression::language::safe_csharp_language().unwrap(),
            CS_QUERY,
            Fidelity::High,
            None,
        )
        .unwrap();
    compiler.invocation_evidence.take().unwrap()
}

#[test]
fn phase3_production_named_generic_member_and_free_calls() {
    use crate::ir::invocation_evidence::{ArgumentQualifier, WrittenCallee};
    let source = "class C { void M() { service.Save<User, Pair<int, string>>(user: dto, count: 2); Load<User>(); } }";
    let set = compiled(source);
    let view = set.view("Example.cs", source).unwrap();
    let calls = view.occurrences();
    assert_eq!(calls.len(), 2);
    let WrittenCallee::Member { receiver, terminal } = calls[0].callee() else {
        panic!("member")
    };
    assert_eq!(view.slice(*receiver).unwrap(), "service");
    assert_eq!(view.slice(*terminal).unwrap(), "Save");
    assert_eq!(
        calls[0]
            .type_arguments()
            .iter()
            .map(|s| view.slice(*s).unwrap())
            .collect::<Vec<_>>(),
        ["User", "Pair<int, string>"]
    );
    for (arg, (text, name)) in calls[0]
        .arguments()
        .iter()
        .zip([("user: dto", "user"), ("count: 2", "count")])
    {
        assert_eq!(view.slice(arg.span()).unwrap(), text);
        let ArgumentQualifier::Named { name: span } = arg.qualifier() else {
            panic!("named")
        };
        assert_eq!(view.slice(*span).unwrap(), name);
    }
    assert!(matches!(calls[1].callee(), WrittenCallee::Free { .. }));
    assert_eq!(calls[1].type_arguments().len(), 1);
}

#[test]
fn phase3_production_initializer_contexts_and_one_parse() {
    use crate::compression::capture_pipeline::{parse_count, reset_parse_count};
    use crate::ir::invocation_evidence::DeclarationContextKind as Kind;
    let source = "class C { object café = Factory.Create(); object Value { get; } = Build(); void M() { var item = Load(); Save(item); } }";
    reset_parse_count();
    let set = compiled(source);
    assert_eq!(parse_count(), 1);
    let view = set.view("Example.cs", source).unwrap();
    assert_eq!(view.occurrences().len(), 4);
    for (call, (kind, name)) in view.occurrences().iter().zip([
        (Kind::FieldInitializer, "café"),
        (Kind::PropertyInitializer, "Value"),
        (Kind::VariableInitializer, "item"),
        (Kind::Callable, "M"),
    ]) {
        let context = call.declaration_context().unwrap();
        assert_eq!(context.kind(), kind);
        assert_eq!(view.slice(context.written_name().unwrap()).unwrap(), name);
    }
}

#[test]
fn phase3_production_nested_receiver_named_argument_and_dynamic_target() {
    use crate::ir::invocation_evidence::{ParentSite, WrittenCallee};
    let source =
        "class C { void M() { Factory.Create().Save(value: Load()); handlers[name](x); } }";
    let set = compiled(source);
    let view = set.view("Example.cs", source).unwrap();
    let calls = view.occurrences();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[1].parent().unwrap().invocation(), calls[0].span());
    assert_eq!(calls[1].parent().unwrap().site(), ParentSite::Receiver);
    assert_eq!(
        calls[2].parent().unwrap().site(),
        ParentSite::Argument { index: 0 }
    );
    let WrittenCallee::Unsupported {
        function: Some(span),
    } = calls[3].callee()
    else {
        panic!("unsupported")
    };
    assert_eq!(view.slice(*span).unwrap(), "handlers[name]");
}

#[test]
fn phase3_production_repeated_calls_and_rich_arguments_ignore_decoys() {
    use crate::ir::invocation_evidence::ArgumentQualifier;
    let source = "class C { void M() { var text = \"Fake()\"; /* Nope() */ Foo(); Foo(); Bar(new Pair(1, 2), new[] { 1, 2 }, x => Baz(x), ref item); } }";
    let set = compiled(source);
    let view = set.view("Example.cs", source).unwrap();
    let calls = view.occurrences();
    assert_eq!(calls.len(), 4);
    assert_ne!(calls[0].span(), calls[1].span());
    assert_eq!(
        calls[2]
            .arguments()
            .iter()
            .map(|a| view.slice(a.span()).unwrap())
            .collect::<Vec<_>>(),
        [
            "new Pair(1, 2)",
            "new[] { 1, 2 }",
            "x => Baz(x)",
            "ref item"
        ]
    );
    assert!(
        calls[2]
            .arguments()
            .iter()
            .all(|a| matches!(a.qualifier(), ArgumentQualifier::Plain))
    );
    assert_eq!(calls[3].parent().unwrap().invocation(), calls[2].span());
}
