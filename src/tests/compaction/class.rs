use super::*;

/// F-07 regression: multi-modifier class declarations were being
/// returned with leftover modifiers because the previous
/// implementation made only one pass over the modifier list.
#[test]
fn extract_class_name_strips_multiple_modifiers() {
    assert_eq!(
        extract_class_name("public static abstract class Foo"),
        "Foo"
    );
}

#[test]
fn extract_class_name_strips_export_default() {
    assert_eq!(
        extract_class_name("export default abstract class Bar"),
        "Bar"
    );
}

#[test]
fn extract_class_name_handles_extends() {
    assert_eq!(
        extract_class_name("public class FooService extends BaseService"),
        "FooService:BaseService"
    );
}

#[test]
fn extract_class_name_handles_extends_and_implements() {
    assert_eq!(
        extract_class_name("class FooService extends BaseService implements IFoo"),
        "FooService:BaseService,IFoo"
    );
}

#[test]
fn extract_class_name_strips_generic_parameters() {
    assert_eq!(extract_class_name("class Foo<T>"), "Foo");
}