use super::*;

#[test]
fn strip_modifiers_handles_single_prefix() {
    assert_eq!(
        strip_modifiers("public class Foo", MODIFIERS_CLASS),
        "class Foo"
    );
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

// ── strip_csharp_attributes ───────────────────────────────────────

#[test]
fn strip_csharp_attributes_basic() {
    assert_eq!(
        strip_csharp_attributes("[HttpGet]\npublic IActionResult Get()"),
        "public IActionResult Get()"
    );
}

#[test]
fn strip_csharp_attributes_with_parens() {
    assert_eq!(
        strip_csharp_attributes("[HttpGet(\"{id}\")]\npublic IActionResult GetById(int id)"),
        "public IActionResult GetById(int id)"
    );
}

#[test]
fn strip_csharp_attributes_with_route_brace() {
    // The `{controller}` inside the attribute would otherwise confuse
    // find_body_start — this asserts it is fully stripped.
    assert_eq!(
        strip_csharp_attributes("[Route(\"api/[controller]\")]\npublic class UserController"),
        "public class UserController"
    );
}

#[test]
fn strip_csharp_attributes_multiple_lines() {
    assert_eq!(
        strip_csharp_attributes(
            "[ApiController]\n[Route(\"api/[controller]\")]\npublic class UserController"
        ),
        "public class UserController"
    );
}

#[test]
fn strip_csharp_attributes_guards_ts_index_signature() {
    // A TS index signature starts with `[` but is NOT an attribute.
    // The remainder after `]` is `:` — not an identifier start.
    assert_eq!(
        strip_csharp_attributes("[key: string]: number"),
        "[key: string]: number"
    );
}

#[test]
fn strip_csharp_attributes_no_leading_bracket() {
    assert_eq!(
        strip_csharp_attributes("public class Foo"),
        "public class Foo"
    );
}
