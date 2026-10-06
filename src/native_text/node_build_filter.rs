use super::{
    filter_facts::FilteredText,
    line_filter::{LineFilterPolicy, filter_lines},
};

const POLICY: LineFilterPolicy = LineFilterPolicy {
    filter_id: "node-build-v1",
    disclosure_label: "node-build",
    max_output_lines: 120,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeBuildOperation {
    Build,
    Compile,
    Bundle,
}

pub fn filter_node_build_output(text: &str, _operation: NodeBuildOperation) -> FilteredText {
    filter_lines(text, POLICY, |line| line.trim().is_empty())
}

#[cfg(test)]
#[path = "../tests/native_text/node_build_filter.rs"]
mod tests;
