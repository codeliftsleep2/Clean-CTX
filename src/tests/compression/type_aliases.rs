// src/tests/compression/type_aliases.rs
//
// R-02 Phase 1 tests: type-alias substitution pass.
//
// Covers token-boundary matching, nested generics, union types,
// optional/array types, collision avoidance, longest-key-first
// ordering, footer emission, and determinism.

use crate::compression::type_aliases::{apply_type_aliases, is_valid_alias, substitute_type_token};
use std::collections::BTreeMap;

fn aliases(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// ── is_valid_alias ────────────────────────────────────────────────

#[test]
fn valid_alias_starts_with_dollar() {
    assert!(is_valid_alias("$uid"));
    assert!(is_valid_alias("$jo2"));
    assert!(is_valid_alias("$_http"));
}

#[test]
fn invalid_alias_rejected() {
    assert!(!is_valid_alias("uid")); // no $
    assert!(!is_valid_alias("$")); // nothing after $
    assert!(!is_valid_alias("$1")); // collides with symbol-dictionary $1
    assert!(!is_valid_alias("$u id")); // space not allowed
    assert!(!is_valid_alias("$u-id")); // hyphen not allowed
}

// ── Token-boundary matching ───────────────────────────────────────

#[test]
fn no_partial_identifier_match() {
    // "User" must NOT match inside "UserService" or "GitUserProfile"
    let body = "getUser(id:string):Promise<UserService>";
    let cfg = aliases(&[("User", "$u")]);
    let (out, _footer) = apply_type_aliases(body, &cfg);
    assert!(!out.contains("$uService"));
    assert!(!out.contains("$u"));
}

#[test]
fn footer_no_duplicates_when_type_appears_multiple_times() {
    // FAANG audit (HIGH): when the same type name appears multiple times
    // in the body, the footer must list it only once — not once per match.
    let body = "a:User b:User c:Promise<User>";
    let (out, footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    // Body should have 3 substitutions
    assert_eq!(out, "a:$uid b:$uid c:Promise<$uid>");
    // Footer should have exactly 1 entry for $uid→User (not 3)
    let count = footer.matches("$uid→User").count();
    assert_eq!(
        count, 1,
        "footer should list $uid→User exactly once, got: {}",
        footer
    );
}

#[test]
fn replacement_at_type_positions() {
    let body = "getUser(id:string):Promise<User>";
    let (out, footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "getUser(id:string):Promise<$uid>");
    assert_eq!(footer, "§TA $uid→User");
}

#[test]
fn nested_generics_substituted() {
    let body = "getUsers():Map<string,User>";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "getUsers():Map<string,$uid>");
}

#[test]
fn union_types_substituted() {
    let body = "id:A | User";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "id:A | $uid");
}

#[test]
fn optional_and_array_types_substituted() {
    let mut body = "id:User?";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "id:$uid?");

    body = "ids:User[]";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "ids:$uid[]");
}

#[test]
fn underscore_boundary_does_not_match() {
    // "User" must NOT match inside "user_id" (lowercase start also guards),
    // nor inside "User_Name" (underscore is an identifier char).
    let body = "x:User_Name";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "x:User_Name");
}

#[test]
fn dollar_prefixed_token_not_corrupted() {
    // FAANG audit (HIGH): `$` is an identifier char, so an original type
    // name must NOT match *inside* a `$`-prefixed token. Without this,
    // `x:$User` with `User → $u` would corrupt to `x:$$u`.
    let body = "x:$User";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$u")]));
    assert_eq!(out, "x:$User");

    // Symbol-dictionary refs (`$1`, `$2`) are also atomic.
    let body = "x:$1User";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$u")]));
    assert_eq!(out, "x:$1User");
}

// ── Collision avoidance ───────────────────────────────────────────

#[test]
fn one_char_alias_rejected() {
    // Alias "$1" is invalid (collides with symbol-dictionary opcodes).
    let body = "x:User";
    let (out, footer) = apply_type_aliases(body, &aliases(&[("User", "$1")]));
    assert_eq!(out, "x:User");
    assert!(footer.is_empty());
}

#[test]
fn short_original_not_substituted() {
    // Original types shorter than 3 chars are not worth substituting.
    let body = "x:int";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("int", "$i")]));
    assert_eq!(out, "x:int");
}

// ── Longest-key-first ordering ────────────────────────────────────

#[test]
fn longest_key_matched_first() {
    // Both "User" and "UserService" configured. "UserService" must win
    // at "UserService" positions; "User" still applies elsewhere.
    let body = "a:UserService b:User";
    let (out, footer) =
        apply_type_aliases(body, &aliases(&[("User", "$u"), ("UserService", "$usvc")]));
    assert_eq!(out, "a:$usvc b:$u");
    assert!(footer.contains("$usvc→UserService"));
    assert!(footer.contains("$u→User"));
}

#[test]
fn no_double_substitution() {
    // After "User" → "$uid", the emitted "$uid" must not be re-scanned
    // by another original that happens to match its text.
    let body = "x:User";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$uid"), ("$uid", "$z")]));
    assert_eq!(out, "x:$uid");
}

// ── Footer emission ───────────────────────────────────────────────

#[test]
fn footer_only_includes_used_aliases() {
    let body = "a:User";
    let (_, footer) =
        apply_type_aliases(body, &aliases(&[("User", "$uid"), ("JsonObject", "$jo")]));
    assert!(footer.contains("$uid→User"));
    assert!(!footer.contains("$jo")); // unused → not emitted
}

#[test]
fn empty_aliases_noop() {
    let body = "x:User";
    let (out, footer) = apply_type_aliases(body, &BTreeMap::new());
    assert_eq!(out, "x:User");
    assert!(footer.is_empty());
}

#[test]
fn footer_empty_when_nothing_matched() {
    let body = "x:Service";
    let (out, footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "x:Service");
    assert!(footer.is_empty());
}

// ── Determinism ───────────────────────────────────────────────────

#[test]
fn deterministic_output() {
    let body = "a:User b:Map<string,User>";
    let cfg = aliases(&[("User", "$uid")]);
    let (out1, f1) = apply_type_aliases(body, &cfg);
    let (out2, f2) = apply_type_aliases(body, &cfg);
    assert_eq!(out1, out2);
    assert_eq!(f1, f2);
}

// ── Single-pair substitution helper ───────────────────────────────

#[test]
fn substitute_type_token_single_pair() {
    assert_eq!(substitute_type_token("id:User", "User", "$uid"), "id:$uid");
    assert_eq!(
        substitute_type_token("id:UserService", "User", "$uid"),
        "id:UserService"
    );
    // Invalid alias → no-op
    assert_eq!(substitute_type_token("id:User", "User", "$1"), "id:User");
}

// ── UTF-8 safety ──────────────────────────────────────────────────

#[test]
fn unicode_content_preserved() {
    // Non-ASCII content must survive byte-oriented scanning intact.
    let body = "α:User // héllo";
    let (out, _footer) = apply_type_aliases(body, &aliases(&[("User", "$uid")]));
    assert_eq!(out, "α:$uid // héllo");
}
