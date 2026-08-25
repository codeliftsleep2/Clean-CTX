// src/tests/mod.rs
//
// This module's sole purpose is to teach rustc that the test files should be
// compiled. The actual test modules are loaded via `#[path]` attributes from
// their respective source files, so this file serves as documentation and a
// compilation anchor.
//
// Shared test helpers live here so any test module can access them via
// `crate::tests::*`.

/// Create a test config with CBM and persistence disabled.
///
/// Most unit tests do not need CBM or persistence. Using a default config
/// would launch a CBM subprocess on every `McpState::new()` call, costing
/// ~500ms+ per test and wasting CI resources. This helper provides a fast,
/// isolated default for tests that don't require live CBM.
pub fn test_config() -> crate::config::CleanCtxConfig {
    let mut c = crate::config::CleanCtxConfig::default();
    c.cbm.enabled = false;
    c
}

// Integration tests for cross-module interactions
#[cfg(test)]
mod integration;
