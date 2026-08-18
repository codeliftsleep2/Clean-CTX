// src/compaction/method.rs
//
// Method/function signature compaction across fidelity levels.

use crate::compaction::modifiers::{
    strip_csharp_attributes, strip_modifiers, MODIFIERS_LOW, MODIFIERS_MEDIUM,
};
use crate::compression::Fidelity;

/// Extract a compact method signature from the raw text of a method/function
/// declaration node.
///
/// Input:  "public async getUserById(id: string): Promise<User> { ... }"
/// Low:    "getUserById(id)"
/// Medium: "async getUserById(id:string):Promise<User>"
/// High:   "public async getUserById(id: string): Promise<User>"
pub fn extract_method_sig(text: &str, fidelity: Fidelity) -> String {
    // C# captures may start with attribute lines (`[HttpGet]`,
    // `[HttpGet("{id}")]`); strip them so the signature line is the
    // actual method declaration, not the attribute.
    let stripped = strip_csharp_attributes(text);
    // Work only with the first line (the signature line)
    let sig_line = stripped.lines().next().unwrap_or(stripped);
    let sig_line = sig_line.split('{').next().unwrap_or(sig_line).trim();

    match fidelity {
        Fidelity::Low => compact_method_low(sig_line),
        Fidelity::Medium => compact_method_medium(sig_line),
        // Edit/Verbatim: return the FULL raw method text (signature + body)
        // so the legacy pipeline carries byte-exact bodies for safe
        // `replace_in_file` SEARCH blocks. High keeps the signature only.
        Fidelity::Edit | Fidelity::Verbatim => text.to_string(),
        Fidelity::High => sig_line.to_string(),
    }
}

/// Detect whether a token is a C# return type (as opposed to a TS/Java
/// modifier like `async`). C# uses return-type-first syntax:
///   "ActionResult<UserDto> GetAll(int id)"
///   "void Delete(int id)"
///   "Task<IActionResult> Create(...)"
/// The token immediately before the method name is the return type. A
/// TS/Java modifier that survives `strip_modifiers` is `async` (lowercase,
/// not a type). We distinguish by: primitive C# keyword, generic `<`, or
/// capitalized type name (C# convention).
fn is_csharp_return_type(token: &str) -> bool {
    let t = token.trim();
    if t.is_empty() {
        return false;
    }
    const PRIMITIVES: &[&str] = &[
        "void", "int", "string", "bool", "double", "decimal", "long",
        "short", "byte", "char", "object", "var", "dynamic", "float",
        "uint", "ulong", "ushort", "sbyte",
    ];
    if PRIMITIVES.contains(&t) {
        return true;
    }
    // Generic return type (e.g. "ActionResult<UserDto>", "Task<...>").
    if t.contains('<') {
        return true;
    }
    // Capitalized type name (C# convention).
    t.chars().next().is_some_and(|c| c.is_uppercase())
}

/// Split a signature into `(params, return_type)`.
fn split_params_ret(s: &str) -> (String, String) {
    if let Some(open) = s.find('(') {
        let close = s.rfind(')').unwrap_or(s.len());
        let params = s[open + 1..close].trim().to_string();
        let ret = s[close + 1..].trim().trim_start_matches(':').trim().to_string();
        (params, ret)
    } else {
        (String::new(), String::new())
    }
}

/// Low-fidelity method signature: strip all modifiers, keep only the name and
/// bare parameter names (no types, no return type).
///
/// "public async getUser(id: string, opts?: Options): Promise<User>"
///   → "getUser(id,opts)"
/// "IActionResult Get()"  (C# return-type-first)
///   → "Get()"
fn compact_method_low(sig: &str) -> String {
    // F-16: use the shared `strip_modifiers` helper. The previous
    // local copy of the helper was a near-duplicate of the version
    // in `modifiers.rs`.
    let s = strip_modifiers(sig, MODIFIERS_LOW);

    // s is now "name(params...): ReturnType" or "name<T>(params): ReturnType"
    // or C# "ActionResult<UserDto> GetAll(params)".
    // Extract name: for C# return-type-first, take the last whitespace
    // token before `(`; otherwise take up to the first `(` or `<`.
    let before_paren = s.split('(').next().unwrap_or(&s);
    let tokens: Vec<&str> = before_paren.split_whitespace().collect();
    let name = if tokens.len() >= 2 && is_csharp_return_type(tokens[tokens.len() - 2]) {
        tokens.last().unwrap().split('<').next().unwrap_or(tokens.last().unwrap())
    } else {
        s.split(['(', '<']).next().unwrap_or(&s)
    };

    // Extract param block
    let params = extract_param_names(&s);

    format!("{}({})", name, params.join(","))
}

/// Medium-fidelity method signature: strip access modifiers but keep `async`,
/// type annotations, and return type. Drop default values.
///
/// "public async getUser(id: string, opts?: Options): Promise<User>"
///   → "async getUser(id:string,opts?:Options):Promise<User>"
/// "ActionResult<UserDto> GetAll(int id)"  (C# return-type-first)
///   → "GetAll(id:int)"
fn compact_method_medium(sig: &str) -> String {
    // F-16: use the shared `strip_modifiers` helper.
    let s = strip_modifiers(sig, MODIFIERS_MEDIUM);

    // Detect C# return-type-first and normalize to name-first.
    let before_paren = s.split('(').next().unwrap_or(&s);
    let tokens: Vec<&str> = before_paren.split_whitespace().collect();
    if tokens.len() >= 2 && is_csharp_return_type(tokens[tokens.len() - 2]) {
        // C#: "ActionResult<UserDto> GetAll(id:int)" → "GetAll(id:int)"
        let name = tokens.last().unwrap();
        let (params, ret) = split_params_ret(&s);
        let mut out = format!("{}({})", name, params);
        if !ret.is_empty() {
            out.push(':');
            out.push_str(&ret);
        }
        out
    } else {
        // TS/Java name-first: collapse spaces around punctuation.
        s.replace(": ", ":")
         .replace(" | ", "|")
         .replace(", ", ",")
         .replace(" >", ">")
         .replace("< ", "<")
    }
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
