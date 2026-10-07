use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines_with_tail},
};

const POLICY: LineFilterPolicy = LineFilterPolicy {
    filter_id: "eslint-v1",
    disclosure_label: "eslint",
    max_output_lines: 140,
};

pub fn filter_eslint_diagnostics(text: &str) -> FilteredText {
    filter_lines_with_tail(
        text,
        POLICY,
        |line| line.trim().is_empty(),
        |line| {
            let trimmed = line.trim();
            (trimmed.starts_with('✖') && trimmed.contains(" problem"))
                || (trimmed.contains(" problem")
                    && trimmed.contains(" error")
                    && trimmed.contains(" warning"))
                || trimmed.contains("potentially fixable with the `--fix` option")
        },
    )
}

#[cfg(test)]
#[path = "../tests/native_text/eslint_filter.rs"]
mod tests;
