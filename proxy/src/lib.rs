// proxy/src/lib.rs
//
// Clean-CTX proxy library root.
// Re-exports public API for integration testing.

pub mod cache;
pub mod community_filters;
pub mod config;
pub mod error;
pub mod filter_loader;
pub mod filter_registry;
pub mod filter_rules;
pub mod filter_stats;
pub mod filters;
pub mod logger;
pub mod pipeline;
pub mod platform;
pub mod rate_limiter;
pub mod scrub;
pub mod scrub_patterns;
pub mod server;
pub mod transform;

#[allow(dead_code)]
mod config_tool_filter {
    // ToolFilterConfig is public API — dead_code warning is intentional.
    // It's implemented for future .clean-ctx.json integration.
}
