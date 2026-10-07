use super::{
    filter_facts::{FilterFacts, FilteredText, ReductionKind},
    line_filter::{LineFilterPolicy, filter_lines_with_tail},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DotnetOperation {
    Build,
    Test,
}

impl DotnetOperation {
    fn policy(self) -> LineFilterPolicy {
        match self {
            Self::Build => LineFilterPolicy {
                filter_id: "dotnet-build-v1",
                disclosure_label: "dotnet-build",
                max_output_lines: 40,
            },
            Self::Test => LineFilterPolicy {
                filter_id: "dotnet-test-v1",
                disclosure_label: "dotnet-test",
                max_output_lines: 100,
            },
        }
    }
}

pub fn filter_dotnet_diagnostics(text: &str, operation: DotnetOperation) -> FilteredText {
    if let (DotnetOperation::Test, Some(collapsed)) =
        (operation, collapse_clean_successful_test(text))
    {
        return collapsed;
    }
    filter_lines_with_tail(
        text,
        operation.policy(),
        |line| {
            let trimmed = line.trim_start();
            line.trim().is_empty()
                || line.starts_with("Microsoft (R)")
                || line.starts_with("Copyright (C)")
                || trimmed.starts_with("Determining")
                || trimmed.starts_with("Restoring")
                || line.starts_with("Build started")
        },
        |line| operation.must_keep_tail(line),
    )
}

#[derive(Debug, PartialEq, Eq)]
struct TestTotals<'a> {
    passed: usize,
    failed: usize,
    skipped: usize,
    total: usize,
    duration: &'a str,
}

fn collapse_clean_successful_test(text: &str) -> Option<FilteredText> {
    if text
        .lines()
        .any(|line| line.starts_with("§FILTERED dotnet-test:"))
        || text.to_ascii_lowercase().contains("warning")
    {
        return None;
    }
    let totals = parse_inline_totals(text).or_else(|| parse_totals_block(text))?;
    if totals.total == 0
        || totals.passed != totals.total
        || totals.failed != 0
        || totals.skipped != 0
    {
        return None;
    }

    let original_lines = text.lines().count();
    let output = format!(
        "§FILTERED dotnet-test: {} passed, 0 failed, 0 skipped, {} ({original_lines} → 1 lines)",
        totals.passed, totals.duration
    );
    Some(FilteredText {
        facts: Some(FilterFacts {
            filter_id: "dotnet-test-v1",
            original_bytes: text.len(),
            reduced_bytes: output.len(),
            original_lines,
            reduced_lines: 1,
            reduction_kind: ReductionKind::SuccessCollapse,
            truncated: false,
            collapsed: true,
        }),
        text: output,
    })
}

fn parse_inline_totals(text: &str) -> Option<TestTotals<'_>> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with("Passed!") || !is_vstest_totals_summary(trimmed) {
            return None;
        }
        Some(TestTotals {
            failed: parse_number_after(trimmed, "Failed:")?,
            passed: parse_number_after(trimmed, "Passed:")?,
            skipped: parse_number_after(trimmed, "Skipped:")?,
            total: parse_number_after(trimmed, "Total:")?,
            duration: value_after(trimmed, "Duration:")?,
        })
    })
}

fn parse_totals_block(text: &str) -> Option<TestTotals<'_>> {
    if !text
        .lines()
        .any(|line| line.trim().starts_with("Test Run Passed"))
    {
        return None;
    }
    let total = find_line_number(text, "Total tests:")?;
    let passed = find_line_number(text, "Passed:")?;
    Some(TestTotals {
        passed,
        failed: find_line_number(text, "Failed:").unwrap_or(0),
        skipped: find_line_number(text, "Skipped:").unwrap_or(0),
        total,
        duration: text
            .lines()
            .find_map(|line| value_after(line.trim(), "Total time:"))?,
    })
}

fn find_line_number(text: &str, label: &str) -> Option<usize> {
    text.lines()
        .find_map(|line| parse_number_after(line.trim(), label))
}

fn parse_number_after(text: &str, label: &str) -> Option<usize> {
    value_after(text, label)?
        .split(|character: char| !character.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

fn value_after<'a>(text: &'a str, label: &str) -> Option<&'a str> {
    let value = text.split_once(label)?.1.trim();
    (!value.is_empty()).then_some(value)
}

impl DotnetOperation {
    fn must_keep_tail(self, line: &str) -> bool {
        let trimmed = line.trim();
        match self {
            Self::Build => {
                trimmed.starts_with("Build succeeded")
                    || trimmed.starts_with("Build FAILED")
                    || trimmed.ends_with("Warning(s)")
                    || trimmed.ends_with("Error(s)")
                    || trimmed.starts_with("Time Elapsed")
            }
            Self::Test => {
                is_vstest_totals_summary(trimmed)
                    || trimmed.starts_with("Test Run Passed")
                    || trimmed.starts_with("Test Run Failed")
                    || trimmed.starts_with("Total tests:")
                    || trimmed.starts_with("Passed:")
                    || trimmed.starts_with("Failed:")
                    || trimmed.starts_with("Skipped:")
                    || trimmed.starts_with("Total time:")
            }
        }
    }
}

fn is_vstest_totals_summary(line: &str) -> bool {
    let mut words = line.split_whitespace();
    matches!(words.next(), Some("Passed!" | "Failed!"))
        && words.next() == Some("-")
        && words.next() == Some("Failed:")
}

#[cfg(test)]
#[path = "../tests/native_text/dotnet_filter.rs"]
mod tests;
