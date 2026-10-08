use super::model::TransformationFacts;
use crate::native_text::{ansi::normalize_terminal_text, redaction::redact_recognized_secrets};

pub(crate) fn sanitize(value: &str, facts: &mut TransformationFacts) -> String {
    sanitize_in_workspace(value, None, facts)
}

pub(crate) fn sanitize_in_workspace(
    value: &str,
    workspace: Option<&super::ApprovedWorkspaceRoot>,
    facts: &mut TransformationFacts,
) -> String {
    let normalized = normalize_terminal_text(value);
    facts.normalization.sequences_removed += normalized.facts.sequences_removed;
    facts.normalization.bytes_removed += normalized.facts.bytes_removed;

    let displayed = workspace.map(|workspace| {
        workspace_text(
            &normalized.text,
            &workspace.canonical_root().to_string_lossy(),
            facts,
            cfg!(windows),
        )
    });
    let redacted = redact_recognized_secrets(displayed.as_deref().unwrap_or(&normalized.text));
    for (class, count) in redacted.facts.counts {
        *facts.redaction.counts.entry(class).or_insert(0) += count;
    }
    redacted.text
}

fn workspace_text(
    value: &str,
    root: &str,
    facts: &mut TransformationFacts,
    windows_paths: bool,
) -> String {
    let mut variants = vec![root.to_owned()];
    if windows_paths {
        let plain = if let Some(unc) = root.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{unc}")
        } else {
            root.strip_prefix(r"\\?\").unwrap_or(root).to_owned()
        };
        variants.extend([root.replace('\\', "/"), plain.replace('\\', "/"), plain]);
    }
    variants.sort_by_key(|variant| std::cmp::Reverse(variant.len()));
    variants.dedup();
    let mut displayed = value.to_owned();
    for variant in variants {
        if variant.is_empty() {
            continue;
        }
        let searched = if windows_paths {
            displayed.to_ascii_lowercase()
        } else {
            displayed.clone()
        };
        let needle = if windows_paths {
            variant.to_ascii_lowercase()
        } else {
            variant
        };
        let mut mapped = String::new();
        let mut cursor = 0;
        for (start, _) in searched.match_indices(&needle) {
            let end = start + needle.len();
            // Never rewrite an unrelated directory that merely shares this prefix.
            if displayed[end..].chars().next().is_some_and(|next| {
                !matches!(
                    next,
                    '/' | '\\' | ':' | ')' | ']' | '}' | '\'' | '"' | ',' | ';'
                ) && !next.is_whitespace()
            }) {
                continue;
            }
            mapped.push_str(&displayed[cursor..start]);
            mapped.push_str("<workspace>");
            cursor = end;
            facts.workspace_paths_mapped = facts.workspace_paths_mapped.saturating_add(1);
        }
        mapped.push_str(&displayed[cursor..]);
        displayed = mapped;
    }
    displayed
}

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_workspace_spellings.rs"]
mod tests;
