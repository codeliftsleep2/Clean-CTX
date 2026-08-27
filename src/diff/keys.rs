// src/diff/keys.rs
//
// Key extraction and grouping helpers used by the diff comparator.

use std::collections::BTreeMap;

use crate::compaction::method::{find_method_params, is_csharp_return_type};

use super::snapshot::CapturedClass;

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

/// Extract the method name from a compact signature for grouping.
///
/// For TS/Java name-first signatures (`getUser(id:string):Promise<User>`)
/// this is the text before the first `(` or `<`. For C# return-type-first
/// signatures (`bool Resolve(term,__)`, `GetTestOrgUnitValidatorData
/// GetTestOrgUnitValidatorData()`) the return type must be skipped —
/// otherwise the key becomes `bool` or `GetTestOrgUnitValidatorData`
/// (the return type), producing doubled tokens in the rendered diff
/// (`+ method bool bool Resolve(...)`) and incorrect grouping of
/// methods that share a return type. F-02 diff audit.
pub(crate) fn method_key(sig: &str) -> String {
    // Use the method's own `(` (`find_method_params` — the name-anchored
    // first depth-0 group) so a C# tuple return type is not mis-tokenized
    // as the parameter list.
    let before_paren = match find_method_params(sig) {
        Some((open, _)) => &sig[..open],
        None => sig,
    };
    let tokens: Vec<&str> = before_paren.split_whitespace().collect();
    if tokens.len() >= 2 && is_csharp_return_type(tokens[tokens.len() - 2]) {
        // C# return-type-first: the method name is the last token.
        tokens
            .last()
            .unwrap()
            .split('<')
            .next()
            .unwrap_or(tokens.last().unwrap())
            .to_string()
    } else if tokens.is_empty() {
        // Defensive: the signature begins with `(`/`<` — no name prefix.
        String::new()
    } else {
        // TS/Java name-first: the method name is the LAST whitespace token
        // before the `(`/`<`. Leading declarator keywords like
        // `export function foo`, `export async function foo`, or
        // `async function foo` are NOT stripped by `strip_modifiers`
        // (MODIFIERS_MEDIUM has no `export`/`function`), so taking the
        // FIRST token mis-keyed every top-level function as "export" or
        // "async" — all top-level functions in a file grouped under the
        // same key and the rendered label was wrong. G3-5 diff audit.
        tokens
            .last()
            .unwrap()
            .split('<')
            .next()
            .unwrap_or(tokens.last().unwrap())
            .to_string()
    }
}

pub(crate) fn field_key(field: &str) -> String {
    let end = field.find([':', '?', '=', ';']).unwrap_or(field.len());
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
