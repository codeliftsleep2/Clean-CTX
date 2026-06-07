use super::*;

#[test]
fn strip_modifiers_handles_single_prefix() {
    assert_eq!(strip_modifiers("public class Foo", MODIFIERS_CLASS), "class Foo");
}

#[test]
fn strip_modifiers_loops_until_stable() {
    // The headline regression: F-07's pre-fix code only made
    // one pass and returned "static abstract class Foo" here.
    let out = strip_modifiers("public static abstract class Foo", MODIFIERS_CLASS);
    assert_eq!(out, "class Foo");
}

#[test]
fn strip_modifiers_handles_export_default() {
    // "export default " must be tried before "export " (otherwise
    // the bare "export " prefix would match the first 7 chars
    // and leave "default class Bar" behind).
    let out = strip_modifiers("export default abstract class Bar", MODIFIERS_CLASS);
    assert_eq!(out, "class Bar");
}

#[test]
fn strip_modifiers_returns_input_when_no_match() {
    assert_eq!(strip_modifiers("class Foo", MODIFIERS_CLASS), "class Foo");
}

#[test]
fn strip_modifiers_handles_low_fidelity_method() {
    // Verify the helper works for the existing Low-fidelity
    // method-compaction case too.
    let out = strip_modifiers("public async getUserById(id: string)", MODIFIERS_LOW);
    assert_eq!(out, "getUserById(id: string)");
}