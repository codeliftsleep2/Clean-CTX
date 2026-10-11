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

    assert_eq!(unique_raw(&captures, "invoke.root"), ["service.Save<User>(user: dto)"]);
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
    let source =
        "class C { object Value = Factory.Create(); void M() { handlers[name](x); } }";
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
