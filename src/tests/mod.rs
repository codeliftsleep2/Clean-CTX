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
/// ~500ms+ per test and wasting CI resources. Furthermore, with persistence
/// enabled each handler test would open the repo's real
/// `.clean-ctx/persistence.db` — cross-test pollution via `rebuild_stats()`
/// and writes leaking into the developer's live database (Phase C0 fix).
/// This helper provides a fast, isolated default for tests that don't
/// require live CBM or persistence.
pub fn test_config() -> crate::config::CleanCtxConfig {
    let mut c = crate::config::CleanCtxConfig::default();
    c.cbm.enabled = false;
    c.persistence.enabled = false;
    c
}

// Integration tests for cross-module interactions
#[cfg(test)]
mod integration;

// Encoding invariant guard: strict UTF-8 validation, mojibake signature scan,
// and the Unicode canary fixture. Authoritative rule: .clinerules/encoding.md
// (rationale: docs/ENCODING_POLICY.md).
#[cfg(test)]
#[path = "encoding.rs"]
mod encoding;

