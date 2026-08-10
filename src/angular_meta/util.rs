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
pub fn collect_call_body(text: &str) -> (String, usize) {
    let mut depth = 0;
    let mut body = String::new();
    let mut end = 0;
    for (i, ch) in text.chars().enumerate() {
        match ch {
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