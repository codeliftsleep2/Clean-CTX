// src/tests/angular_meta/util.rs
//
// Unit tests for the shared string-scanning helpers used across the
// Angular Ecosystem Deepening sub-layers (RxJS, NgRx, Routing).

use crate::angular_meta::util::{
    collect_call_body, extract_entity_type, is_inside_comment_or_string, split_top_level,
};

// ── collect_call_body: string-awareness (Round-6 audit) ────────────
//
// `(`/`)` inside single-quoted, double-quoted, or backtick template
// literals are literal characters — they must NOT affect bracket depth.
// The old naive scan would close the body early on a string containing a
// `)`, truncating e.g. an effect body.

#[test]
fn collect_call_body_ignores_parens_in_strings() {
    // The body contains a string with a `)` — the scan must NOT treat
    // that as the call's closing paren. `text` is the slice AFTER the
    // opening `(` of the outer call (e.g. after `console.log(`).
    let text = r#"'foo(bar)'), map(x => x)"#;
    let (body, end) = collect_call_body(text);
    assert!(
        body.contains("foo(bar)"),
        "string content should be preserved, got: {}",
        body
    );
    // The end offset must point at the REAL closing paren of the outer
    // call, not the `)` inside the string literal.
    assert!(
        end > body.find("foo(bar)").unwrap_or(0),
        "end offset should be past the string literal"
    );
    assert!(end > 0, "body should extend to the true closing paren");
}

#[test]
fn collect_call_body_handles_nested_parens_and_templates() {
    // `text` is the slice AFTER the opening `(` of the outer call.
    // The template literal contains `(nested)` which must NOT affect
    // bracket depth.
    let text = "x => `${x}(nested)`), map(y => y)";
    let (body, end) = collect_call_body(text);
    assert!(
        body.contains("`${x}(nested)`"),
        "template literal content should be preserved"
    );
    assert!(end > 0, "should find a closing paren");
}

#[test]
fn collect_call_body_handles_escaped_quotes() {
    // An escaped quote inside a string must not terminate the string.
    let text = r#"map(x => "it\'s (fine)"), tap(y => y)"#;
    let (body, _) = collect_call_body(text);
    assert!(
        body.contains("it\\'s"),
        "escaped quote should be preserved, got: {}",
        body
    );
}

// ── split_top_level: depth-aware comma splitting (Round-6 audit) ───

#[test]
fn split_top_level_ignores_commas_in_nested_braces() {
    let text = r#"selectUserState, selectLoadingState, (userState, loadingState) => ({ users: userState.users, loading: loadingState.loading })"#;
    let parts = split_top_level(text, ',');
    // Top-level split: input 1, input 2, projection fn (commas inside
    // the braces/parens must NOT fragment it).
    assert_eq!(parts.len(), 3, "parts: {:?}", parts);
    assert_eq!(parts[0].trim(), "selectUserState");
    assert_eq!(parts[1].trim(), "selectLoadingState");
    assert!(parts[2].contains("loading: loadingState.loading"));
}

#[test]
fn split_top_level_ignores_commas_in_nested_brackets() {
    let text = "[this.searchTerm$, this.results$], { some: 'option', other: null }";
    let parts = split_top_level(text, ',');
    assert_eq!(parts.len(), 2, "parts: {:?}", parts);
    assert!(parts[0].contains("searchTerm$"));
    assert!(parts[0].contains("results$"));
    assert!(parts[1].contains("'option'"));
}

#[test]
fn split_top_level_ignores_commas_in_strings() {
    let text = r#"createAction('[User] Load, Users'), map(x => x)"#;
    let parts = split_top_level(text, ',');
    assert_eq!(parts.len(), 2, "parts: {:?}", parts);
    assert!(parts[0].contains("'[User] Load, Users'"));
}

#[test]
fn split_top_level_empty_segments_are_skipped() {
    let text = "  a,   , b,  ";
    let parts = split_top_level(text, ',');
    assert_eq!(parts.len(), 2, "parts: {:?}", parts);
    assert_eq!(parts[0].trim(), "a");
    assert_eq!(parts[1].trim(), "b");
}

// ── is_inside_comment_or_string (Round-11 audit) ───────────────────
//
// The meta-layer extractors use this to reject pattern matches that occur
// inside comments (line, trailing, block) or string/template literals.

#[test]
fn is_inside_comment_or_string_detects_line_comment() {
    let src = "const x = 1; // users$ = of(1)";
    let pos = src.find("users$").unwrap();
    assert!(is_inside_comment_or_string(src, pos));
}

#[test]
fn is_inside_comment_or_string_detects_trailing_comment() {
    let src = "users$ = of([]);  // phantom$ = of(1)";
    let pos = src.find("phantom$").unwrap();
    assert!(is_inside_comment_or_string(src, pos));
}

#[test]
fn is_inside_comment_or_string_detects_block_comment() {
    let src = "/* implements CanActivate */";
    let pos = src.find("implements").unwrap();
    assert!(is_inside_comment_or_string(src, pos));
}

#[test]
fn is_inside_comment_or_string_detects_string_literal() {
    let src = r#"label = "combineLatest([a$, b$])""#;
    let pos = src.find("combineLatest").unwrap();
    assert!(is_inside_comment_or_string(src, pos));
}

#[test]
fn is_inside_comment_or_string_false_for_code() {
    let src = "users$ = of([]);";
    let pos = src.find("users$").unwrap();
    assert!(!is_inside_comment_or_string(src, pos));
}

#[test]
fn is_inside_comment_or_string_false_after_comment_ends() {
    let src = "// comment\nusers$ = of([]);";
    let pos = src.find("users$").unwrap();
    assert!(!is_inside_comment_or_string(src, pos));
}

// ── extract_entity_type: nested generics ──────────────────────────

#[test]
fn extract_entity_type_handles_nested_generics() {
    let text = "EntityState<User>>({ ... })";
    let entity = extract_entity_type(text);
    assert_eq!(entity, "EntityState<User>");
}

#[test]
fn extract_entity_type_simple() {
    assert_eq!(extract_entity_type("User>({...})"), "User");
}
