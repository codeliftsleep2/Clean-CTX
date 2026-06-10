use super::*;

#[test]
fn heuristic_picks_csharp_for_csharp_content() {
    let src = "namespace Foo { public class Bar { private void Baz() {} } }";
    assert!(looks_like_csharp(src));
}

#[test]
fn heuristic_picks_typescript_for_typescript_content() {
    let src = "export class Foo { async getUser(id: string): Promise<User> {} }";
    assert!(!looks_like_csharp(src));
}

#[test]
fn detect_language_returns_correct_tuple() {
    let ts_src = "const x: number = 1;";
    let (lang, q) = detect_language(ts_src);
    assert_eq!(q, queries::TS_QUERY);
    let _ = lang;

    let cs_src = "namespace Foo {}";
    let (_, q) = detect_language(cs_src);
    assert_eq!(q, queries::CS_QUERY);
}

#[test]
fn language_for_extension_handles_known_extensions() {
    // F-FULL-16: .js is no longer supported (TypeScript grammar doesn't
    // match all JS constructs). Only .ts and .cs are accepted.
    assert!(language_for_extension("ts").is_some());
    assert!(language_for_extension("js").is_none());
    assert!(language_for_extension("cs").is_some());
    assert!(language_for_extension("py").is_none());
    assert!(language_for_extension("").is_none());
}
