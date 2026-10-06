//! Narrow provider-independent text capabilities used by native adapters.

pub mod ansi;
pub mod cargo_filter;
pub mod filter_facts;
pub mod git_diff_filter;
mod line_filter;
pub mod redaction;
mod secret_patterns;
