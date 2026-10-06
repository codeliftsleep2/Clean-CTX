use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct NormalizationFacts {
    pub sequences_removed: usize,
    pub bytes_removed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedText {
    pub text: String,
    pub facts: NormalizationFacts,
}

pub fn normalize_terminal_text(text: &str) -> NormalizedText {
    static ANSI: OnceLock<Regex> = OnceLock::new();
    let regex = ANSI.get_or_init(|| {
        Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]").expect("native ANSI regex must compile")
    });
    let sequences_removed = regex.find_iter(text).count();
    let normalized = regex.replace_all(text, "").into_owned();
    NormalizedText {
        facts: NormalizationFacts {
            sequences_removed,
            bytes_removed: text.len().saturating_sub(normalized.len()),
        },
        text: normalized,
    }
}

#[cfg(test)]
#[path = "../tests/native_text/ansi.rs"]
mod tests;
