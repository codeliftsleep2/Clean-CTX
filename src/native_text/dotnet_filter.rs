use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines},
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
    filter_lines(text, operation.policy(), |line| {
        let trimmed = line.trim_start();
        line.trim().is_empty()
            || line.starts_with("Microsoft (R)")
            || line.starts_with("Copyright (C)")
            || trimmed.starts_with("Determining")
            || trimmed.starts_with("Restoring")
            || line.starts_with("Build started")
            || line.starts_with("Time Elapsed")
    })
}

#[cfg(test)]
#[path = "../tests/native_text/dotnet_filter.rs"]
mod tests;
