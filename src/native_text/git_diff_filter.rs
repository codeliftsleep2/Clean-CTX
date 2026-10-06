pub use super::filter_facts::{FilterFacts, FilteredText, ReductionKind};
use super::line_filter::{LineFilterPolicy, filter_lines};
use regex::Regex;
use std::sync::OnceLock;

const FILTER_ID: &str = "git-diff-v1";
const MAX_OUTPUT_LINES: usize = 500;

pub fn filter_git_diff(text: &str) -> FilteredText {
    static INDEX: OnceLock<Regex> = OnceLock::new();
    let index = INDEX
        .get_or_init(|| Regex::new(r"^index [0-9a-f]{7,}").expect("git index regex must compile"));
    filter_lines(
        text,
        LineFilterPolicy {
            filter_id: FILTER_ID,
            disclosure_label: "git-diff",
            max_output_lines: MAX_OUTPUT_LINES,
        },
        |line| {
            index.is_match(line)
                || line == "\\ No newline at end of file"
                || line.starts_with("Binary files ")
        },
    )
}

#[cfg(test)]
#[path = "../tests/native_text/git_diff_filter.rs"]
mod tests;
