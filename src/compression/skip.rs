// src/compression/skip.rs
//
// CBM filter-first capture skipping. Decides whether a tree-sitter capture
// (class/method/field) is excluded from the output because its name is in the
// caller-supplied skip set. Extracted from `pipeline.rs` at a structural
// boundary (active-file size rule).

use std::collections::HashSet;

/// Check if a capture should be skipped due to CBM filter-first rules.
pub(crate) fn should_skip_capture(
    cap: &crate::compression::CapEntry,
    skip_set: &HashSet<String>,
) -> bool {
    if matches!(
        cap.name.as_str(),
        "class.root"
            | "struct.root"
            | "enum.root"
            | "trait.root"
            | "impl.root"
            | "interface.root"
            | "record.root"
    ) {
        return skip_set.contains(cap.text.trim());
    }

    // C-8 (FAANG audit): at Edit/Verbatim `cap.text` is the FULL method body
    // (e.g. "public async getUser..."), so the first word is the access
    // modifier, not the method name. Extract the actual method name by
    // scanning for the identifier that precedes the first `(`.
    if matches!(
        cap.name.as_str(),
        "method.root" | "constructor.root" | "func.root" | "arrow.root"
    ) {
        if let Some(name) = extract_method_name_for_skip(&cap.text) {
            return skip_set.contains(name);
        }
        return skip_set.contains(cap.text.trim());
    }

    // C-8: at Edit/Verbatim `cap.text` is the full field text
    // (e.g. "private readonly userId: string = '';"). The `:` split still
    // yields the leading modifiers + name, so we take the LAST whitespace
    // token before the `:` to get the actual field name.
    if cap.name == "field.root" {
        let before_colon = cap.text.split(':').next().unwrap_or(cap.text.as_str());
        let field_name = before_colon
            .split_whitespace()
            .last()
            .unwrap_or(before_colon);
        return skip_set.contains(field_name.trim());
    }

    false
}

/// Extract the method name from a method capture's text for CBM skip-set
/// matching. At Edit/Verbatim the text is the full method body, so we scan
/// for the identifier immediately preceding the first `(` (the method name).
/// At lower fidelities the text is already the compact signature, so the
/// same scan works. Returns `None` if no `(` is found.
fn extract_method_name_for_skip(text: &str) -> Option<&str> {
    let open = text.find('(')?;
    let before = &text[..open];
    // Take the last whitespace-delimited token before the `(`.
    // Handles "public async getUser" → "getUser", "getUser" → "getUser",
    // "async getUser<T>" → "getUser<T>".
    let name = before.split_whitespace().last()?;
    // Strip generic parameters for matching (skip sets use bare names).
    let bare = name.split('<').next().unwrap_or(name);
    Some(bare.trim())
}
