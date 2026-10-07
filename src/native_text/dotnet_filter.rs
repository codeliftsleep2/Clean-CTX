use super::{
    filter_facts::FilteredText,
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
