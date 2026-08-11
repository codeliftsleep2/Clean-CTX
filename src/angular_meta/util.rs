// src/angular_meta/util.rs
//
// Shared string-scanning helpers for the Angular Ecosystem Deepening
// sub-layers (RxJS, NgRx, Routing).
//
// # Why this module exists
//
// `collect_call_body` was previously duplicated verbatim in `rx.rs` and
// `ngrx.rs`, and bracket-depth scanning was re-invented independently in
// rx, ngrx, and routing. Centralizing these primitives here avoids
// copy-paste drift and makes the extraction logic testable in isolation.

/// Collect the full body of a call expression (up to the matching
/// close paren). Returns `(body, end_offset)`.
///
/// `text` is the slice **after** the opening `(`. The returned `body`
/// includes the nested parens but not the outer `(`; `end_offset` is the
/// index of the matching `)` in `text`.
///
/// The scan is **string-aware**: `(`/`)` inside single-quoted, double-quoted,
/// or backtick template literals are treated as literal characters and do
/// NOT affect bracket depth. This prevents a string like
/// `console.log('foo(bar)')` from prematurely closing the call body.
pub fn collect_call_body(text: &str) -> (String, usize) {
    let mut depth = 0;
    let mut body = String::new();
    let mut end = 0;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_template = false;
    let mut chars = text.chars().enumerate().peekable();

    while let Some((i, ch)) = chars.next() {
        // String-literal awareness: once inside a quote, only the matching
        // unescaped quote (or backtick for templates) exits it.
        if in_single {
            body.push(ch);
            if ch == '\\' {
                // Skip the escaped char.
                if let Some(&(_, next)) = chars.peek() {
                    body.push(next);
                    chars.next();
                }
            } else if ch == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            body.push(ch);
            if ch == '\\' {
                if let Some(&(_, next)) = chars.peek() {
                    body.push(next);
                    chars.next();
                }
            } else if ch == '"' {
                in_double = false;
            }
            continue;
        }
        if in_template {
            body.push(ch);
            if ch == '\\' {
                if let Some(&(_, next)) = chars.peek() {
                    body.push(next);
                    chars.next();
                }
            } else if ch == '`' {
                in_template = false;
            }
            continue;
        }

        match ch {
            '\'' => { in_single = true; body.push(ch); }
            '"' => { in_double = true; body.push(ch); }
            '`' => { in_template = true; body.push(ch); }
            '(' => { depth += 1; body.push(ch); }
            ')' => {
                if depth == 0 {
                    end = i;
                    break;
                }
                depth -= 1;
                body.push(ch);
            }
            _ => body.push(ch),
        }
    }
    (body, end)
}

/// Extract the first quoted string from a text (single or double quotes).
pub fn extract_first_quoted(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    if let Some(stripped) = trimmed.strip_prefix('\'') {
        stripped.split('\'').next().map(|s| s.to_string())
    } else if let Some(stripped) = trimmed.strip_prefix('"') {
        stripped.split('"').next().map(|s| s.to_string())
    } else {
        None
    }
}

/// Extract the generic type parameter from between `<` and `>` using
/// bracket-depth tracking (handles nested generics like `EntityState<User>`).
pub fn extract_entity_type(text: &str) -> String {
    let mut depth = 0;
    let mut result = String::new();
    for ch in text.chars() {
        match ch {
            '<' => { depth += 1; result.push(ch); }
            '>' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                result.push(ch);
            }
            _ => {
                result.push(ch);
            }
        }
    }
    result.trim().to_string()
}

/// Split `text` on `delim` at the top nesting level only.
///
/// Nested `()`, `[]`, `{}`, and string/template literals are treated as
/// atomic — a `delim` inside them does NOT split. This is the correct
/// primitive for splitting call arguments (e.g. `createSelector` inputs,
/// `combineLatest` args) where a projection function or object literal
/// may legitimately contain commas.
///
/// Returns the trimmed, non-empty top-level segments.
pub fn split_top_level(text: &str, delim: char) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut paren = 0i32;
    let mut bracket = 0i32;
    let mut brace = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut in_template = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        // String-literal awareness: once inside a quote, only the matching
        // unescaped quote (or backtick for templates) exits it.
        if in_single {
            current.push(ch);
            if ch == '\\' {
                // Skip the escaped char.
                if let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                }
            } else if ch == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            current.push(ch);
            if ch == '\\' {
                if let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                }
            } else if ch == '"' {
                in_double = false;
            }
            continue;
        }
        if in_template {
            current.push(ch);
            if ch == '\\' {
                if let Some(&next) = chars.peek() {
                    current.push(next);
                    chars.next();
                }
            } else if ch == '`' {
                in_template = false;
            }
            continue;
        }

        match ch {
            '\'' => { in_single = true; current.push(ch); }
            '"' => { in_double = true; current.push(ch); }
            '`' => { in_template = true; current.push(ch); }
            '(' => { paren += 1; current.push(ch); }
            ')' => { paren -= 1; current.push(ch); }
            '[' => { bracket += 1; current.push(ch); }
            ']' => { bracket -= 1; current.push(ch); }
            '{' => { brace += 1; current.push(ch); }
            '}' => { brace -= 1; current.push(ch); }
            c if c == delim && paren == 0 && bracket == 0 && brace == 0 => {
                let seg = current.trim().to_string();
                if !seg.is_empty() {
                    segments.push(seg);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let last = current.trim().to_string();
    if !last.is_empty() {
        segments.push(last);
    }
    segments
}

#[cfg(test)]
#[path = "../tests/angular_meta/util.rs"]
mod tests;
