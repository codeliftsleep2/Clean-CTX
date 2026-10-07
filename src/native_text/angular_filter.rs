use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines_with_tail},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngularOperation {
    Build,
    Test,
    Lint,
}

impl AngularOperation {
    fn filter_id(self) -> &'static str {
        match self {
            Self::Build => "angular-build-v1",
            Self::Test => "angular-test-v1",
            Self::Lint => "angular-lint-v1",
        }
    }

    fn disclosure_label(self) -> &'static str {
        match self {
            Self::Build => "angular-build",
            Self::Test => "angular-test",
            Self::Lint => "angular-lint",
        }
    }
}

pub fn filter_angular_diagnostics(text: &str, operation: AngularOperation) -> FilteredText {
    filter_lines_with_tail(
        text,
        LineFilterPolicy {
            filter_id: operation.filter_id(),
            disclosure_label: operation.disclosure_label(),
            max_output_lines: 80,
        },
        |line| {
            line.trim().is_empty()
                || line.starts_with("Browser application bundle")
                || line.starts_with("Generating browser")
                || line.starts_with("Processing assets")
                || line.starts_with("Output location:")
        },
        |line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("Build succeeded")
                || trimmed.starts_with("Build failed")
                || (trimmed.starts_with("Executed ")
                    && (trimmed.contains("SUCCESS") || trimmed.contains("FAILED")))
                || trimmed.starts_with("All files pass linting")
        },
    )
}

#[cfg(test)]
#[path = "../tests/native_text/angular_filter.rs"]
mod tests;
