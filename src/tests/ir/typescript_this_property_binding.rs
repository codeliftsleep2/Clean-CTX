use super::*;

fn span_of_nth(source: &str, needle: &str, occurrence: usize) -> SourceSpan {
    let start = source
        .match_indices(needle)
        .nth(occurrence)
        .map(|(start, _)| start)
        .expect("fixture occurrence");
    SourceSpan::new(start, start + needle.len()).unwrap()
}

#[test]
fn arbitrary_this_property_binds_exact_parameter_property_and_type() {
    let source =
        "class Example { constructor(private state: SomeType) {} run() { use(this.state); } }";
    let binding = bind_this_property("member.ts", source, span_of_nth(source, "this.state", 0))
        .expect("structured this.state must bind");

    assert_eq!(binding.source_id(), "member.ts");
    assert_eq!(
        binding.source_hash(),
        crate::cache::content_hash(source.as_bytes())
    );
    assert_eq!(
        binding.occurrence_span(),
        span_of_nth(source, "this.state", 0)
    );
    assert_eq!(binding.property_span(), span_of_nth(source, "state", 1));
    assert_eq!(
        binding.declaration_name_span(),
        span_of_nth(source, "state", 0)
    );
    assert_eq!(
        binding.type_reference_span(),
        Some(span_of_nth(source, "SomeType", 0))
    );
}

#[test]
fn same_property_name_binds_only_inside_its_exact_enclosing_class() {
    let source = "class First { constructor(private state: FirstType) {} run() { use(this.state); } } class Second { constructor(private state: SecondType) {} run() { use(this.state); } }";
    let first = bind_this_property("classes.ts", source, span_of_nth(source, "this.state", 0))
        .expect("First member");
    let second = bind_this_property("classes.ts", source, span_of_nth(source, "this.state", 1))
        .expect("Second member");

    assert_eq!(
        first.declaration_name_span(),
        span_of_nth(source, "state", 0)
    );
    assert_eq!(
        first.type_reference_span(),
        Some(span_of_nth(source, "FirstType", 0))
    );
    assert_eq!(
        second.declaration_name_span(),
        span_of_nth(source, "state", 2)
    );
    assert_eq!(
        second.type_reference_span(),
        Some(span_of_nth(source, "SecondType", 0))
    );
    assert_ne!(first.enclosing_class_span(), second.enclosing_class_span());
}

#[test]
fn parameter_property_modifiers_and_missing_type_are_structural() {
    for modifier in [
        "private",
        "public",
        "protected",
        "readonly",
        "private readonly",
    ] {
        let source = format!(
            "class Example {{ constructor({modifier} client: ClientType) {{}} run() {{ use(this.client); }} }}"
        );
        let binding = bind_this_property(
            "modifiers.ts",
            &source,
            span_of_nth(&source, "this.client", 0),
        )
        .unwrap_or_else(|| panic!("{modifier} parameter-property"));
        assert_eq!(
            binding.type_reference_span(),
            Some(span_of_nth(&source, "ClientType", 0))
        );
    }

    let source = "class Example { constructor(readonly client) {} run() { use(this.client); } }";
    let binding = bind_this_property("untyped.ts", source, span_of_nth(source, "this.client", 0))
        .expect("untyped parameter-property identity");
    assert_eq!(binding.type_reference_span(), None);
}

#[test]
fn lexical_shadows_do_not_hijack_explicit_this_member() {
    for method in [
        "run() { const state = other; use(this.state); }",
        "run(state: OtherType) { use(this.state); }",
    ] {
        let source =
            format!("class Example {{ constructor(private state: StateType) {{}} {method} }}");
        let binding =
            bind_this_property("shadow.ts", &source, span_of_nth(&source, "this.state", 0))
                .expect("explicit this remains member-owned");
        assert_eq!(
            binding.declaration_name_span(),
            span_of_nth(&source, "state", 0)
        );
    }
}

#[test]
fn similar_names_bind_exactly_and_missing_or_ambiguous_members_refuse() {
    let source = "class Example { constructor(private state: A, private state2: B) {} run() { use(this.state2); } }";
    let binding = bind_this_property("similar.ts", source, span_of_nth(source, "this.state2", 0))
        .expect("state2");
    assert_eq!(
        binding.declaration_name_span(),
        span_of_nth(source, "state2", 0)
    );

    for source in [
        "class Example { run() { use(this.missing); } }",
        "class Example { constructor(state: StateType) {} run() { use(this.state); } }",
        "class Example { constructor(private state: A, private state: B) {} run() { use(this.state); } }",
    ] {
        let member = if source.contains("missing") {
            "this.missing"
        } else {
            "this.state"
        };
        assert!(
            bind_this_property("refuse.ts", source, span_of_nth(source, member, 0)).is_none(),
            "{source}"
        );
    }
}

#[test]
fn unsafe_this_contexts_and_other_receiver_forms_refuse() {
    for (source, occurrence) in [
        (
            "class E { constructor(private state: T) {} static run() { use(this.state); } }",
            "this.state",
        ),
        (
            "class E { constructor(private state: T) {} run() { function inner() { return this.state; } } }",
            "this.state",
        ),
        (
            "class E { constructor(private state: T) {} run() { const f = () => this.state; } }",
            "this.state",
        ),
        (
            "class E { constructor(private state: T) {} run() { return obj.state; } }",
            "obj.state",
        ),
        (
            "class E { constructor(private state: T) {} run() { return this?.state; } }",
            "this?.state",
        ),
        (
            "class E { constructor(private state: T) {} run() { return this['state']; } }",
            "this['state']",
        ),
        (
            "class Base { protected state: T; } class E extends Base { run() { return this.state; } }",
            "this.state",
        ),
    ] {
        assert!(
            bind_this_property("contexts.ts", source, span_of_nth(source, occurrence, 0)).is_none(),
            "{source}"
        );
    }
}

#[test]
fn invocation_receiver_span_feeds_the_same_binder() {
    use crate::ir::invocation_evidence::WrittenCallee;

    let source = "class Example { constructor(private repository: Repository) {} run() { return this.repository.read(); } }";
    let evidence = crate::ir::invocation_evidence::collect_typescript_source(source)
        .expect("real TypeScript invocation evidence");
    let view = evidence.view("standalone-typescript", source).unwrap();
    let receiver = view
        .occurrences()
        .iter()
        .find_map(|call| match call.callee() {
            WrittenCallee::Member { receiver, terminal }
                if view.slice(*terminal).ok() == Some("read") =>
            {
                Some(*receiver)
            }
            _ => None,
        })
        .expect("structured receiver span");
    let binding = bind_this_property("standalone-typescript", source, receiver)
        .expect("invocation receiver must bind");
    assert_eq!(
        binding.property_span(),
        span_of_nth(source, "repository", 1)
    );
    assert_eq!(
        binding.declaration_name_span(),
        span_of_nth(source, "repository", 0)
    );
    assert_eq!(
        binding.type_reference_span(),
        Some(span_of_nth(source, "Repository", 0))
    );
}
