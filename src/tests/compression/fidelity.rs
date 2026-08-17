// src/tests/compression/fidelity.rs
//
// Tests for `Fidelity::parse` (FAANG audit F-03). The whole point of
// the rewrite is that unrecognised inputs now produce a hard error
// rather than silently downgrading to `Low`.

use super::*;

#[test]
fn parse_low_accepted() {
    assert_eq!(Fidelity::parse("low").unwrap(), Fidelity::Low);
}

#[test]
fn parse_medium_accepted() {
    assert_eq!(Fidelity::parse("medium").unwrap(), Fidelity::Medium);
}

#[test]
fn parse_high_accepted() {
    assert_eq!(Fidelity::parse("high").unwrap(), Fidelity::High);
}

#[test]
fn parse_is_case_insensitive() {
    assert_eq!(Fidelity::parse("LOW").unwrap(), Fidelity::Low);
    assert_eq!(Fidelity::parse("Medium").unwrap(), Fidelity::Medium);
    assert_eq!(Fidelity::parse("HIGH").unwrap(), Fidelity::High);
}

#[test]
fn parse_typo_rejected() {
    // The headline regression: "hihg" used to silently map to Low.
    let err = Fidelity::parse("hihg").unwrap_err();
    assert_eq!(err.0, "hihg");
}

#[test]
fn parse_empty_string_rejected() {
    let err = Fidelity::parse("").unwrap_err();
    assert_eq!(err.0, "");
}

#[test]
fn parse_emoji_rejected() {
    let err = Fidelity::parse("🚀").unwrap_err();
    assert_eq!(err.0, "🚀");
}

#[test]
fn parse_numeric_rejected() {
    let err = Fidelity::parse("1").unwrap_err();
    assert_eq!(err.0, "1");
}

#[test]
fn parse_preserves_offending_value_in_error() {
    // The display impl surfaces the bad value so the operator can fix it.
    let err = Fidelity::parse("hihg").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("hihg"), "error must include the bad value: {}", msg);
    assert!(
        msg.contains("low") && msg.contains("medium") && msg.contains("high"),
        "error must list valid options: {}",
        msg
    );
}

#[test]
fn parse_edit_accepted() {
    assert_eq!(Fidelity::parse("edit").unwrap(), Fidelity::Edit);
}

#[test]
fn parse_verbatim_accepted() {
    assert_eq!(Fidelity::parse("verbatim").unwrap(), Fidelity::Verbatim);
}

#[test]
fn parse_edit_case_insensitive() {
    assert_eq!(Fidelity::parse("EDIT").unwrap(), Fidelity::Edit);
    assert_eq!(Fidelity::parse("Verbatim").unwrap(), Fidelity::Verbatim);
}

#[test]
fn parse_or_default_accepts_valid_input() {
    assert_eq!(Fidelity::parse_or_default("low"), Fidelity::Low);
    assert_eq!(Fidelity::parse_or_default("medium"), Fidelity::Medium);
    assert_eq!(Fidelity::parse_or_default("high"), Fidelity::High);
    assert_eq!(Fidelity::parse_or_default("edit"), Fidelity::Edit);
    assert_eq!(Fidelity::parse_or_default("verbatim"), Fidelity::Verbatim);
}

#[test]
fn parse_or_default_falls_back_to_low_on_typo() {
    // The lenient path: explicitly opted into by callers (e.g. config
    // loaders). Still returns Low on bad input, but at least the
    // operator sees a stderr warning.
    assert_eq!(Fidelity::parse_or_default("hihg"), Fidelity::Low);
    assert_eq!(Fidelity::parse_or_default(""), Fidelity::Low);
    assert_eq!(Fidelity::parse_or_default("🚀"), Fidelity::Low);
}

#[test]
fn fidelity_is_hashable() {
    // Sanity check on the derived trait — a missing `Hash` impl would
    // break F-10's planned cache-key refactor.
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Fidelity::Low);
    set.insert(Fidelity::Medium);
    set.insert(Fidelity::High);
    assert_eq!(set.len(), 3);
    assert!(set.contains(&Fidelity::Low));
    assert!(set.contains(&Fidelity::Medium));
    assert!(set.contains(&Fidelity::High));
}
