use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines_with_tail},
};

const MAX_OUTPUT_LINES: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CargoOperation {
    Build,
    Check,
    Clippy,
}

impl CargoOperation {
    fn filter_id(self) -> &'static str {
        match self {
            Self::Build => "cargo-build-v1",
            Self::Check => "cargo-check-v1",
            Self::Clippy => "cargo-clippy-v1",
        }
    }

    fn disclosure_label(self) -> &'static str {
        match self {
            Self::Build => "cargo-build",
            Self::Check => "cargo-check",
            Self::Clippy => "cargo-clippy",
        }
    }
}

pub fn filter_cargo_diagnostics(text: &str, operation: CargoOperation) -> FilteredText {
    filter_lines_with_tail(
        text,
        LineFilterPolicy {
            filter_id: operation.filter_id(),
            disclosure_label: operation.disclosure_label(),
            max_output_lines: MAX_OUTPUT_LINES,
        },
        is_progress_noise,
        |line| line.trim_start().starts_with("Finished "),
    )
}

fn is_progress_noise(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty()
        || trimmed.starts_with("Compiling ")
        || trimmed.starts_with("Checking ")
        || trimmed.starts_with("Downloading ")
        || trimmed.starts_with("Downloaded ")
        || trimmed.starts_with("Fresh ")
}

#[cfg(test)]
#[path = "../tests/native_text/cargo_filter.rs"]
mod tests;
