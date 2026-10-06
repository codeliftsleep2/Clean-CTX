use super::filter_facts::{FilterFacts, FilteredText, ReductionKind};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LineFilterPolicy {
    pub filter_id: &'static str,
    pub disclosure_label: &'static str,
    pub max_output_lines: usize,
}

pub(crate) fn filter_lines(
    text: &str,
    policy: LineFilterPolicy,
    mut remove_line: impl FnMut(&str) -> bool,
) -> FilteredText {
    let disclosure_prefix = format!("§FILTERED {}:", policy.disclosure_label);
    if text
        .lines()
        .any(|line| line.starts_with(&disclosure_prefix))
    {
        return unchanged(text);
    }

    let original_lines = text.lines().count();
    let mut lines: Vec<&str> = text.lines().filter(|line| !remove_line(line)).collect();
    let noise_removed = lines.len() != original_lines;
    let truncated = lines.len() + 1 > policy.max_output_lines;
    if truncated {
        lines.truncate(policy.max_output_lines - 1);
    }
    if !noise_removed && !truncated {
        return unchanged(text);
    }

    let content_lines = lines.len();
    let reduction_kind = match (noise_removed, truncated) {
        (true, true) => ReductionKind::NoiseRemovalAndTruncation,
        (true, false) => ReductionKind::NoiseRemoval,
        (false, true) => ReductionKind::Truncation,
        (false, false) => unreachable!(),
    };
    lines.push("");
    let mut output = lines.join("\n");
    output.push_str(&format!(
        "{disclosure_prefix} {original_lines} → {content_lines} lines"
    ));
    let facts = FilterFacts {
        filter_id: policy.filter_id,
        original_bytes: text.len(),
        reduced_bytes: output.len(),
        original_lines,
        reduced_lines: output.lines().count(),
        reduction_kind,
        truncated,
        collapsed: false,
    };
    FilteredText {
        text: output,
        facts: Some(facts),
    }
}

fn unchanged(text: &str) -> FilteredText {
    FilteredText {
        text: text.to_owned(),
        facts: None,
    }
}

#[cfg(test)]
#[path = "../tests/native_text/line_filter.rs"]
mod tests;
