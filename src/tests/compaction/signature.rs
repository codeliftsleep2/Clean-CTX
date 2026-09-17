// src/tests/compaction/signature.rs
//
// Structural declaration-identity regressions for the SHARED boundary
// (`compaction::signature`), which method labels, the IR's
// `parse_method_sig`, and the diff key deriver all consume.
//
// Every case below pins a value that whitespace-token inference gets WRONG:
// a generic declaration's type-parameter list may itself contain `, `
// (`Pair<TFirst, TSecond>` → the old rule read `TSecond>`), and a tuple return
// type puts a modifier where a name is expected
// (`public static (int a, int b) GetPair(...)` → the old rule read `static`).

use super::*;
use crate::compaction::method::{extract_method_sig, find_method_params};
use crate::compression::Fidelity;

/// The C# extension-method declaration from the reported defect (Case A):
/// two method type parameters, one of them used by a nested generic parameter
/// type whose own argument list contains a comma.
const CS_EXTENSION_TWO_TYPE_PARAMS: &str = concat!(
    "public static IOrderedQueryable<TFirst> Pair<TFirst, TSecond>(\n",
    "    this IQueryable<TFirst> source,\n",
    "    Expression<Func<TFirst, TSecond>> keySelector,\n",
    "    ListSortDirection direction)"
);

/// The C# tuple-returning declaration from the reported defect (Case B).
const CS_NAMED_TUPLE_RETURN: &str = "public static (int alpha, int beta) GetPair(int[] values)";

// ── Structural head extraction ─────────────────────────────────────

#[test]
fn csharp_two_type_params_name_is_the_declared_identifier() {
    let sig = CS_EXTENSION_TWO_TYPE_PARAMS;
    let (open, _) = find_method_params(sig).expect("the declaration's parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(
        parts.name, "Pair<TFirst, TSecond>",
        "a generic name keeps every type parameter"
    );
    assert_eq!(parts.bare_name, "Pair");
    assert_eq!(
        parts.prefix.trim(),
        "public static IOrderedQueryable<TFirst>"
    );
    assert_ne!(
        parts.name, "TSecond>",
        "token-position inference is the defect"
    );
    assert_ne!(parts.bare_name, "TSecond>");
}

#[test]
fn csharp_three_type_params_are_structural_not_hard_coded_for_two() {
    let sig = "public static Mapper<A> Build<A, B, C>(A a, B b, C c)";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "Build<A, B, C>");
    assert_eq!(parts.bare_name, "Build");
    assert_eq!(parts.prefix.trim(), "public static Mapper<A>");
}

#[test]
fn csharp_ordinary_static_generic_method_keeps_its_identity() {
    let sig = "public static TResult Pair<TFirst, TSecond>(TFirst first, TSecond second)";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "Pair<TFirst, TSecond>");
    assert_eq!(parts.bare_name, "Pair");
    assert_eq!(parts.prefix.trim(), "public static TResult");
}

#[test]
fn csharp_instance_generic_method_keeps_its_identity() {
    let sig = "public Task<TValue> Load<TKey, TValue>(TKey key)";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "Load<TKey, TValue>");
    assert_eq!(parts.bare_name, "Load");
    assert_eq!(parts.prefix.trim(), "public Task<TValue>");
}

#[test]
fn typescript_function_with_two_type_params_keeps_its_identity() {
    let sig = "function pair<A, B>(a: A, b: B): [A, B]";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "pair<A, B>");
    assert_eq!(parts.bare_name, "pair");
    assert_eq!(parts.prefix.trim(), "function");
}

#[test]
fn typescript_method_with_two_type_params_keeps_its_identity() {
    let sig = "pair<A, B>(a: A, b: B): [A, B]";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "pair<A, B>");
    assert_eq!(parts.bare_name, "pair");
    assert!(parts.prefix.trim().is_empty());
}

#[test]
fn rust_fn_with_two_type_params_keeps_its_identity() {
    let sig = "fn pair<A, B>(a: A, b: B) -> (i32, i32)";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "pair<A, B>");
    assert_eq!(parts.bare_name, "pair");
    assert_eq!(parts.prefix.trim(), "fn");
}

/// Java's method type-parameter list PRECEDES the return type, so it belongs to
/// the prefix: the declared name is the identifier that owns the parameter
/// list, nothing else.
#[test]
fn java_leading_type_parameters_stay_in_the_prefix() {
    let sig = "<A, B> Result<A> pair(A a, B b)";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "pair");
    assert_eq!(parts.bare_name, "pair");
    assert_eq!(parts.prefix.trim(), "<A, B> Result<A>");
}

#[test]
fn explicit_interface_implementation_is_one_declared_name() {
    let sig = "public int IFoo.Bar(int x)";
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.bare_name, "IFoo.Bar");
    assert_eq!(parts.prefix.trim(), "public int");
}

#[test]
fn head_parts_are_none_when_no_identifier_precedes_the_group() {
    assert!(split_head_parts("(int x)", 0).is_none());
    assert!(split_head_parts("", 0).is_none());
}

// ── Parenthesized return types (Case B) ────────────────────────────

/// The reported field shift: the tuple group is name-anchored (its `(` follows
/// `static`) but it is a RETURN TYPE, so the declaration's own parameter list
/// must be selected instead — otherwise name/params/return type all shift.
#[test]
fn tuple_return_group_is_not_selected_as_the_parameter_list() {
    let sig = CS_NAMED_TUPLE_RETURN;
    let (open, close) = find_method_params(sig).expect("parameter list");
    assert_eq!(
        &sig[open..=close],
        "(int[] values)",
        "the tuple return type must never be selected as the parameter list"
    );
}

#[test]
fn named_tuple_return_fields_are_structural() {
    let sig = CS_NAMED_TUPLE_RETURN;
    let (open, _) = find_method_params(sig).expect("parameter list");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "GetPair");
    assert_eq!(parts.bare_name, "GetPair");
    assert_eq!(parts.prefix.trim(), "public static (int alpha, int beta)");
    assert_eq!(
        return_type_from_prefix(parts.prefix),
        Some("(int alpha, int beta)"),
        "the return type is the tuple, and the modifiers stay out of it"
    );
    assert_ne!(parts.name, "static");
}

/// Tuple ELEMENT NAMES are not part of the trigger: an unnamed tuple selects
/// the same structural fields.
#[test]
fn unnamed_tuple_return_fields_are_structural() {
    let sig = "public (int, int) GetPair(int[] values)";
    let (open, close) = find_method_params(sig).expect("parameter list");
    assert_eq!(&sig[open..=close], "(int[] values)");
    let parts = split_head_parts(sig, open).expect("head parts");

    assert_eq!(parts.name, "GetPair");
    assert_eq!(parts.prefix.trim(), "public (int, int)");
    assert_eq!(return_type_from_prefix(parts.prefix), Some("(int, int)"));
}

/// Nonregression: a `where T : new()` constraint group follows the parameter
/// list and must never be selected instead of it.
#[test]
fn constraint_new_group_is_still_not_the_parameter_list() {
    let sig = "void M(int id) where T : new()";
    let (open, close) = find_method_params(sig).expect("parameter list");
    assert_eq!(&sig[open..=close], "(int id)");
}

/// Nonregression: a constructor initializer is a call site that FOLLOWS the
/// parameter list and must never be selected instead of it.
#[test]
fn base_initializer_group_is_still_not_the_parameter_list() {
    let sig = "Greeter(string prefix) : base(prefix)";
    let (open, close) = find_method_params(sig).expect("parameter list");
    assert_eq!(&sig[open..=close], "(string prefix)");
}

// ── Return-type classification and extraction ──────────────────────

#[test]
fn return_type_first_classification_covers_the_name_first_languages() {
    // Return-type-first declarations.
    assert!(is_return_type_first("public IActionResult "));
    assert!(is_return_type_first("public static async Task "));
    assert!(is_return_type_first("public static (int alpha, int beta) "));
    assert!(is_return_type_first("void "));
    // Name-first declarations: the leftover prefix token is a declarator or a
    // modifier, never a type.
    assert!(!is_return_type_first("function "));
    assert!(!is_return_type_first("async function "));
    assert!(!is_return_type_first("fn "));
    assert!(!is_return_type_first("async "));
    assert!(!is_return_type_first(""));
}

#[test]
fn return_type_expression_excludes_modifiers_type_params_and_arrays() {
    assert_eq!(
        return_type_from_prefix("public static IOrderedQueryable<TFirst> "),
        Some("IOrderedQueryable<TFirst>")
    );
    assert_eq!(
        return_type_from_prefix("internal static async Task<(A section, Term term)> "),
        Some("Task<(A section, Term term)>"),
        "a nested comma inside the type must not leak modifiers in"
    );
    assert_eq!(
        return_type_from_prefix("public static (int alpha, int beta) "),
        Some("(int alpha, int beta)")
    );
    assert_eq!(return_type_from_prefix("public int[] "), Some("int[]"));
    assert_eq!(
        return_type_from_prefix("public ILookup<K, V>[] "),
        Some("ILookup<K, V>[]")
    );
    assert_eq!(
        return_type_from_prefix("public (int, int)[] "),
        Some("(int, int)[]")
    );
    assert_eq!(return_type_from_prefix("public string? "), Some("string?"));
    // Name-first declarations declare no return type here.
    assert_eq!(return_type_from_prefix("function "), None);
    assert_eq!(return_type_from_prefix("fn "), None);
    assert_eq!(return_type_from_prefix(""), None);
    // A declaration's OWN type-parameter list is not a return type.
    assert_eq!(return_type_from_prefix("<A, B> Result<A> "), None);
}

// ── Formal parameters ──────────────────────────────────────────────

#[test]
fn split_parameters_ignores_nested_commas() {
    assert_eq!(split_parameters("int[] values"), vec!["int[] values"]);
    assert_eq!(
        split_parameters(
            "this IQueryable<TFirst> source, \
             Expression<Func<TFirst, TSecond>> keySelector, \
             ListSortDirection direction"
        ),
        vec![
            "this IQueryable<TFirst> source",
            "Expression<Func<TFirst, TSecond>> keySelector",
            "ListSortDirection direction",
        ],
        "a comma inside a generic argument list is not a separator"
    );
    assert_eq!(
        split_parameters("cb: (x: A, y: B) => void, other: int"),
        vec!["cb: (x: A, y: B) => void", "other: int"]
    );
    assert_eq!(
        split_parameters("s: string = \"a,b\", n: int"),
        vec!["s: string = \"a,b\"", "n: int"],
        "a comma inside a literal default is not a separator"
    );
    assert!(split_parameters("").is_empty());
}

// ── Compaction labels for the affected shapes ─────────────────────
//
// `extract_method_sig` produces the compressed labels AND feeds the IR
// compiler (CoreIRPass calls it per `method.root` capture), so a wrong label
// reaches both the rendered compaction/diff output and the IR identity.

#[test]
fn low_label_keeps_generic_identity_and_parameter_count() {
    let out = extract_method_sig(CS_EXTENSION_TWO_TYPE_PARAMS, Fidelity::Low);
    assert_eq!(
        out, "Pair(source,keySelector,direction)",
        "the name is the declared identifier, and the three written parameters stay three"
    );
    assert!(!out.starts_with("TSecond>"));
}

#[test]
fn medium_label_keeps_generic_identity_and_parameter_count() {
    let out = extract_method_sig(CS_EXTENSION_TWO_TYPE_PARAMS, Fidelity::Medium);
    assert!(out.starts_with("Pair<TFirst, TSecond>("), "got: {out}");
    assert!(out.contains("keySelector"), "got: {out}");
    assert!(
        out.contains("Expression<Func<TFirst, TSecond>>"),
        "got: {out}"
    );
    assert!(!out.starts_with("TSecond>"), "got: {out}");
}

#[test]
fn medium_label_tuple_return_is_name_first_without_tuple_members() {
    assert_eq!(
        extract_method_sig(CS_NAMED_TUPLE_RETURN, Fidelity::Medium),
        "GetPair(int[] values)"
    );
}

#[test]
fn low_label_tuple_return_keeps_the_parameter_name() {
    assert_eq!(
        extract_method_sig(CS_NAMED_TUPLE_RETURN, Fidelity::Low),
        "GetPair(values)"
    );
}

#[test]
fn low_label_typescript_two_type_params_keeps_the_identifier() {
    assert_eq!(
        extract_method_sig("function pair<A, B>(a: A, b: B): [A, B]", Fidelity::Low),
        "pair(a,b)"
    );
}

#[test]
fn medium_label_typescript_two_type_params_keeps_the_identifier() {
    let out = extract_method_sig("function pair<A, B>(a: A, b: B): [A, B]", Fidelity::Medium);
    assert!(out.starts_with("function pair<A,B>("), "got: {out}");
}

#[test]
fn low_label_rust_two_type_params_keeps_the_identifier() {
    assert_eq!(
        extract_method_sig("fn pair<A, B>(a: A, b: B) -> (i32, i32)", Fidelity::Low),
        "pair(a,b)"
    );
}

#[test]
fn medium_label_rust_two_type_params_keeps_the_identifier() {
    let out = extract_method_sig("fn pair<A, B>(a: A, b: B) -> (i32, i32)", Fidelity::Medium);
    assert!(out.starts_with("fn pair<A,B>("), "got: {out}");
}
