// src/compression/fidelity.rs
//
// The `Fidelity` enum and its string parser. Previously this lived inline
// in `compressor.rs`; both `compressor.rs` and `diff/builder.rs` depend on
// it, so it now lives in the shared `compression` namespace.

/// Compression fidelity level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fidelity {
    /// Maximum compression — strips keywords, async, fields, errors (current default)
    Low,
    /// Balanced — preserves async, field types, errors, control flow markers
    Medium,
    /// Minimal compression — preserves as much semantic depth as possible
    High,
}

impl Fidelity {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "medium" => Fidelity::Medium,
            "high" => Fidelity::High,
            _ => Fidelity::Low,
        }
    }
}
