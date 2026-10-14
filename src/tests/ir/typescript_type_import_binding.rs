use super::*;

fn span_of_nth(source: &str, needle: &str, occurrence: usize) -> SourceSpan {
    let start = source.match_indices(needle).nth(occurrence).unwrap().0;
    SourceSpan::new(start, start + needle.len()).unwrap()
}

#[test]
fn prereq3a_generic_type_span_binds_exact_named_import() {
    let source = r#"import { Thing } from "some-package";
class Example {
  constructor(private state: Thing<SomeArg>) {}
  run() { return this.state; }
}"#;
    let member = crate::ir::typescript_this_property_binding::bind_this_property(
        "type-import.ts",
        source,
        span_of_nth(source, "this.state", 0),
    )
    .expect("Prereq3a member binding");
    let type_span = member
        .type_reference_span()
        .expect("written type reference");
    let binding = bind_type_reference_to_named_import("type-import.ts", source, type_span)
        .expect("direct named import must bind");

    assert_eq!(binding.source_id(), "type-import.ts");
    assert_eq!(
        binding.source_hash(),
        crate::cache::content_hash(source.as_bytes())
    );
    assert_eq!(
        binding.type_reference_span(),
        span_of_nth(source, "Thing<SomeArg>", 0)
    );
    assert_eq!(binding.type_head_span(), span_of_nth(source, "Thing", 1));
    assert_eq!(
        binding.local_binding_span(),
        span_of_nth(source, "Thing", 0)
    );
    assert_eq!(
        binding.imported_symbol_span(),
        span_of_nth(source, "Thing", 0)
    );
    assert_eq!(binding.imported_symbol_name(), "Thing");
    assert_eq!(
        binding.module_specifier_span(),
        span_of_nth(source, "some-package", 0)
    );
    assert_eq!(binding.module_specifier(), "some-package");
    assert_eq!(
        binding.import_statement_span(),
        span_of_nth(source, "import { Thing } from \"some-package\";", 0)
    );
}

#[test]
fn multiple_imports_select_exact_symbol_and_preserve_foreign_module() {
    let source = "import { Alpha } from 'pkg-a'; import { Beta } from './my-own-thing'; class E { constructor(private value: Beta<Arg>) {} }";
    let binding = bind_type_reference_to_named_import(
        "multiple.ts",
        source,
        span_of_nth(source, "Beta<Arg>", 0),
    )
    .expect("Beta import");
    assert_eq!(binding.imported_symbol_name(), "Beta");
    assert_eq!(binding.module_specifier(), "./my-own-thing");
    assert_eq!(binding.local_binding_span(), span_of_nth(source, "Beta", 0));
}

fn bind_at(source_id: &str, source: &str, written_type: &str) -> Option<TypeImportBinding> {
    let start = source.rmatch_indices(written_type).next().unwrap().0;
    bind_type_reference_to_named_import(
        source_id,
        source,
        SourceSpan::new(start, start + written_type.len()).unwrap(),
    )
}

#[test]
fn direct_and_multi_symbol_imports_select_only_the_exact_type_head() {
    let direct = "import { Thing } from 'pkg'; type Use = Thing;";
    let binding = bind_at("direct.ts", direct, "Thing").expect("direct type identifier");
    assert_eq!(binding.type_reference_span(), binding.type_head_span());
    assert_eq!(binding.module_specifier(), "pkg");

    let multiple = "import { Alpha, Beta, Gamma } from 'pkg'; type Use = Beta<Arg>;";
    let binding = bind_at("symbols.ts", multiple, "Beta<Arg>").expect("Beta only");
    assert_eq!(binding.imported_symbol_name(), "Beta");
    assert_eq!(
        binding.local_binding_span(),
        span_of_nth(multiple, "Beta", 0)
    );
    assert_eq!(binding.module_specifier(), "pkg");
}

#[test]
fn missing_import_and_type_space_collisions_fail_closed() {
    let missing = "type Use = Thing<Arg>;";
    assert!(bind_at("missing.ts", missing, "Thing<Arg>").is_none());

    let collision = "import { Thing } from 'pkg'; type Thing = unknown; type Use = Thing<Arg>;";
    assert!(bind_at("collision.ts", collision, "Thing<Arg>").is_none());

    let duplicate = "import { Thing } from 'a'; import { Thing } from 'b'; type Use = Thing<Arg>;";
    assert!(bind_at("duplicate.ts", duplicate, "Thing<Arg>").is_none());
}

#[test]
fn alias_namespace_default_and_type_only_forms_are_explicitly_refused() {
    for (source, written_type) in [
        (
            "import { Thing as Alias } from 'pkg'; type Use = Alias<Arg>;",
            "Alias<Arg>",
        ),
        (
            "import * as NS from 'pkg'; type Use = NS.Thing<Arg>;",
            "NS.Thing<Arg>",
        ),
        (
            "import Thing from 'pkg'; type Use = Thing<Arg>;",
            "Thing<Arg>",
        ),
        (
            "import type { Thing } from 'pkg'; type Use = Thing<Arg>;",
            "Thing<Arg>",
        ),
        (
            "import { type Thing } from 'pkg'; type Use = Thing<Arg>;",
            "Thing<Arg>",
        ),
    ] {
        assert!(
            bind_at("unsupported.ts", source, written_type).is_none(),
            "{source}"
        );
    }
}

#[test]
fn barrel_provenance_stops_at_written_module_and_sources_are_isolated() {
    let barrel = "import { Thing } from './barrel'; type Use = Thing<Arg>;";
    let binding = bind_at("consumer.ts", barrel, "Thing<Arg>").expect("source-level barrel import");
    assert_eq!(binding.module_specifier(), "./barrel");

    let importing = "import { Thing } from 'pkg'; type Use = Thing<Arg>;";
    let without_import = "type Use = Thing<Arg>;";
    assert!(bind_at("other.ts", without_import, "Thing<Arg>").is_none());
    let first = bind_at("first.ts", importing, "Thing<Arg>").expect("first source");
    let second = bind_at("second.ts", importing, "Thing<Arg>").expect("second source");
    assert_eq!(first.source_hash(), second.source_hash());
    assert_ne!(first.source_id(), second.source_id());
}
