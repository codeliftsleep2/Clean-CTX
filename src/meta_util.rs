//! Layer-agnostic parsing utilities shared across ALL meta-layers
//! (Angular, Spring, .NET).
//!
//! All scanners across the meta-layers MUST use these primitives instead of
//! hand-rolling their own string/depth awareness. This eliminates the defect
//! class where a fix in one layer is not propagated to duplicated logic in
//! another (Round-8 structural audit).
//!
//! This module is deliberately free of any Angular/Spring/.NET-specific
//! vocabulary so every meta-layer can depend on it without a layering
//! violation. `angular_meta::util` re-exports these for backward
//! compatibility with the Angular sub-layers.

use crate::compression::capture_pipeline::CapEntry;

/// Split `text` on `delim` (usually ',') at depth zero, respecting:
/// - nested `()`, `[]`, `{}` groups
/// - string literals ('...', "...", `...`) with escaped-quote handling
/// - template-literal `${...}` interpolation
///
/// Returns trimmed, non-empty segments.
pub fn split_top_level(text: &str, delim: char) -> Vec<String> {
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut depth: i32 = 0;
    let mut chars = text.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            '(' | '[' | '{' if depth >= 0 => depth += 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            '\'' => {
                skip_string(&mut chars, i, '\'');
            }
            '"' => {
                skip_string(&mut chars, i, '"');
            }
            '`' => {
                skip_template(&mut chars, i);
            }
            c if c == delim && depth == 0 => {
                let seg = text[start..i].trim();
                if !seg.is_empty() {
                    segments.push(seg.to_string());
                }
                start = i + delim.len_utf8();
            }
            _ => {}
        }
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        segments.push(tail.to_string());
    }
    segments
}

/// Find the matching closing bracket for the bracket at `text[open_idx..]`.
/// `open` must be `(`, `[`, or `{` and is the first char of `text`.
///
/// Returns the byte index (relative to `text`) of the matching close bracket,
/// or `None` if unbalanced.
///
/// String-aware: parens/brackets inside string literals or template
/// interpolations are ignored. Escaped quotes are handled.
pub fn find_matching_brace(text: &str, open: char) -> Option<usize> {
    let (close, other_open, other_close) = match open {
        '(' => (')', '[', ']'),
        '[' => (']', '(', ')'),
        '{' => ('}', '(', ')'),
        _ => return None,
    };
    let mut depth: i32 = 0;
    let mut chars = text.char_indices().peekable();
    let mut in_string: Option<(char, bool)> = None; // (quote, is_template_with_interp)
    let mut interp_depth: i32 = 0;

    while let Some((i, c)) = chars.next() {
        if let Some((quote, is_template)) = in_string {
            if is_template && interp_depth > 0 {
                // inside ${...} interpolation — treat like code
                match c {
                    '{' => interp_depth += 1,
                    '}' => {
                        interp_depth -= 1;
                        if interp_depth == 0 {
                            in_string = Some(('`', true));
                        }
                    }
                    _ => {
                        if c == quote {
                            in_string = Some(('`', true));
                        }
                    }
                }
            } else if c == '\\' {
                chars.next(); // skip escaped char
            } else if c == quote {
                in_string = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                in_string = Some((c, c == '`'));
            }
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            c if c == other_open => depth += 1,
            c if c == other_close => depth -= 1,
            _ => {}
        }
    }
    None
}

/// Find the first occurrence of `open` at bracket-depth zero, scanning
/// forward from byte index `start`.
///
/// `open` must be `(`, `[`, or `{`. Only the given bracket pair contributes
/// to depth (matching the original per-layer `find_class_body_open`
/// semantics — parens/brackets of other kinds do not affect the scan).
///
/// Returns the absolute byte index (relative to `text`) of the first
/// depth-zero `open`, or `None` if none is found.
///
/// String-aware: brackets inside string literals or template interpolations
/// are ignored. Escaped quotes are handled.
pub fn find_first_top_level(text: &str, open: char, start: usize) -> Option<usize> {
    let close = match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => return None,
    };
    let mut depth: i32 = 0;
    let mut chars = text[start..].char_indices().peekable();
    let mut in_string: Option<(char, bool)> = None;
    let mut interp_depth: i32 = 0;

    while let Some((rel, c)) = chars.next() {
        let i = start + rel;
        if let Some((quote, is_template)) = in_string {
            if is_template && interp_depth > 0 {
                // inside ${...} interpolation — treat like code
                match c {
                    '{' => interp_depth += 1,
                    '}' => {
                        interp_depth -= 1;
                        if interp_depth == 0 {
                            in_string = Some(('`', true));
                        }
                    }
                    _ => {
                        if c == quote {
                            in_string = Some(('`', true));
                        }
                    }
                }
            } else if c == '\\' {
                chars.next(); // skip escaped char
            } else if c == quote {
                in_string = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                in_string = Some((c, c == '`'));
            }
            c if c == open => {
                if depth == 0 {
                    return Some(i);
                }
                depth += 1;
            }
            c if c == close => {
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Find the enclosing `{` for the position at byte index `pos` in `source`.
///
/// Scans backward from `pos`, skipping over string literals / template
/// interpolations (determined via a forward pre-pass that records per-char
/// in-string state). Returns the byte index of the matching `{`.
pub fn find_enclosing_brace(source: &str, pos: usize) -> Option<usize> {
    // Forward pass: record per-char in-string state so the backward scan
    // doesn't confuse braces inside strings/templates.
    let mut in_string_at = vec![false; source.len()];
    let mut chars = source.char_indices().peekable();
    let mut in_string: Option<(char, bool)> = None;
    let mut interp_depth: i32 = 0;

    while let Some((i, c)) = chars.next() {
        if let Some((quote, is_template)) = in_string {
            in_string_at[i] = true;
            if is_template && interp_depth > 0 {
                match c {
                    '{' => interp_depth += 1,
                    '}' => {
                        interp_depth -= 1;
                        if interp_depth == 0 {
                            in_string = Some(('`', true));
                        }
                    }
                    _ => {
                        if c == quote {
                            in_string = Some(('`', true));
                        }
                    }
                }
            } else if c == '\\' {
                if let Some((j, _)) = chars.next() {
                    in_string_at[j] = true;
                }
            } else if c == quote {
                in_string = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                in_string = Some((c, c == '`'));
            }
            _ => {}
        }
    }

    // Backward scan for enclosing `{`.
    // Only brace-characters that are NOT in-string count.
    // Also skip a `(` or `[` if we hit one at the same nesting level,
    // since a `{` inside call arguments belongs to the function body.
    let mut depth: i32 = 0;
    let mut paren_depth: i32 = 0;
    let mut bunker: i32 = 0;

    let bytes = source.as_bytes();
    let mut i = pos.saturating_sub(1);
    while i > 0 {
        let c = bytes[i] as char;
        if !in_string_at[i] {
            match c {
                '}' => depth += 1,
                '{' if depth == 0 && paren_depth == 0 && bunker == 0 => return Some(i),
                '{' if depth > 0 => depth -= 1,
                '{' => {}
                ')' => paren_depth += 1,
                '(' if paren_depth > 0 => paren_depth -= 1,
                ']' => bunker += 1,
                '[' if bunker > 0 => bunker -= 1,
                _ => {}
            }
        }
        i -= 1;
    }
    None
}

/// Extract the value of `key` from a JS object literal `obj`.
///
/// Handles quoted keys (`"path"`, `'path'`), escaped quotes in the value,
/// and nested object literals / arrays / template literals in the value.
pub fn extract_quoted_value<'a>(obj: &'a str, key: &str) -> Option<&'a str> {
    // Match `key` or `"key"` or `'key'` followed by `:`.
    // `find_map` over the three key forms avoids a nested `if let` in a
    // loop body (clippy::collapsible_match).
    ["", "\"", "'"].iter().find_map(|q| {
        let pat = format!("{}{}:", q, key);
        let idx = obj.find(&pat)?;
        let rest = obj[idx + pat.len()..].trim_start();
        if let Some(value) = rest.strip_prefix('"') {
            read_quoted(value, '"')
        } else if let Some(value) = rest.strip_prefix('\'') {
            read_quoted(value, '\'')
        } else {
            // Object-literal shorthand: `path` (no value)
            None
        }
    })
}

/// Read a quoted string starting immediately after the opening quote char.
/// Returns the raw content (without surrounding quotes), handling escapes.
fn read_quoted(text: &str, quote: char) -> Option<&str> {
    let mut chars = text.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            chars.next();
            continue;
        }
        if c == quote {
            return Some(&text[..i]);
        }
    }
    None
}

/// Skip a single-quoted or double-quoted string body in a `Peekable` char
/// iterator. `open_idx` is the index of the opening quote char (already
/// consumed by the caller). Handles `\` escapes.
pub fn skip_string<I>(chars: &mut std::iter::Peekable<I>, open_idx: usize, quote: char)
where
    I: Iterator<Item = (usize, char)>,
{
    let _ = open_idx;
    while let Some(&(_, c)) = chars.peek() {
        chars.next();
        if c == '\\' {
            chars.next();
        } else if c == quote {
            break;
        }
    }
}

/// Skip a template-literal body in a `Peekable` char iterator.
/// `open_idx` is the index of the opening backtick char (already consumed
/// by the caller). Handles `\` escapes and `${...}` interpolation.
pub fn skip_template<I>(chars: &mut std::iter::Peekable<I>, open_idx: usize)
where
    I: Iterator<Item = (usize, char)> + Clone,
{
    let _ = open_idx;
    while let Some(&(_, c)) = chars.peek() {
        chars.next();
        if c == '\\' {
            chars.next();
        } else if c == '`' {
            break;
        } else if c == '$' {
            // Peek for `{`
            let mut peeked = chars.clone();
            if let Some((_, '{')) = peeked.next() {
                chars.next(); // consume `{`
                // Skip interpolation body tracking braces
                let mut depth = 1i32;
                while let Some(&(_, ic)) = chars.peek() {
                    chars.next();
                    match ic {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        '\\' => {
                            chars.next();
                        }
                        '\'' | '"' | '`' => {
                            skip_string_or_template(chars, ic);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Skip a nested string or template inside `${...}` interpolation,
/// recursively invoking template handling.
fn skip_string_or_template<I>(chars: &mut std::iter::Peekable<I>, quote: char)
where
    I: Iterator<Item = (usize, char)> + Clone,
{
    match quote {
        '\'' | '"' => {
            while let Some(&(_, c)) = chars.peek() {
                chars.next();
                if c == '\\' {
                    chars.next();
                } else if c == quote {
                    break;
                }
            }
        }
        '`' => {
            // Template interpolation with its own ${...}
            while let Some(&(_, c)) = chars.peek() {
                chars.next();
                if c == '\\' {
                    chars.next();
                } else if c == '`' {
                    break;
                } else if c == '$' {
                    let mut peeked = chars.clone();
                    if let Some((_, '{')) = peeked.next() {
                        chars.next();
                        let mut depth = 1i32;
                        while let Some(&(_, ic)) = chars.peek() {
                            chars.next();
                            match ic {
                                '{' => depth += 1,
                                '}' => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                '\\' => {
                                    chars.next();
                                }
                                '\'' | '"' | '`' => {
                                    skip_string_or_template(chars, ic);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Collect a balanced call body starting immediately after the opening `(`.
/// `text` must be the slice starting at the char AFTER the `(`.
///
/// Returns `(body, end_offset)` where `body` is the raw slice (untrimmed)
/// and `end_offset` is the byte offset (relative to `text`) **just past**
/// the closing `)`. Callers MUST advance their scan position by `end_offset`
/// (NOT `body.len()`) to skip the whole call including its close paren.
///
/// String-aware: parens inside strings, template literals, comments and
/// regex literals are ignored. Handles nested parens/braces/brackets.
///
/// # Contract
/// The returned `end_offset` is the offset a caller should add to its
/// "after the opening paren" position to land just past the matching `)`.
/// For an unbalanced body, `end_offset == text.len()` and `body == text`.
pub fn collect_call_body(text: &str) -> (String, usize) {
    let mut depth: i32 = 0;
    let mut chars = text.char_indices().peekable();
    let mut in_string: Option<(char, bool)> = None;
    let mut interp_depth: i32 = 0;

    while let Some((i, c)) = chars.next() {
        if let Some((quote, is_template)) = in_string {
            if is_template && interp_depth > 0 {
                // Inside `${...}` interpolation — treat like code
                match c {
                    '{' => interp_depth += 1,
                    '}' => {
                        interp_depth -= 1;
                        if interp_depth == 0 {
                            in_string = Some(('`', true));
                        }
                    }
                    _ => {
                        if c == quote {
                            in_string = Some(('`', true));
                        }
                    }
                }
            } else if c == '\\' {
                chars.next();
            } else if c == quote {
                in_string = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                in_string = Some((c, c == '`'));
            }
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    // `i` is the char index of `)`; `i + 1` is the byte
                    // offset just past it (the close paren is always 1 byte).
                    return (text[..i].to_string(), i + 1);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    // Unbalanced — return everything.
    (text.to_string(), text.len())
}

/// Collect a balanced call expression starting at the `(` at byte index
/// `open_paren` in `text`.
///
/// Returns `(consumed, arg)` where:
/// - `consumed` is the number of bytes from `open_paren` **just past** the
///   matching `)` (so `open_paren + consumed` lands after the close paren)
/// - `arg` is the raw inner content (between the parens, untrimmed)
///
/// Returns `None` if the call is unterminated (no matching `)`).
///
/// This is a thin adapter over [`collect_call_body`] — the single source of
/// truth for bracket-depth + string-literal awareness. It exists so callers
/// that already hold the index of the `(` (rather than a slice starting after
/// it) can use the same primitive without re-implementing the scanner.
pub fn consume_call_expression(text: &str, open_paren: usize) -> Option<(usize, String)> {
    let after = text.get(open_paren + 1..)?;
    let (arg, end_offset) = collect_call_body(after);
    // Distinguish a genuinely terminated call from an unbalanced one:
    // `collect_call_body` returns the whole remainder when unbalanced,
    // which is indistinguishable from a balanced call whose `)` is the
    // last char (e.g. `()` → after = ")", end_offset = 1). So verify
    // the scan actually stopped at a real `)`.
    if end_offset == 0 || after.as_bytes()[end_offset - 1] != b')' {
        return None;
    }
    // `end_offset` is relative to `after` (the slice starting after `(`).
    // The consumed length from `open_paren` is `1 (the `(`) + end_offset`.
    Some((1 + end_offset, arg))
}

/// Extract the first quoted string from a text (single or double quotes).
///
/// Escape-aware: an escaped quote (`\'` / `\"`) does not terminate the
/// string. The returned value excludes the surrounding quotes.
pub fn extract_first_quoted(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    let (value, quote) = if let Some(stripped) = trimmed.strip_prefix('\'') {
        (stripped, '\'')
    } else if let Some(stripped) = trimmed.strip_prefix('"') {
        (stripped, '"')
    } else {
        return None;
    };
    let mut chars = value.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            chars.next();
            continue;
        }
        if c == quote {
            return Some(value[..i].to_string());
        }
    }
    // Unterminated — return the rest.
    Some(value.to_string())
}

/// Extract the generic type parameter from the start of `text` up to and
/// including the first top-level `>`.
///
/// Accumulates every character from the beginning — this handles the two
/// common call sites:
/// - `EntityState<User>>({ ... })` → `EntityState<User>` (nested generic)
/// - `User>({...})` → `User` (simple generic)
///
/// String-aware: a `>` inside a string literal or template interpolation
/// does NOT terminate the scan (a type object literal like
/// `EntityState<{ tag: 'x>y' }>` is parsed correctly).
pub fn extract_entity_type(text: &str) -> String {
    let mut depth = 0i32;
    let mut result = String::new();
    let mut chars = text.char_indices().peekable();
    let mut in_string: Option<(char, bool)> = None;
    let mut interp_depth: i32 = 0;

    while let Some((_, c)) = chars.next() {
        if let Some((quote, is_template)) = in_string {
            result.push(c);
            if is_template && interp_depth > 0 {
                match c {
                    '{' => interp_depth += 1,
                    '}' => {
                        interp_depth -= 1;
                        if interp_depth == 0 {
                            in_string = Some(('`', true));
                        }
                    }
                    _ => {}
                }
            } else if c == '\\' {
                if let Some(&(_, next)) = chars.peek() {
                    result.push(next);
                    chars.next();
                }
            } else if c == quote {
                in_string = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                in_string = Some((c, c == '`'));
                result.push(c);
            }
            '<' => {
                depth += 1;
                result.push(c);
            }
            '>' => {
                if depth == 0 {
                    // First top-level `>` — done.
                    break;
                }
                depth -= 1;
                result.push(c);
            }
            _ => {
                result.push(c);
            }
        }
    }
    result.trim().to_string()
}

/// Backward-compatible wrapper over [`find_class_source_start`].
///
/// Returns `Some(decorator_start)` when an annotation/attribute group
/// immediately precedes the type declaration at `class_pos`, `None`
/// otherwise (the non-decorated fallback — callers keep their existing
/// declaration-keyword position).
///
/// The canonical trilingual helper [`find_class_source_start`] supersedes
/// the historical TypeScript-only scanning logic; this wrapper preserves
/// the legacy `Option<usize>` contract for
/// `mcp::workspace_util::extract_class_blocks`.
pub fn find_decorator_inclusive_start(source: &str, class_pos: usize) -> Option<usize> {
    let start = find_class_source_start(source, class_pos);
    (start < class_pos).then_some(start)
}

/// Class/modifier keywords used when walking backward across an annotation
/// group. Mirrors the authoritative list in `compaction::modifiers`.
const CLASS_DECLARATION_MODIFIERS: &[&str] = &[
    "export",
    "default",
    "abstract",
    "sealed",
    "public",
    "private",
    "protected",
    "static",
    "final",
];

#[inline]
fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

fn find_matching_open_backward(source: &str, close_pos: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let close = bytes.get(close_pos).copied()?;
    let (open, other_open, other_close) = match close {
        b')' => (b'(', b'[', b']'),
        b']' => (b'[', b'(', b')'),
        _ => return None,
    };
    let mut depth: i32 = 0;
    let mut j = close_pos;
    loop {
        let c = bytes[j];
        if !is_inside_comment_or_string(source, j) {
            if c == close {
                depth += 1;
            } else if c == open {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            } else if c == other_close {
                depth += 1;
            } else if c == other_open {
                depth -= 1;
            }
        }
        if j == 0 {
            break;
        }
        j -= 1;
    }
    None
}

fn skip_backward_trivia(bytes: &[u8], mut i: usize) -> usize {
    loop {
        // Skip whitespace.
        while i > 0 && bytes[i - 1].is_ascii_whitespace() {
            i -= 1;
        }
        // Skip `//` line comment.
        if i >= 2 && bytes[i - 2] == b'/' && bytes[i - 1] == b'/' {
            let mut j = i.saturating_sub(2);
            while j > 0 && bytes[j - 1] != b'\n' {
                j -= 1;
            }
            i = j;
            continue;
        }
        // Skip `/* */` block comment.
        if i >= 2 && bytes[i - 2] == b'*' && bytes[i - 1] == b'/' {
            let mut j = i.saturating_sub(2);
            let mut start = None;
            while j > 0 {
                j -= 1;
                if bytes[j] == b'*' && j > 0 && bytes[j - 1] == b'/' {
                    start = Some(j - 1);
                    break;
                }
            }
            if let Some(start) = start {
                i = start;
                continue;
            }
            break;
        }
        // No whitespace or comment skipped — done.
        break;
    }
    i
}

/// Returns the start byte of the decorator/annotation/attribute group that
/// immediately precedes the type declaration at `type_keyword_pos`, or
/// `type_keyword_pos` itself when no group precedes (non-decorated fallback
/// that preserves existing behavior verbatim).
///
/// This is the CANONICAL trilingual class-source reconstruction helper:
/// `workspace_util::extract_class_blocks` and the text/IR pipelines all
/// delegate here. Supports TS decorators (`@Name(...)`), Java annotations
/// (`@RestController`, `@RequestMapping(...)`) and C# attributes (`[A]`,
/// `[B]`), stacked groups, and declaration modifiers between the group and
/// the type keyword.
pub fn find_class_source_start(source: &str, type_keyword_pos: usize) -> usize {
    if type_keyword_pos > source.len() {
        return type_keyword_pos;
    }
    let bytes = source.as_bytes();
    let mut i = type_keyword_pos;
    let mut found: Option<usize> = None;

    loop {
        i = skip_backward_trivia(bytes, i);
        if i == 0 {
            break;
        }
        let prev = bytes[i - 1];

        // If we encounter a closing brace, we've crossed into the preceding
        // class's closing `}`. Stop immediately — this is a hard class boundary.
        // `}` is never a valid annotation prefix, and any annotations before it
        // belong to the preceding class, not the current one.
        if prev == b'}' {
            break;
        }

        if prev == b'@' || prev == b'[' {
            i -= 1;
            found = Some(i);
            continue;
        }
        if is_ident_byte(prev) {
            let word_end = i;
            let mut k = i;
            while k > 0 && is_ident_byte(bytes[k - 1]) {
                k -= 1;
            }
            let word = &source[k..word_end];
            if CLASS_DECLARATION_MODIFIERS.contains(&word) {
                i = k;
                continue;
            }
            if k > 0 && bytes[k - 1] == b'@' {
                i = k - 1;
                found = Some(i);
                continue;
            }
            break;
        }
        if prev == b')' {
            let Some(open_paren) = find_matching_open_backward(source, i - 1) else {
                break;
            };
            let mut k = open_paren;
            while k > 0 && is_ident_byte(bytes[k - 1]) {
                k -= 1;
            }
            if k > 0 && bytes[k - 1] == b'@' {
                i = k - 1;
                found = Some(i);
                continue;
            }
            break;
        }
        if prev == b']' {
            let Some(open_bracket) = find_matching_open_backward(source, i - 1) else {
                break;
            };
            i = open_bracket;
            found = Some(i);
            continue;
        }
        break;
    }

    found.unwrap_or(type_keyword_pos)
}

/// Locate the absolute byte position of the type declaration keyword within
/// the raw capture span. Java/C# raw captures may begin with annotations
/// (`@RestController` newline `public class`, `[ApiController]` newline
/// `public class`), so the keyword is found by scanning for the first
/// occurrence of a type keyword outside comments/strings.
fn find_type_keyword_in_capture(source: &str, cap: &CapEntry) -> Option<usize> {
    const TYPE_KEYWORDS: &[&str] = &[
        "class ",
        "interface ",
        "struct ",
        "enum ",
        "trait ",
        "record ",
        "impl ",
    ];
    for kw in TYPE_KEYWORDS {
        let mut search_from = 0usize;
        while let Some(rel) = cap.raw_text[search_from..].find(kw) {
            let abs = cap.start_byte.saturating_add(search_from + rel);
            if !is_inside_comment_or_string(source, abs) {
                return Some(abs);
            }
            search_from = search_from + rel + kw.len();
        }
    }
    None
}

/// Canonical class-source text for a type capture. Returns
/// `source[start .. cap.start_byte + cap.raw_text.len()]` where `start` is
/// the leading decorator/annotation/attribute byte (or the declaration
/// keyword byte when no group precedes — the non-decorated fallback). The
/// slice includes the declaration keyword through the closing `}` because
/// the meta-layer extractors scan the class body for method/field-level
/// annotations.
pub fn class_source_from_capture<'a>(source: &'a str, cap: &'a CapEntry) -> &'a str {
    let cap_end = cap
        .start_byte
        .saturating_add(cap.raw_text.len())
        .min(source.len());
    if cap.start_byte >= source.len() {
        return &source[source.len()..source.len()];
    }
    let type_pos = find_type_keyword_in_capture(source, cap).unwrap_or(cap.start_byte);
    let start = find_class_source_start(source, type_pos).min(cap_end);
    &source[start..cap_end]
}

/// Returns `true` if byte index `pos` in `source` lies inside a comment
/// (`//` line comment or `/* */` block comment) or a string/template
/// literal.
///
/// Used by the meta-layer extractors to reject pattern matches that occur
/// inside comments or strings — the defect class where a global
/// `.find(pattern)` scan picks up phantom artifacts (e.g. a `path:` key
/// inside a trailing `// path: 'x'` comment, or a `combineLatest(` inside
/// a string literal). Round-11 audit.
///
/// `pos` must be a valid byte index into `source` (i.e. `pos <= source.len()`).
/// The opening quote/comment marker itself is NOT considered "inside" —
/// only characters after it are.
pub fn is_inside_comment_or_string(source: &str, pos: usize) -> bool {
    let bytes = source.as_bytes();
    let mut i = 0usize;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string: Option<(char, bool)> = None; // (quote, is_template)
    let mut interp_depth: i32 = 0;

    while i < pos && i < bytes.len() {
        let c = bytes[i] as char;
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == '*' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if let Some((quote, is_template)) = in_string {
            if is_template && interp_depth > 0 {
                // Inside `${...}` interpolation — treat like code.
                match c {
                    '{' => interp_depth += 1,
                    '}' => {
                        interp_depth -= 1;
                        if interp_depth == 0 {
                            in_string = Some(('`', true));
                        }
                    }
                    _ => {
                        if c == quote {
                            in_string = Some(('`', true));
                        }
                    }
                }
            } else if c == '\\' {
                i += 2; // skip escaped char
                continue;
            } else if c == quote {
                in_string = None;
            }
            i += 1;
            continue;
        }
        match c {
            '/' if i + 1 < bytes.len() && bytes[i + 1] as char == '/' => {
                in_line_comment = true;
                i += 2;
                continue;
            }
            '/' if i + 1 < bytes.len() && bytes[i + 1] as char == '*' => {
                in_block_comment = true;
                i += 2;
                continue;
            }
            '\'' | '"' | '`' => {
                in_string = Some((c, c == '`'));
            }
            _ => {}
        }
        i += 1;
    }
    in_line_comment || in_block_comment || in_string.is_some()
}

#[cfg(test)]
#[path = "tests/meta_util.rs"]
mod tests;

/// Extract a declarator/assignment name from the text BEFORE the value
/// expression. Handles:
/// - simple identifiers: `count`
/// - member expressions: `this.count`, `obj.x.y` → `y`
/// - optional chaining: `user?.name` → `name`
/// - trailing `=` or `:` before the value: `count =`, `count:`
///
/// Returns `None` when no valid identifier is found.
pub fn extract_decl_name(before: &str) -> Option<String> {
    let mut tokens: Vec<&str> = before.split_whitespace().collect();
    while let Some(last) = tokens.last() {
        let last = last.trim_end_matches('=').trim();
        if last.is_empty() || last == "=" {
            tokens.pop();
            continue;
        }
        let last = last.trim_end_matches(':').trim();
        if last.is_empty() || last == ":" {
            tokens.pop();
            continue;
        }
        // Strip optional chaining and member access to the final segment.
        let last = last
            .rsplit('.')
            .next()
            .unwrap_or(last)
            .trim_end_matches('?');
        if last.is_empty() {
            tokens.pop();
            continue;
        }
        // Must be a valid identifier-ish token. If not, this is a bare
        // statement (e.g. `{` before a constructor `effect()` call) — NOT
        // an assignment. Do NOT keep walking backward past the invalid
        // punctuation to pick up an enclosing class name; return `None`
        // so the caller renders `?` (matches the original signals layer
        // semantics — Round-8 audit regression guard).
        if !last
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        {
            return None;
        }
        return Some(last.to_string());
    }
    None
}
