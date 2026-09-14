//! Class capture span and decorator/annotation reconstruction utilities.

use super::is_inside_comment_or_string;
use crate::compression::capture_pipeline::CapEntry;

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
