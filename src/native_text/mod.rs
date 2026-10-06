//! Narrow provider-independent text capabilities used by native adapters.

pub mod angular_filter;
pub mod ansi;
pub mod cargo_filter;
pub mod dotnet_filter;
pub mod eslint_filter;
pub mod filter_facts;
pub mod git_diff_filter;
mod line_filter;
pub mod maven_filter;
pub mod node_build_filter;
pub mod redaction;
mod secret_patterns;
pub mod tsc_filter;
