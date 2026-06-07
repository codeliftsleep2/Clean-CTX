// src/compaction/method.rs
//
// Method/function signature compaction across fidelity levels.

use crate::compression::Fidelity;
use crate::compaction::modifiers::{MODIFIERS_LOW, MODIFIERS_MEDIUM};

/// Extract a compact method signature from the raw text of a method/function
/// declaration node.
///
/// Input:  "public async getUserById(id: string): Promise<User> { ... }"
/// Low:    "getUserById(id)"
/// Medium: "async getUserById(id:string):Promise<User>"
/// High:   "public async getUserById(id: string): Promise<User>"
pub fn extract_method_sig(text: &str, fidelity: Fidelity) -> String {
    // Work only with the first line (the signature line)
    let sig_line = text.lines().next().unwrap_or(text);
    let sig_line = sig_line.split('{').next().unwrap_or(sig_line).trim();

    match fidelity {
        Fidelity::Low => compact_method_low(sig_line),
        Fidelity::Medium => compact_method_medium(sig_line),
        Fidelity::High => sig_line.to_string(),
    }
}

/// Low-fidelity method signature: strip all modifiers, keep only the name and
/// bare parameter names (no types, no return type).
///
/// "public async getUser(id: string, opts?: Options): Promise<User>"
///   → "getUser(id,opts)"
fn compact_method_low(sig: &str) -> String {
    // Reuse the shared modifier list — `MODIFIERS_LOW` is the single
    // source of truth (Phase 2 consolidation). The helper
    // `strip_modifiers` is local to this module and not exported.
    let s = strip_modifiers(sig.trim(), MODIFIERS_LOW);

    // s is now "name(params...): ReturnType" or "name<T>(params): ReturnType"
    // Extract name (up to first `(` or `<`)
    let name = s.split(['(', '<']).next().unwrap_or(&s);

    // Extract param block
    let params = extract_param_names(&s);

    format!("{}({})", name, params.join(","))
}

/// Medium-fidelity method signature: strip access modifiers but keep `async`,
/// type annotations, and return type. Drop default values.
///
/// "public async getUser(id: string, opts?: Options): Promise<User>"
///   → "async getUser(id:string,opts?:Options):Promise<User>"
fn compact_method_medium(sig: &str) -> String {
    // Reuse the shared modifier list — `MODIFIERS_MEDIUM` keeps `async`
    // and `readonly` (which are not in the medium-strip list).
    let s = strip_modifiers(sig.trim(), MODIFIERS_MEDIUM);

    // Collapse spaces around punctuation in the signature
    s.replace(": ", ":")
     .replace(" | ", "|")
     .replace(", ", ",")
     .replace(" >", ">")
     .replace("< ", "<")
}

/// Repeatedly strip any of `modifiers` from the start of `s`, trimming
/// whitespace between prefixes. Pure function: returns a new `String`.
fn strip_modifiers(s: &str, modifiers: &[&str]) -> String {
    let mut s = s.to_string();
    loop {
        let mut stripped = false;
        for m in modifiers {
            if let Some(rest) = s.strip_prefix(m) {
                s = rest.trim().to_string();
                stripped = true;
            }
        }
        if !stripped { break; }
    }
    s
}

/// Extract bare parameter names from a method signature string.
/// Handles optional markers (`?`), rest params (`...`), and ignores defaults.
fn extract_param_names(sig: &str) -> Vec<String> {
    let Some(open) = sig.find('(') else { return Vec::new(); };
    let close = sig.rfind(')').unwrap_or(sig.len());
    if open >= close { return Vec::new(); }

    let params_str = &sig[open + 1..close];
    if params_str.trim().is_empty() { return Vec::new(); }

    params_str
        .split(',')
        .map(|p| {
            // Take the part before `:` (the name), strip optional/rest markers
            let name_part = p.split(':').next().unwrap_or(p).trim();
            // Strip default value if no colon present
            let name_part = name_part.split('=').next().unwrap_or(name_part).trim();
            // Remove `...` rest prefix, trailing `?`
            name_part
                .trim_start_matches("...")
                .trim_end_matches('?')
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
#[path = "../tests/compaction/method.rs"]
mod tests;
