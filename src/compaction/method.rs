// src/compaction/method.rs
//
// Method/function signature compaction across fidelity levels.

use crate::compaction::modifiers::{
    MODIFIERS_LOW, MODIFIERS_MEDIUM, strip_csharp_attributes, strip_modifiers,
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
    // Take everything before the SIGNATURE/BODY boundary brace. This
    // handles multi-line C# signatures where the parameter list spans
    // multiple lines — the previous implementation only took the first
    // line, producing unbalanced-paren garbage for signatures like:
    //   private void ValidateRow(
    //       DataRow data,
    //       string extra)
    // F-03 diff audit.
    //
    // Expression-bodied members (`M() => expr;`) have NO body brace, so
    // for them the first `{` in the raw text belongs to something else —
    // typically a `{hole}` inside an interpolated string. That made the
    // header stop mid-literal and bled `=>` / string fragments onto the
    // signature line (see regression test
    // `gitdiff_interpolation_does_not_bleed_into_signature_line`).
    // A depth-0 `=>` outside string/char literals therefore ends the
    // signature too, alongside the body brace.
    //
    // ff2a29a audit: the SAME literal-unaware trap applies to the brace
    // boundary itself. A brace-bodied C# constructor whose
    // base-initializer argument is an INTERPOLATED string
    // (`: base($"Unexpected value: {value}, …")`) opens its real body
    // `{` only AFTER the initializer clause; a bare `find('{')` stopped
    // at the interpolation HOLE inside the literal, truncating the
    // header mid-string (regression tests
    // `high_fidelity_base_initializer_interpolation_keeps_full_header`
    // and `gitdiff_ctor_base_initializer_interpolation_not_truncated`).
    // The boundary brace is therefore located with the shared
    // literal-aware scanning contract (`skip_quoted_literal` +
    // paren/bracket depth tracking): only a structural-depth-0 `{`
    // OUTSIDE quoted/interpolated literals may end the signature.
    let brace_end = find_depth_zero_brace(stripped);
    let arrow_end = find_depth_zero_arrow(stripped);
    let sig_end = match (brace_end, arrow_end) {
        (Some(b), Some(a)) => Some(b.min(a)),
        (b, a) => b.or(a),
    };
    let sig = match sig_end {
        Some(end) => &stripped[..end],
        None => stripped,
    };
    let sig = sig.trim();

    match fidelity {
        Fidelity::Low => compact_method_low(sig),
        Fidelity::Medium => compact_method_medium(sig),
        // Edit/Verbatim: return the FULL raw method text (signature + body)
        // so the legacy pipeline carries byte-exact bodies for safe
        // `replace_in_file` SEARCH blocks. High keeps the signature only.
        Fidelity::Edit | Fidelity::Verbatim => text.to_string(),
        Fidelity::High => sig.to_string(),
    }
}

/// Find the byte offset of the first `=>` at paren/bracket depth 0 that
/// lies OUTSIDE string and character literals, or `None`.
///
/// An expression-bodied member (`M() => expr;`) has no body brace, so
/// `=>` is an explicit end-of-signature boundary. The depth-0 constraint
/// keeps `=>` inside the parameter list (TS callback types such as
/// `(cb: (x) => void)`) from terminating the header. Literals are
/// skipped so a default value containing an arrow
/// (`void M(string s = "a => b")`) does not truncate the signature.
///
/// Conservative by construction: an unmatched quote consumes to EOF and
/// a missed arrow merely restores the legacy first-`{` behavior; the
/// function can never manufacture a spurious boundary from commented or
/// quoted text.
fn find_depth_zero_arrow(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(bytes, i);
                continue;
            }
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b'=' if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b'>' => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Advance past a quoted literal beginning at `open`, returning the index
/// just past its closing quote. Backslash escapes are honored; verbatim
/// (`@"..."`) and raw-literal quote-doubling degenerate to scanning for
/// the next unescaped closer, which can only cause a MISSED `=>` boundary
/// (never a spurious one — see `find_depth_zero_arrow`).
fn skip_quoted_literal(bytes: &[u8], open: usize) -> usize {
    let mut i = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            ch if ch == bytes[open] => return i + 1,
            _ => i += 1,
        }
    }
    bytes.len()
}

/// Find the byte offset of the first `{` at paren/bracket depth 0 that
/// lies OUTSIDE string and character literals, or `None`.
///
/// This is the SIGNATURE/BODY boundary for brace-bodied declarations. A
/// bare `find('{')` would instead stop at the first `{` anywhere in the
/// text — which for a constructor whose base-initializer argument is an
/// interpolated string is an interpolation HOLE inside the literal
/// (`: base($"msg: {param}")`, ff2a29a audit), truncating the extracted
/// signature mid-literal.
///
/// Shares the conservative-by-construction contract of
/// `find_depth_zero_arrow`: `skip_quoted_literal` makes interpolation
/// hole braces invisible, and a degenerate/unmatched quote merely defers
/// to the arrow bound — the function can never manufacture a spurious
/// boundary from commented or quoted text. Depth tracking additionally
/// keeps braces nested inside the parameter list or an initializer
/// argument (`M(opts = { a: 1 })`) out of play until true structural
/// depth 0 is restored.
fn find_depth_zero_brace(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(bytes, i);
                continue;
            }
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth -= 1,
            b'{' if depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find the byte range of a method's parameter list — the FIRST balanced
/// paren group at depth 0 that is ANCHORED TO THE DECLARED NAME: its opening
/// `(` is immediately preceded (after whitespace) by `>` (a generic-type
/// close) or by the end of an identifier that is neither the `base` nor
/// `this` constructor-initializer keyword.
///
/// C# tuple return types like
///   `Task<(Dictionary<string, Guid> Exact, Dictionary<string, Guid> IgnoreCase)> GetOrgUnitDlc(int id)`
/// open a top-level `(` for the tuple, but that group is preceded by `<`,
/// so it is skipped and the method's own group is selected. Constructor
/// initializers (`Greeter(string prefix) : base(prefix)`) are call sites that
/// FOLLOW the parameter list; the name anchor selects the declaration's own
/// group ahead of them. The scan skips string/char literals so a default
/// value such as `void M(string s = "a (", int n)` cannot break it.
///
/// Returns `Some((start, end))` byte indices of the `(` and matching `)`, or
/// `None` when no name-anchored balanced group exists (or the parens are
/// unbalanced — defensive).
pub(crate) fn find_method_params(sig: &str) -> Option<(usize, usize)> {
    let bytes = sig.as_bytes();
    let mut depth = 0i32;
    let mut start: Option<usize> = None;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(bytes, i);
                continue;
            }
            b'(' => {
                if depth == 0 {
                    start = if is_name_anchored(sig, i) {
                        Some(i)
                    } else {
                        None
                    };
                }
                depth += 1;
            }
            b'[' => depth += 1,
            b')' | b']' => {
                depth = (depth - 1).max(0);
                if depth == 0 {
                    if let Some(s) = start {
                        if s < i {
                            return Some((s, i));
                        }
                    }
                    start = None;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Whether the `(` at byte `at` is immediately preceded (after whitespace) by
/// `>` (a generic-type close) or by the end of an identifier token other than
/// the `base`/`this` constructor-initializer keywords.
fn is_name_anchored(sig: &str, at: usize) -> bool {
    let bytes = sig.as_bytes();
    let mut p = at;
    while p > 0 && bytes[p - 1].is_ascii_whitespace() {
        p -= 1;
    }
    if p == 0 {
        return false;
    }
    let prev = bytes[p - 1];
    if prev == b'>' {
        return true;
    }
    if !(prev.is_ascii_alphanumeric() || prev == b'_') {
        return false;
    }
    // Back up to the start of the identifier token ending at `prev`.
    let tok_end = p - 1;
    let mut tok_start = tok_end;
    while tok_start > 0
        && (bytes[tok_start - 1].is_ascii_alphanumeric() || bytes[tok_start - 1] == b'_')
    {
        tok_start -= 1;
    }
    let ident = &sig[tok_start..=tok_end];
    ident != "base" && ident != "this"
}

/// Drop a trailing C# constructor initializer clause (`: base(...)` /
/// `: this(...)`) from a method-header slice, returning the shorter subslice.
///
/// An initializer clause is a CALL SITE, not signature content: it must never
/// render as a return-type annotation (`M(...):base(...)`) nor feed the
/// parameter/name scan. The boundary `:` is accepted only at structural
/// depth 0 outside string/char literals, so an interpolated-argument clause
/// (`: base($"msg: {p}")`) cannot mask it.
pub(crate) fn strip_base_initializer_clause(sig: &str) -> &str {
    if sig.is_empty() {
        return sig;
    }
    let bytes = sig.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                i = skip_quoted_literal(bytes, i);
                continue;
            }
            b'(' | b'[' => depth += 1,
            b')' | b']' => {
                depth = (depth - 1).max(0);
            }
            b':' if depth == 0 => {
                let rest = sig[i + 1..].trim_start();
                let ident_end = rest
                    .char_indices()
                    .find_map(|(idx, c)| {
                        if c.is_alphanumeric() || c == '_' {
                            None
                        } else {
                            Some(idx)
                        }
                    })
                    .unwrap_or(rest.len());
                let ident = &rest[..ident_end];
                if (ident == "base" || ident == "this")
                    && rest[ident_end..].trim_start().starts_with('(')
                {
                    // The clause ran from this `:` to end-of-header; the
                    // whitespace between the parameter list and the `:` is
                    // not signature content either.
                    return sig[..i].trim_end();
                }
            }
            _ => {}
        }
        i += 1;
    }
    sig
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
///
/// `pub(crate)` so `src/diff/keys.rs` can reuse it for C#-aware method
/// key extraction (F-02 diff audit: `method_key` was taking the return
/// type as the key for C# return-type-first signatures, producing
/// doubled tokens like `+ method bool bool Resolve(...)`).
pub(crate) fn is_csharp_return_type(token: &str) -> bool {
    let t = token.trim();
    if t.is_empty() {
        return false;
    }
    const PRIMITIVES: &[&str] = &[
        "void", "int", "string", "bool", "double", "decimal", "long", "short", "byte", "char",
        "object", "var", "dynamic", "float", "uint", "ulong", "ushort", "sbyte",
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
///
/// Uses `find_method_params` so a C# tuple return type (which opens a
/// top-level `(` for the tuple) does not get mis-tokenized as the
/// parameter list.
fn split_params_ret(s: &str) -> (String, String) {
    if let Some((open, close)) = find_method_params(s) {
        let params = s[open + 1..close].trim().to_string();
        let ret = s[close + 1..]
            .trim()
            .trim_start_matches(':')
            .trim()
            .to_string();
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
    // token before the method's own `(` (the name-anchored first depth-0
    // group from `find_method_params`, so a C# tuple return type is not
    // mis-tokenized); otherwise take up to the first `(` or `<`.
    let before_paren = match find_method_params(&s) {
        Some((open, _)) => &s[..open],
        None => &s,
    };
    let has_param_group = before_paren.len() != s.len() || s.contains('(');
    let tokens: Vec<&str> = before_paren.split_whitespace().collect();
    let name = if tokens.len() >= 2 && is_csharp_return_type(tokens[tokens.len() - 2]) {
        tokens
            .last()
            .unwrap()
            .split('<')
            .next()
            .unwrap_or(tokens.last().unwrap())
    } else if has_param_group && !tokens.is_empty() {
        // Non-CBM audit 2026-08-25 #2: when the parameter list was located
        // STRUCTURALLY, the identifier is simply the last whitespace token
        // before it — regardless of naming convention and regardless of
        // whether a named-tuple return type defeats the
        // `is_csharp_return_type` heuristic (e.g.
        // `Task<(A section, Guid requestId)> CreateRecordWithDefaults(...)`
        // previously fell into the split-at-first-`<` fallback and yielded
        // the whole type prefix `Task` / `internal static async Task` as
        // the "name"). Splitting only the final token keeps generic names
        // (`Foo<T>` → `Foo`) identical to the detected branch above.
        tokens
            .last()
            .unwrap()
            .split(['(', '<'])
            .next()
            .unwrap_or(tokens.last().unwrap())
    } else {
        // No parameter list at all — keep the legacy split behavior.
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
    let stripped = strip_modifiers(sig, MODIFIERS_MEDIUM);
    // A C# constructor initializer clause (`: base(...)`/`: this(...)`) is a
    // call site, not signature content: dropping it first keeps the clause
    // from collapsing onto the label as a fake return-type annotation.
    let s = strip_base_initializer_clause(&stripped);

    // Detect C# return-type-first and normalize to name-first.
    // Use the method's own `(` (`find_method_params` — the name-anchored
    // first depth-0 group) so a C# tuple return type is not mis-tokenized
    // as the parameter list.
    let before_paren = match find_method_params(s) {
        Some((open, _)) => &s[..open],
        None => s,
    };
    let tokens: Vec<&str> = before_paren.split_whitespace().collect();
    if tokens.len() >= 2 && is_csharp_return_type(tokens[tokens.len() - 2]) {
        // C#: "ActionResult<UserDto> GetAll(id:int)" → "GetAll(id:int)"
        let name = tokens.last().unwrap();
        let (params, ret) = split_params_ret(s);
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
    // Use the method's own `(` (`find_method_params` — the name-anchored
    // first depth-0 group) so a C# tuple return type is not mis-tokenized
    // as the parameter list.
    let Some((open, close)) = find_method_params(sig) else {
        return Vec::new();
    };

    let params_str = &sig[open + 1..close];
    if params_str.trim().is_empty() {
        return Vec::new();
    }

    params_str
        .split(',')
        .map(|p| {
            let p = p.trim();
            // TS/Java name-first: "id: string" → "id" (before the colon).
            // C# type-first: "int id" → "id" (last whitespace token).
            // A colon may also appear in a default value ("x = foo:bar"),
            // so only split on the FIRST colon.
            let name_part = if let Some(colon) = p.find(':') {
                p[..colon].trim()
            } else {
                p.split_whitespace().last().unwrap_or(p)
            };
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
