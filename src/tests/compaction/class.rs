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

// ── C# attribute handling ─────────────────────────────────────────

#[test]
fn extract_class_name_strips_csharp_attributes() {
    // C# uses `:` for inheritance — `extract_class_name` does not
    // preserve base types (the CSharpLayer emits `X ControllerBase`
    // separately). We only assert the bare name here.
    assert_eq!(
        extract_class_name("[ApiController]\npublic class UserController : ControllerBase"),
        "UserController"
    );
}

#[test]
fn extract_class_name_strips_multiple_csharp_attributes() {
    assert_eq!(
        extract_class_name(
            "[ApiController]\n[Route(\"api/[controller]\")]\npublic class UserController"
        ),
        "UserController"
    );
}

// ── Phase E: Rust impl generic preservation regression tests ───────

#[test]
fn rust_extract_struct_name_basic_struct() {
    assert_eq!(extract_rust_struct_name("pub struct MyStruct"), "MyStruct");
}

#[test]
fn rust_extract_struct_name_enum() {
    assert_eq!(extract_rust_struct_name("pub enum Status"), "Status");
}

#[test]
fn rust_extract_struct_name_trait() {
    assert_eq!(
        extract_rust_struct_name("pub trait Repository"),
        "Repository"
    );
}

#[test]
fn rust_extract_struct_name_inherent_impl() {
    assert_eq!(extract_rust_struct_name("impl MyStruct"), "MyStruct");
}

#[test]
fn rust_extract_struct_name_trait_impl() {
    assert_eq!(
        extract_rust_struct_name("impl Display for MyStruct"),
        "MyStruct:Display"
    );
}

/// Phase E regression: generics must be preserved for trait impls.
#[test]
fn rust_extract_struct_name_generic_trait_impl() {
    assert_eq!(
        extract_rust_struct_name("impl<T> Repository<T> for PostgresRepo"),
        "PostgresRepo:Repository<T>"
    );
}

/// Phase E regression: inherent impl with generics.
#[test]
fn rust_extract_struct_name_inherent_impl_with_generics() {
    assert_eq!(extract_rust_struct_name("impl<T> Cache<T>"), "Cache<T>");
}

/// Phase E regression: complex generics with nested types.
#[test]
fn rust_extract_struct_name_complex_generics() {
    assert_eq!(
        extract_rust_struct_name("impl<T> Repository<T> for Vec<T>"),
        "Vec<T>:Repository<T>"
    );
}

/// Phase E regression: where clause should be stripped.
#[test]
fn rust_extract_struct_name_where_clause() {
    assert_eq!(
        extract_rust_struct_name("pub struct MyStruct<T> where T: Clone"),
        "MyStruct<T>"
    );
}

// ── Non-CBM Tool Audit 2026-08-25, finding #1 ────────────────────────
//
// Ground-truth C# declaration shapes from the `diff_commits` audit. The
// audit's hand-trace claimed these extract correctly, but `MODIFIERS_CLASS`
// had no `internal ` entry, so `strip_modifiers` stopped immediately and
// the first whitespace token became the "name":
//   "internal static class TestDataFactory" → "internal"
// which the diff renderer then emitted as `~ class internal`.
// Also covers the enum/struct keyword strips used by the diff snapshot
// builder.

#[test]
fn extract_class_name_strips_internal_modifier() {
    assert_eq!(
        extract_class_name("internal static class TestDataFactory"),
        "TestDataFactory"
    );
}

#[test]
fn extract_class_name_strips_internal_on_interface() {
    assert_eq!(
        extract_class_name("internal interface IEntityRepository"),
        "IEntityRepository"
    );
}

#[test]
fn extract_class_name_strips_enum_keyword() {
    assert_eq!(
        extract_class_name("public enum PriorityLevel"),
        "PriorityLevel"
    );
    assert_eq!(
        extract_class_name("internal enum StatusFlags"),
        "StatusFlags"
    );
}

#[test]
fn extract_class_name_strips_struct_keyword() {
    // C# structs are captured as `struct.root` and routed through this
    // helper (non-Rust parsers only).
    assert_eq!(extract_class_name("public struct Point"), "Point");
}
