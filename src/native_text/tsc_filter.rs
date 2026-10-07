use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines_with_tail},
};

const POLICY: LineFilterPolicy = LineFilterPolicy {
    filter_id: "tsc-v1",
    disclosure_label: "tsc",
    max_output_lines: 100,
};

pub fn filter_tsc_diagnostics(text: &str) -> FilteredText {
    filter_lines_with_tail(
        text,
        POLICY,
        |line| {
            let trimmed = line.trim();
            (!trimmed.is_empty() && trimmed.chars().all(|character| character == '~'))
                || line.starts_with("Version")
        },
        |line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("Found ")
                && (trimmed.contains(" error") || trimmed.contains(" errors"))
        },
    )
}

#[cfg(test)]
#[path = "../tests/native_text/tsc_filter.rs"]
mod tests;
