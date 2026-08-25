// src/compression/type_aliases.rs
//
// R-02: Type-aware compression. Replaces configured type names in the
// compressed body with short alias tokens (`UserId` → `$uid`) and emits
// a reversible `§TA` footer so the LLM can resolve every alias.
//
// Design principles (see docs/TYPE_AWARE_COMPRESSION_PLAN.md):
//
//   - Additive: existing opcodes, markers, and structural output are
//     never modified. This is a pure text pass applied after structural
//     assembly and before micro-opcodes/symbol compression.
//   - Reversible: the `§TA` footer maps `$uid → UserId` for every alias
//     actually used in the body.
//   - Deterministic: aliases are applied longest-key-first to prevent
//     partial matches (`User` before `UserService`).
//   - Single-pass: a left-to-right scan emits each alias token and
//     advances past it, so an emitted alias is never re-scanned by a
//     later (shorter) original — no double substitution.

use std::collections::{BTreeMap, HashSet};

/// Minimum length of an original type name worth substituting.
/// Avoids replacing trivial types (`int`, `str` are exactly 3 chars)
/// where savings are negligible and false-positive risk is highest.
const MIN_ORIGINAL_LEN: usize = 4;

/// Apply configured type aliases to a compressed body.
///
/// Replaces whole-token occurrences of configured type names with their
/// alias tokens (longest key first). Returns the substituted body and a
/// `§TA` footer mapping each *used* alias back to its original type.
///
/// Only aliases that actually appear in the body are emitted in the
/// footer (avoids dead footer entries).
pub fn apply_type_aliases(body: &str, aliases: &BTreeMap<String, String>) -> (String, String) {
    // Filter to valid pairs and sort by original length descending so
    // longer type names are matched before shorter ones (`UserService`
    // before `User`).
    let mut pairs: Vec<(&String, &String)> = aliases
        .iter()
        .filter(|(original, alias)| is_valid_alias(alias) && original.len() >= MIN_ORIGINAL_LEN)
        .collect();
    pairs.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

    if pairs.is_empty() {
        return (body.to_string(), String::new());
    }

    let bytes = body.as_bytes();
    let mut result = String::with_capacity(body.len());
    let mut used: Vec<(&String, &String)> = Vec::new();
    // Track which originals have already been added to `used` to avoid
    // duplicate footer entries when the same type name appears multiple
    // times in the body.
    let mut seen: HashSet<&str> = HashSet::new();
    let mut i = 0;
    while i < bytes.len() {
        // Try to match the longest original at position i.
        let mut matched = false;
        for (original, alias) in &pairs {
            let orig_bytes = original.as_bytes();
            if orig_bytes.len() > bytes.len() - i {
                continue;
            }
            if &bytes[i..i + orig_bytes.len()] == orig_bytes
                && is_boundary_before(bytes, i)
                && is_boundary_after(bytes, i + orig_bytes.len())
            {
                result.push_str(alias);
                // Only add to `used` on first match — subsequent matches
                // of the same original would produce duplicate footer entries.
                if seen.insert(original.as_str()) {
                    used.push((original, alias));
                }
                i += orig_bytes.len();
                matched = true;
                break;
            }
        }
        if !matched {
            // Emit the current character (UTF-8 aware) and advance.
            let ch = body[i..].chars().next().unwrap_or_default();
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    let footer = build_footer(&used);
    (result, footer)
}

/// Validate an alias token.
///
/// Rules (from the R-02 plan):
///   - Must start with `$` (distinguishes from structural markers
///     `⊕`, `Φ`, `§` and symbol refs).
///   - Must be ≥ 2 chars total (at least one char after `$`).
///   - Chars after `$` must be `[A-Za-z0-9_]`.
///   - Must NOT be numeric-only after `$` (`$1`, `$2`, …) — the
///     symbol dictionary owns the `$N` opcode space.
pub fn is_valid_alias(alias: &str) -> bool {
    let Some(rest) = alias.strip_prefix('$') else {
        return false;
    };
    !rest.is_empty()
        && rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !rest.chars().all(|c| c.is_ascii_digit())
}

/// Substitute a single type-name pair in a body string.
///
/// Replaces whole-token occurrences of `original` with `alias` using the
/// same boundary rules as [`apply_type_aliases`]. Exposed for targeted
/// single-pair substitution and unit testing. Not called from production
/// code (the pipeline uses [`apply_type_aliases`] for multi-pair batch
/// substitution), but kept as a public helper for consumers and tests
/// that need single-pair control.
#[allow(dead_code)]
pub fn substitute_type_token(body: &str, original: &str, alias: &str) -> String {
    if !is_valid_alias(alias) || original.len() < MIN_ORIGINAL_LEN {
        return body.to_string();
    }
    let bytes = body.as_bytes();
    let orig_bytes = original.as_bytes();
    let mut result = String::with_capacity(body.len());
    let mut i = 0;
    while i < bytes.len() {
        if orig_bytes.len() <= bytes.len() - i
            && &bytes[i..i + orig_bytes.len()] == orig_bytes
            && is_boundary_before(bytes, i)
            && is_boundary_after(bytes, i + orig_bytes.len())
        {
            result.push_str(alias);
            i += orig_bytes.len();
        } else {
            let ch = body[i..].chars().next().unwrap_or_default();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    result
}

/// Build the `§TA` footer for the aliases actually used in the body.
///
/// Format: `§TA $uid→UserId $jo→JsonObject`
fn build_footer(used: &[(&String, &String)]) -> String {
    if used.is_empty() {
        return String::new();
    }
    let mut footer = String::from("§TA");
    for (original, alias) in used {
        footer.push(' ');
        footer.push_str(alias);
        footer.push('→');
        footer.push_str(original);
    }
    footer
}

/// A boundary character is any non-identifier character. This is a
/// superset of the plan's whitelist (`: < > | ( , ; { } [ ] space tab
/// newline`) and additionally handles dotted namespaces
/// (`System.Collections.Generic.List` → `System.Collections.Generic.$l`).
///
/// `$` is treated as an identifier character (like `_`) so that
/// `$`-prefixed tokens (aliases, symbol-dictionary refs) are atomic.
/// Without this, an original type name could match *inside* a
/// `$`-prefixed token — e.g. `x:$User` with `User → $u` would corrupt
/// to `x:$$u`.
fn is_boundary_char(c: u8) -> bool {
    !c.is_ascii_alphanumeric() && c != b'_' && c != b'$'
}

/// True if position `i` is the start of the string or preceded by a
/// boundary character (so `original` is not part of a longer identifier).
fn is_boundary_before(bytes: &[u8], i: usize) -> bool {
    i == 0 || is_boundary_char(bytes[i - 1])
}

/// True if position `end` is the end of the string or followed by a
/// boundary character (so `original` is not part of a longer identifier).
fn is_boundary_after(bytes: &[u8], end: usize) -> bool {
    end >= bytes.len() || is_boundary_char(bytes[end])
}

#[cfg(test)]
#[path = "../tests/compression/type_aliases.rs"]
mod tests;
