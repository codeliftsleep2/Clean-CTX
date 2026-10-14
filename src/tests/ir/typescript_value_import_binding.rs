use super::{SourceSpan, bind_direct_named_value_import};

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
    assert_eq!(binding.module_specifier, "./actions");
}

#[test]
fn shadow_alias_namespace_type_only_and_nonrelative_forms_fail_closed() {
    for source in [
        "import { saveThing } from './actions'; function f(saveThing: any) { useValue(saveThing); }",
        "import { saveThing as save } from './actions'; useValue(save);",
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
