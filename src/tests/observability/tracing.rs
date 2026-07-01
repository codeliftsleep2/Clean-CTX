// src/tests/observability/tracing.rs
//
// A-04: Tests for the tracing initialization.
//
// Note: `init_tracing` uses `try_init()` so it's safe to call
// multiple times in tests (subsequent calls are no-ops).

use crate::observability::tracing::init_tracing;

#[test]
fn test_init_tracing_default() {
    // Should not panic — uses default env filter (info)
    init_tracing();
}

#[test]
fn test_init_tracing_json_format() {
    // Set JSON format env var
    unsafe { std::env::set_var("CLEAN_CTX_LOG_FORMAT", "json"); }
    init_tracing();
    // Clean up
    unsafe { std::env::remove_var("CLEAN_CTX_LOG_FORMAT"); }
}

#[test]
fn test_init_tracing_custom_filter() {
    unsafe { std::env::set_var("CLEAN_CTX_LOG_FILTER", "warn,clean_ctx=debug"); }
    init_tracing();
    unsafe { std::env::remove_var("CLEAN_CTX_LOG_FILTER"); }
}

#[test]
fn test_init_tracing_custom_level() {
    unsafe { std::env::set_var("CLEAN_CTX_LOG", "debug"); }
    init_tracing();
    unsafe { std::env::remove_var("CLEAN_CTX_LOG"); }
}

#[test]
fn test_init_tracing_twice_is_safe() {
    // Calling init_tracing twice should be a no-op (try_init)
    init_tracing();
    init_tracing();
}