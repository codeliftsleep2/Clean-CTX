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

/// Validate the MCP CallToolResult envelope: only content, structuredContent,
/// isError, and _meta are permitted at the result level. `content` must be a
/// non-empty array. Pure serde-json helper — usable from any `#[cfg(test)]` module.
#[cfg(test)]
pub fn assert_valid_mcp_envelope(result: &serde_json::Map<String, serde_json::Value>) {
    let allowed = ["content", "structuredContent", "isError", "_meta"];
    for key in result.keys() {
        assert!(
            allowed.contains(&key.as_str()),
            "unexpected result-level field: {key} — must use structuredContent or _meta"
        );
    }
    assert!(
        result.contains_key("content"),
        "result must contain content, got keys: {:?}",
        result.keys().collect::<Vec<_>>()
    );
    assert!(
        result["content"].as_array().is_some_and(|a| !a.is_empty()),
        "content must be a non-empty array"
    );
}

/// Validate that structuredContent contains the expected keys. Pure serde-json
/// helper — usable from any `#[cfg(test)]` module.
#[cfg(test)]
pub fn assert_structured_content_has(
    sc: &serde_json::Map<String, serde_json::Value>,
    required_keys: &[&str],
) {
    for key in required_keys {
        assert!(
            sc.contains_key(*key),
            "structuredContent should contain '{key}', got keys: {:?}",
            sc.keys().collect::<Vec<_>>()
        );
    }
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
