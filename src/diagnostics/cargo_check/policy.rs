//! Owner-approved CargoCheck operational limits.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CargoCheckPolicy {
    pub stdout_capture_bytes: usize,
    pub stderr_capture_bytes: usize,
    pub aggregate_capture_bytes: usize,
    pub maximum_frame_bytes: usize,
    pub maximum_diagnostics: usize,
    pub maximum_primary_spans: usize,
    pub maximum_related_spans: usize,
    pub maximum_children: usize,
    pub maximum_suggestions: usize,
    pub evidence_bytes: usize,
    pub structured_content_bytes: usize,
    pub content_bytes: usize,
    pub content_lines: usize,
}

impl CargoCheckPolicy {
    pub const APPROVED: Self = Self {
        stdout_capture_bytes: 8 * 1024 * 1024,
        stderr_capture_bytes: 4 * 1024 * 1024,
        aggregate_capture_bytes: 12 * 1024 * 1024,
        maximum_frame_bytes: 2 * 1024 * 1024,
        maximum_diagnostics: 64,
        maximum_primary_spans: 4,
        maximum_related_spans: 8,
        maximum_children: 8,
        maximum_suggestions: 4,
        evidence_bytes: 96 * 1024,
        structured_content_bytes: 512 * 1024,
        content_bytes: 24 * 1024,
        content_lines: 240,
    };
}
