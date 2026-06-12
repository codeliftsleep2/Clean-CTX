// src/tests/mcp/state.rs
//
// Tests for McpState: creation, warnings, source cache

use super::*;
use crate::config::CleanCtxConfig;

#[test]
fn state_new_creates_empty_registries() {
    let config = CleanCtxConfig::default();
    let state = McpState::new(config);
    // Warnings should be empty
    assert!(state.warnings.is_empty());
}

#[test]
fn push_and_drain_warnings() {
    let config = CleanCtxConfig::default();
    let mut state = McpState::new(config);
    state.push_warning("test warning 1");
    state.push_warning("test warning 2");
    assert_eq!(state.warnings.len(), 2);
    let drained = state.drain_warnings();
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0], "test warning 1");
    assert_eq!(drained[1], "test warning 2");
    // After draining, warnings should be empty
    assert!(state.warnings.is_empty());
}

#[test]
fn drain_warnings_on_empty_returns_empty() {
    let config = CleanCtxConfig::default();
    let mut state = McpState::new(config);
    let drained = state.drain_warnings();
    assert!(drained.is_empty());
}

#[test]
fn read_source_caches_file_content() {
    use std::sync::Arc;
    let config = CleanCtxConfig::default();
    let mut state = McpState::new(config);
    // Read a known file
    let result = state.read_source("src/lib.rs");
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(!content.is_empty());
    // Reading again should come from cache
    let content2 = state.read_source("src/lib.rs").unwrap();
    // Both should be the same Arc (pointer equality means cache hit)
    assert!(Arc::ptr_eq(&content, &content2));
}

#[test]
fn read_source_nonexistent_file_returns_error() {
    let config = CleanCtxConfig::default();
    let mut state = McpState::new(config);
    let result = state.read_source("/nonexistent/file/path.rs");
    assert!(result.is_err());
}

#[test]
fn state_accessor_mut_methods() {
    let config = CleanCtxConfig::default();
    let mut state = McpState::new(config);
    // Verify accessor methods return the correct types
    let _dict = state.dict_mut();
    let _cache = state.cache_mut();
    let _ir = state.ir_context_mut();
    let _td = state.text_delta_mut();
}