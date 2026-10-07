use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines_with_tail},
};

const POLICY: LineFilterPolicy = LineFilterPolicy {
    filter_id: "maven-build-v1",
    disclosure_label: "maven-build",
    max_output_lines: 50,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MavenOperation {
    Compile,
    Package,
    Install,
}

pub fn filter_maven_diagnostics(text: &str, _operation: MavenOperation) -> FilteredText {
    filter_lines_with_tail(
        text,
        POLICY,
        |line| {
            line.trim().is_empty()
                || line.starts_with("[INFO] ---")
                || line.starts_with("[INFO] Downloading")
                || line.starts_with("[INFO] Downloaded")
                || line.starts_with("[INFO] Progress")
                || is_module_progress(line)
                || line.starts_with("[INFO] Building ")
        },
        |line| {
            line.starts_with("[INFO] BUILD ")
                || line.starts_with("[INFO] Total time:")
                || line.starts_with("[INFO] Finished at:")
        },
    )
}

fn is_module_progress(line: &str) -> bool {
    let Some(progress) = line.strip_prefix("[INFO] [") else {
        return false;
    };
    let Some((position, _)) = progress.split_once(']') else {
        return false;
    };
    let Some((current, total)) = position.split_once('/') else {
        return false;
    };
    !current.is_empty()
        && !total.is_empty()
        && current.chars().all(|character| character.is_ascii_digit())
        && total.chars().all(|character| character.is_ascii_digit())
}

#[cfg(test)]
#[path = "../tests/native_text/maven_filter.rs"]
mod tests;
