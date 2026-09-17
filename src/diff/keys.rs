// src/diff/keys.rs
//
// Key extraction and grouping helpers used by the diff comparator.

use std::collections::BTreeMap;

use crate::compaction::method::find_method_params;
use crate::compaction::signature;

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
    // The method's own `(` (`find_method_params` — the name-anchored first
    // depth-0 group that is NOT a parenthesized return type) and the
    // identifier that OWNS it (`compaction::signature`) are the authority for
    // the name — never token position.
    //
    // The previous "last whitespace token before the `(`" rule destroyed the
    // identity of a generic method whose type-parameter list contains `, `
    // (`Pair<TFirst, TSecond>` keyed as `TSecond>`) and of a tuple-returning
    // method (`public static (int alpha, int beta) GetPair(...)` keyed as
    // `static`): distinct methods then grouped under one fabricated key.
    if let Some(parts) =
        find_method_params(sig).and_then(|(open, _)| signature::split_head_parts(sig, open))
    {
        return parts.bare_name.to_string();
    }
    // No name-anchored parameter list — keep the legacy reading.
    match sig.split_whitespace().last() {
        Some(last) => last.split('<').next().unwrap_or(last).to_string(),
        None => String::new(),
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
