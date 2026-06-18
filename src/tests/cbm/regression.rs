// src/tests/cbm/regression.rs
//
// Regression tests for CBM audit fixes.
// Tests pure functions: is_retryable, check_cache behavior,
// json_compress utilities, and enrichment compression savings.

use serde_json::json;

// ── Fix 2: is_retryable coverage ────────────────────────────

#[test]
fn is_retryable_connection_lost_is_true() {
    use crate::cbm::client::CbmError;
    use crate::cbm::client::is_retryable;
    let err = CbmError::ConnectionLost("pipe broke".into());
    assert!(is_retryable(&err));
}

#[test]
fn is_retryable_timeout_is_true() {
    use crate::cbm::client::CbmError;
    use crate::cbm::client::is_retryable;
    let err = CbmError::Timeout(std::time::Duration::from_secs(30));
    assert!(is_retryable(&err));
}

#[test]
fn is_retryable_rpc_internal_is_true() {
    use crate::cbm::client::CbmError;
    use crate::cbm::client::is_retryable;
    let err = CbmError::RpcError { code: -32603, message: "Internal".into() };
    assert!(is_retryable(&err));
}

#[test]
fn is_retryable_launch_is_false() {
    use crate::cbm::client::CbmError;
    use crate::cbm::client::is_retryable;
    assert!(!is_retryable(&CbmError::LaunchError("bin".into())));
}

#[test]
fn is_retryable_parse_is_false() {
    use crate::cbm::client::CbmError;
    use crate::cbm::client::is_retryable;
    assert!(!is_retryable(&CbmError::ParseError("bad".into())));
}

#[test]
fn is_retryable_method_not_found_is_false() {
    use crate::cbm::client::CbmError;
    use crate::cbm::client::is_retryable;
    let err = CbmError::RpcError { code: -32601, message: "Method".into() };
    assert!(!is_retryable(&err));
}

// ── Fix 5: check_cache eviction regression ──────────────────

#[test]
fn expired_cache_entry_is_evicted() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;
    use std::path::Path;
    use crate::cbm::bridge::CachedGraphData;
    use dashmap::mapref::entry::Entry;

    let config = CbmConfig { enabled: false, ..Default::default() };
    let mut bridge = GraphBridge::try_create(&config, Path::new("."));

    // Insert expired entry under "symbol_importance" so get_symbol_importance_mut
    // finds it via check_cache, evicts it, then the query fails (no CBM),
    // returning an empty map — but the expired entry should be gone.
    bridge.cache.insert("symbol_importance".into(), CachedGraphData {
        data: json!("value"),
        expires_at: std::time::Instant::now() - std::time::Duration::from_secs(1),
    });

    // get_symbol_importance_mut calls check_cache("symbol_importance")
    // which finds the expired entry, evicts it, then fails the query
    let result = bridge.get_symbol_importance_mut();
    assert!(result.is_empty(), "Should return empty map with no CBM");
    // The expired entry must be gone from cache
    assert!(bridge.cache.get("symbol_importance").is_none(),
        "Expired cache entry should be evicted after check_cache");
}

// ── Fix 1: compress_cbm_response on enrichment-shaped data ──

#[test]
fn compress_enrichment_data_produces_savings() {
    use crate::cbm::json_compress::compress_cbm_response;

    let enrichment = json!({
        "symbols": [
            {"sy": "UserService", "sc": 0.95, "f": "src/user.rs"},
            {"sy": "PaymentGateway", "sc": 0.87, "f": "src/payment.rs"},
            {"sy": "AuthService", "sc": 0.72, "f": "src/auth.rs"},
        ]
    }).to_string();

    let raw_len = enrichment.len();
    let result = compress_cbm_response(&enrichment);
    assert!(result.is_some());
    let comp = result.unwrap();
    assert!(comp.cbm_error.is_none());
    assert!(comp.compressed_text.len() < raw_len);
}

#[test]
fn compress_empty_enrichment_ok() {
    use crate::cbm::json_compress::compress_cbm_response;
    let result = compress_cbm_response(&json!({"symbols": []}).to_string());
    assert!(result.is_some());
    assert!(result.unwrap().cbm_error.is_none());
}

#[test]
fn compress_preserves_key_values() {
    use crate::cbm::json_compress::compress_cbm_response;
    let data = json!({"symbols": [{"sy": "UserService", "sc": 1.0, "f": "src/user.rs"}]}).to_string();
    let result = compress_cbm_response(&data).unwrap();
    assert!(result.compressed_text.contains("UserService"));
}

#[test]
fn enrichment_50_symbols_savings_above_30pct() {
    use crate::cbm::json_compress::compress_cbm_response;
    let mut symbols = Vec::new();
    for i in 0..50 {
        symbols.push(json!({"sy": format!("S_{}", i), "sc": (i as f64) * 0.02, "f": format!("src/m{}.rs", i%10)}));
    }
    let data = json!({"symbols": symbols}).to_string();
    let raw = data.len();
    let comp = compress_cbm_response(&data).unwrap().compressed_text.len();
    let savings = (1.0 - comp as f64 / raw as f64) * 100.0;
    assert!(savings > 30.0, "Savings {}% > 30%", savings);
}

// ── Fix 4: shorten_key coverage ─────────────────────────────

#[test]
fn shorten_key_covers_all_cbm_keys() {
    use crate::cbm::json_compress::shorten_key;
    let keys = ["results","symbols","edges","nodes","name","file","label","id",
        "score","importance","reason","symbol","change_type"];
    for key in &keys {
        let s = shorten_key(key);
        assert!(s.len() <= key.len() && s != *key, "Key '{}' not shortened", key);
    }
}

// ── CBM config & status tests ───────────────────────────────

#[test]
fn cbm_config_defaults_sane() {
    let c = crate::cbm::config::CbmConfig::default();
    assert!(c.enabled);
    assert_eq!(c.cache_ttl, 300);
    assert_eq!(c.query_timeout_ms, 30000);
    assert!(c.auto_launch);
}

#[test]
fn cbm_status_transitions() {
    use crate::cbm::config::CbmStatus;
    assert!(CbmStatus::Available.is_available());
    assert!(!CbmStatus::Degraded("x".into()).is_available());
    assert!(!CbmStatus::Unavailable.is_available());
}

// ── Proxy fallback: apply_minimum_compression ───────────────

#[test]
fn min_compression_strips_jsonrpc() {
    use crate::cbm::proxy::apply_minimum_compression;
    let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"data":"hello"}}"#;
    let c = apply_minimum_compression(raw);
    assert!(c.len() < raw.len());
    assert!(!c.contains("jsonrpc"));
}

#[test]
fn min_compression_non_json_strips_whitespace() {
    use crate::cbm::proxy::apply_minimum_compression;
    let c = apply_minimum_compression("plain text");
    assert!(!c.contains(' '));
}

// ── GraphBridge initial state ───────────────────────────────

#[test]
fn bridge_disabled_is_unavailable() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;
    let mut config = CbmConfig::default();
    config.enabled = false;
    let bridge = GraphBridge::try_create(&config, std::path::Path::new("."));
    assert!(!bridge.is_available());
}

#[test]
fn bridge_detect_changes_returns_none_when_no_client() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;
    let config = CbmConfig { enabled: false, ..Default::default() };
    let mut bridge = GraphBridge::try_create(&config, std::path::Path::new("."));
    assert_eq!(bridge.detect_changes().unwrap(), None);
}

// ── CbmError Display ────────────────────────────────────────

#[test]
fn cbm_error_display_has_message() {
    use crate::cbm::client::CbmError;
    let e = CbmError::LaunchError("bad".into());
    assert!(format!("{}", e).contains("bad"));
}

#[test]
fn cbm_error_timeout_has_seconds() {
    use crate::cbm::client::CbmError;
    let e = CbmError::Timeout(std::time::Duration::from_secs(5));
    assert!(format!("{}", e).contains("5s"));
}

// ── Bridge::invalidate_symbol ───────────────────────────────

#[test]
fn invalidate_symbol_removes_matching_keys() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;
    use crate::cbm::bridge::CachedGraphData;

    let config = CbmConfig { enabled: false, ..Default::default() };
    let mut bridge = GraphBridge::try_create(&config, std::path::Path::new("."));
    let t = std::time::Instant::now() + std::time::Duration::from_secs(300);

    bridge.cache.insert("blast:UserService".into(), CachedGraphData { data: json!("d1"), expires_at: t });
    bridge.cache.insert("blast:Payment".into(), CachedGraphData { data: json!("d2"), expires_at: t });
    bridge.cache.insert("arch".into(), CachedGraphData { data: json!("d3"), expires_at: t });

    bridge.invalidate_symbol("UserService");
    assert!(bridge.cache.get("blast:UserService").is_none());
    assert!(bridge.cache.get("blast:Payment").is_some());
    assert!(bridge.cache.get("arch").is_some());
}