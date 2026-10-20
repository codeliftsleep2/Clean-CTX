use super::{
    SourceSpan, bind_direct_named_object_member_import, bind_direct_named_value_import,
    bind_direct_namespace_value_import,
};

fn occurrence(source: &str, needle: &str) -> SourceSpan {
    let start = source.rfind(needle).expect("reference occurrence");
    SourceSpan::new(start, start + needle.len()).expect("ordered span")
}
#[test]
fn exact_direct_named_value_import_binds_without_framework_classification() {
    let source = "import { saveThing } from './actions';\nuseValue(saveThing);";
    let binding = bind_direct_named_value_import(
        "/fixture/consumer.ts",
        source,
        occurrence(source, "saveThing"),
    )
    .expect("direct named import binding");
    assert_eq!(binding.imported_name, "saveThing");
    assert_eq!(binding.local_name, "saveThing");
    assert_eq!(binding.module_specifier, "./actions");
}

#[test]
fn shadow_namespace_type_only_and_nonrelative_forms_fail_closed() {
    for source in [
        "import { saveThing } from './actions'; function f(saveThing: any) { useValue(saveThing); }",
        "import * as Actions from './actions'; useValue(Actions.saveThing);",
        "import type { saveThing } from './actions'; useValue(saveThing);",
        "import { saveThing } from '@app/actions'; useValue(saveThing);",
    ] {
        let needle = if source.contains("useValue(save)") {
            "save"
        } else {
            "saveThing"
        };
        assert!(
            bind_direct_named_value_import(
                "/fixture/consumer.ts",
                source,
                occurrence(source, needle),
            )
            .is_none(),
            "{source}"
        );
    }
}

#[test]
fn named_import_alias_resolves_to_exported_object_identity() {
    let source =
        "import { ACTIONS as FeatureActions } from './actions'; useValue(FeatureActions.SET_X);";
    let binding = bind_direct_named_object_member_import(
        "/fixture/consumer.ts",
        source,
        occurrence(source, "FeatureActions.SET_X"),
    )
    .expect("aliased object member binding");
    assert_eq!(binding.imported_object, "ACTIONS");
    assert_eq!(binding.local_object, "FeatureActions");
    assert_eq!(binding.member_name, "SET_X");
    assert_eq!(binding.module_specifier, "./actions");
}

#[test]
fn exact_namespace_member_binds_without_framework_classification() {
    let source = "import * as Actions from './actions';\nuseValue(Actions.saveThing);";
    let binding = bind_direct_namespace_value_import(
        "/fixture/consumer.ts",
        source,
        occurrence(source, "Actions.saveThing"),
    )
    .expect("direct namespace member binding");
    assert_eq!(binding.namespace_name, "Actions");
    assert_eq!(binding.imported_name, "saveThing");
    assert_eq!(binding.module_specifier, "./actions");
}

#[test]
fn namespace_shadow_dynamic_access_type_only_and_nonrelative_forms_fail_closed() {
    for (source, needle) in [
        (
            "import * as Actions from './actions'; function f(Actions: any) { useValue(Actions.saveThing); }",
            "Actions.saveThing",
        ),
        (
            "import * as Actions from './actions'; useValue(Actions['saveThing']);",
            "Actions['saveThing']",
        ),
        (
            "import type * as Actions from './actions'; useValue(Actions.saveThing);",
            "Actions.saveThing",
        ),
        (
            "import * as Actions from '@app/actions'; useValue(Actions.saveThing);",
            "Actions.saveThing",
        ),
    ] {
        assert!(
            bind_direct_namespace_value_import(
                "/fixture/consumer.ts",
                source,
                occurrence(source, needle),
            )
            .is_none(),
            "{source}"
        );
    }
}
