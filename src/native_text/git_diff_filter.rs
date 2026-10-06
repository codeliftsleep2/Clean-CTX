use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

const FILTER_ID: &str = "git-diff-v1";
const DISCLOSURE_PREFIX: &str = "§FILTERED git-diff:";
const MAX_OUTPUT_LINES: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReductionKind {
    NoiseRemoval,
    Truncation,
    NoiseRemovalAndTruncation,
    AlreadyFiltered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FilterFacts {
    pub filter_id: &'static str,
    pub original_bytes: usize,
    pub reduced_bytes: usize,
    pub original_lines: usize,
    pub reduced_lines: usize,
    pub reduction_kind: ReductionKind,
    pub truncated: bool,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredText {
    pub text: String,
    pub facts: Option<FilterFacts>,
}

pub fn command_selects_git_diff(command: &str) -> bool {
    static COMMAND: OnceLock<Regex> = OnceLock::new();
    COMMAND
        .get_or_init(|| {
            Regex::new(r"^git\s+(diff|show)\b").expect("git command regex must compile")
        })
        .is_match(command.trim_start())
}

pub fn filter_git_diff(text: &str) -> FilteredText {
    if text.lines().any(|line| line.starts_with(DISCLOSURE_PREFIX)) {
        return FilteredText {
            text: text.to_owned(),
            facts: None,
        };
    }
    static INDEX: OnceLock<Regex> = OnceLock::new();
    let index = INDEX
        .get_or_init(|| Regex::new(r"^index [0-9a-f]{7,}").expect("git index regex must compile"));
    let original_lines = text.lines().count();
    let mut lines: Vec<&str> = text
        .lines()
        .filter(|line| {
            !index.is_match(line)
                && *line != "\\ No newline at end of file"
                && !line.starts_with("Binary files ")
        })
        .collect();
    let noise_removed = lines.len() != original_lines;
    let truncated = lines.len() + 1 > MAX_OUTPUT_LINES;
    if truncated {
        lines.truncate(MAX_OUTPUT_LINES - 1);
    }
    if !noise_removed && !truncated {
        return FilteredText {
            text: text.to_owned(),
            facts: None,
        };
    }
    let content_lines = lines.len();
    let kind = match (noise_removed, truncated) {
        (true, true) => ReductionKind::NoiseRemovalAndTruncation,
        (true, false) => ReductionKind::NoiseRemoval,
        (false, true) => ReductionKind::Truncation,
        (false, false) => unreachable!(),
    };
    lines.push("");
    let mut output = lines.join("\n");
    let marker = format!("{DISCLOSURE_PREFIX} {original_lines} → {content_lines} lines");
    output.push_str(&marker);
    let reduced_lines = output.lines().count();
    let facts = FilterFacts {
        filter_id: FILTER_ID,
        original_bytes: text.len(),
        reduced_bytes: output.len(),
        original_lines,
        reduced_lines,
        reduction_kind: kind,
        truncated,
        collapsed: false,
    };
    FilteredText {
        text: output,
        facts: Some(facts),
    }
}

#[cfg(test)]
#[path = "../tests/native_text/git_diff_filter.rs"]
mod tests;
