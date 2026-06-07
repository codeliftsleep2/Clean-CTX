// src/diff/keys.rs
//
// Key extraction and grouping helpers used by the diff comparator.

use std::collections::BTreeMap;

use super::snapshot::{CapturedClass, CapturedMethod};

/// Group a vector of items by a derived key, preserving the relative order
/// of items within each group.
pub(crate) fn group_by_key<T, F>(items: &[T], key_fn: F) -> BTreeMap<String, Vec<&T>>
where
    F: Fn(&T) -> String,
{
    let mut out: BTreeMap<String, Vec<&T>> = BTreeMap::new();
    for item in items {
        let k = key_fn(item);
        out.entry(k).or_default().push(item);
    }
    out
}

pub(crate) fn group_strings_by_key(
    items: &[String],
    key_fn: fn(&str) -> String,
) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in items {
        let k = key_fn(item);
        out.entry(k).or_default().push(item.clone());
    }
    out
}

pub(crate) fn method_key(sig: &str) -> String {
    let end = sig
        .find(|c: char| c == '(' || c == '<' || c.is_whitespace())
        .unwrap_or(sig.len());
    sig[..end].to_string()
}

pub(crate) fn field_key(field: &str) -> String {
    let end = field
        .find(|c: char| c == ':' || c == '?' || c == '=' || c == ';')
        .unwrap_or(field.len());
    field[..end].trim().to_string()
}

pub(crate) fn summarize_class(cls: &CapturedClass) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !cls.fields.is_empty() {
        parts.push(format!("{} fields", cls.fields.len()));
    }
    if !cls.methods.is_empty() {
        parts.push(format!("{} methods", cls.methods.len()));
    }
    if parts.is_empty() {
        String::new()
    } else {
        parts.join(", ")
    }
}

// Bring CapturedMethod into scope to keep the import set explicit even
// though it isn't directly referenced yet. (Removed once tests land here.)
#[allow(dead_code)]
fn _typecheck(_: CapturedMethod) {}
