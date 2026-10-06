use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines},
};

const POLICY: LineFilterPolicy = LineFilterPolicy {
    filter_id: "eslint-v1",
    disclosure_label: "eslint",
    max_output_lines: 140,
};

pub fn filter_eslint_diagnostics(text: &str) -> FilteredText {
    filter_lines(text, POLICY, |line| line.trim().is_empty())
}

#[cfg(test)]
#[path = "../tests/native_text/eslint_filter.rs"]
mod tests;
