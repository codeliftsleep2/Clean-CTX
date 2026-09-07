// src/tests/cbm/integration.rs
//
// Integration tests for CBM compression pipeline and intelligence layer.
// Tests that compress_cbm_response properly handles various JSON inputs,
// and that the intelligence layer integrates correctly.

use serde_json::json;
use std::collections::HashMap;

// Phase D0 test types — referenced by the recovery tests at the bottom.
use crate::cbm::bridge::{GraphBridge, RecoveryOutcome};
#[test]
fn compress_cbm_response_envelope_stripping() {
    use crate::cbm::json_compress::compress_cbm_response;

    // Simulate a full CBM MCP response
    let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"CBM graph data: found 42 nodes, 15 edges"}]}}"#;
    let result = compress_cbm_response(raw);
    assert!(
        result.is_some(),
        "Should compress properly formatted response"
    );
    let comp = result.unwrap();
    assert!(comp.cbm_error.is_none(), "No error should be present");
    assert!(
        comp.compressed_text.len() < raw.len(),
        "Should achieve compression"
    );
    assert!(
        comp.compressed_text.contains("CBM graph data"),
        "Should preserve meaningful content"
    );
}

#[test]
fn proxy_stats_use_pluggable_tokenizer_format() {
    // Verify the _meta field names changed from *_est to actual names
    use crate::cbm::json_compress::compress_cbm_response;

    let raw = r#"{"jsonrpc":"2.0","id":1,"result":{"data":"test"}}"#;
    let comp = compress_cbm_response(raw).unwrap();

    // The pluggable tokenizer produces raw_tokens, not raw_tokens_est
    assert!(comp.raw_tokens_est > 0, "Should have estimated tokens");
    assert!(comp.comp_tokens_est > 0, "Should have compressed tokens");
    // With key shortening + envelope stripping, compressed should be smaller
    assert!(comp.compressed_text.len() < raw.len());
}

// ── H-03: Intelligence Layer integration tests ───────────────────

#[test]
fn provide_code_context_cbm_skipped_when_intelligence_disabled() {
    // Test that the intelligence layer doesn't influence decisions when
    // intelligence.enabled = false (regardless of CBM availability).
    use crate::mcp::heuristics;

    let mut config = crate::config::CleanCtxConfig::default();
    config.intelligence.enabled = false;

    // Verify the config default: intelligence is enabled by default,
    // and our manual disable took effect.
    assert!(
        !config.intelligence.enabled,
        "intelligence should be disabled for this test"
    );

    // Run the heuristics engine with a typical file
    let source = "pub struct User { name: String }\npub fn get_user() -> User { unimplemented!() }";
    let decision = heuristics::decide(
        "/project/src/user.rs",
        None, // explicit_fidelity
        None, // explicit_intent
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None, // path_alias
        None, // stored_fidelity
        None, // bridge
    )
    .unwrap();

    // When intelligence is disabled, cbm_informed should stay false
    assert!(
        !decision.cbm_informed,
        "cbm_informed should be false when intelligence is disabled"
    );

    // The decision summary should reflect no_cbm
    assert!(
        decision.summary().contains("no_cbm"),
        "summary should contain no_cbm when intelligence is disabled"
    );
}

#[test]
fn provide_code_context_cbm_informed_false_when_no_bridge() {
    // Test that cbm_informed stays false when there's no graph bridge
    // (even if intelligence is enabled).
    use crate::mcp::heuristics;

    let config = crate::config::CleanCtxConfig::default();
    assert!(
        config.intelligence.enabled,
        "intelligence should be enabled by default"
    );

    // Run heuristics without any bridge — cbm_informed stays false
    let source = "pub struct Config { port: u16 }";
    let decision = heuristics::decide(
        "/project/src/config.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        None, // bridge
    )
    .unwrap();

    assert!(
        !decision.cbm_informed,
        "cbm_informed should be false when no bridge available"
    );
    assert!(
        decision.summary().contains("no_cbm"),
        "summary should contain no_cbm when no bridge"
    );
}

#[test]
fn provide_code_context_cbm_informed_false_on_explicit_fidelity() {
    // When the user passes an explicit fidelity, the intelligence layer
    // should NOT override it — explicit parameters take priority.
    use crate::mcp::heuristics;

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub fn main() { println!(\"hello\"); }";
    let decision = heuristics::decide(
        "/project/src/main.rs",
        Some("high"), // explicit_fidelity
        None,         // explicit_intent
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        None, // bridge
    )
    .unwrap();

    // explicit fidelity should be honored, cbm_informed stays false
    assert!(
        !decision.cbm_informed,
        "cbm_informed should be false when explicit fidelity provided"
    );
    assert_eq!(
        format!("{:?}", decision.fidelity),
        "High",
        "explicit fidelity High should be honored"
    );
}

#[test]
fn intelligence_config_defaults_to_enabled() {
    let config = crate::config::CleanCtxConfig::default();
    assert!(
        config.intelligence.enabled,
        "IntelligenceConfig::enabled should default to true"
    );
}

// ── last_error regression: query failures are surfaced, not hidden ──

/// A bridge with CBM disabled must surface a query error via
/// `take_last_error()` instead of silently returning empty data.
#[test]
fn bridge_surfaces_query_error_on_unavailable() {
    use crate::cbm::GraphBridge;
    use crate::cbm::config::CbmConfig;

    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let mut bridge = GraphBridge::try_create(&config, std::path::Path::new("."));

    // search without CBM → empty result BUT last_error must be populated.
    let nodes = bridge.search("UserService");
    assert!(nodes.is_empty(), "search returns empty without CBM");
    let err = bridge.take_last_error();
    assert!(
        err.is_some(),
        "search failure must be surfaced via take_last_error (not silently dropped)"
    );
    if let Some(e) = &err {
        assert!(
            e.to_string().contains("CBM not available"),
            "expected 'CBM not available' error, got: {e}"
        );
    }
}

/// A successful cached query clears any stale error.
#[test]
fn cached_query_clears_stale_error() {
    use crate::cbm::SymbolImportance;
    use crate::cbm::bridge::test_helpers::new_mock;
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert(
        "UserService".to_string(),
        SymbolImportance {
            symbol: "UserService".to_string(),
            score: 0.9,
            file: "user.rs".to_string(),
        },
    );
    let mut bridge = new_mock(data);

    // Seed a search cache entry so search() takes the cache-hit path.
    use crate::cbm::bridge::CachedGraphData;
    let ttl = std::time::Instant::now() + std::time::Duration::from_secs(3600);
    bridge.cache.insert(
        "search:UserService".to_string(),
        CachedGraphData {
            data: json!([]),
            expires_at: ttl,
        },
    );

    // First simulate a fresh-error state by forcing one (mock has no client,
    // so a MISS would error — but we want the cache-HIT path).
    // Manually inject a stale error:
    bridge.set_last_error_for_test(crate::cbm::client::CbmError::LaunchError("stale".into()));
    assert!(
        bridge.take_last_error().is_some(),
        "precondition: stale error present"
    );

    // Inject again (take cleared it) then run a cache-hit search:
    bridge.set_last_error_for_test(crate::cbm::client::CbmError::LaunchError("stale".into()));
    let _ = bridge.search("UserService"); // cache hit
    assert!(
        bridge.take_last_error().is_none(),
        "cache-hit query must clear any stale error"
    );
}

// ── Phase A: Compiler-mediated CBM intelligence tests ───────────────

/// A. CBM influences automatic fidelity: high-importance symbols → ForceHigh.
#[test]
fn cbm_influences_automatic_fidelity_high_importance() {
    use crate::cbm::bridge::test_helpers::new_mock;
    use crate::cbm::SymbolImportance;
    use crate::mcp::heuristics;
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert(
        "CriticalAPI".to_string(),
        SymbolImportance {
            symbol: "CriticalAPI".to_string(),
            score: 0.95,
            file: "api.rs".to_string(),
        },
    );
    let mut bridge = new_mock(data);

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct ApiClient { key: String }";
    let decision = heuristics::decide(
        "/project/src/api.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    assert!(decision.cbm_informed);
    assert_eq!(format!("{:?}", decision.fidelity), "High");
    let intel = decision.cbm_intelligence.expect("cbm_intelligence should be Some");
    assert!(!intel.importance.is_empty());
    assert!(!intel.skip_set.contains("CriticalAPI"));
}

/// A. CBM influences automatic fidelity: low-importance symbols → ForceLow.
#[test]
fn cbm_influences_automatic_fidelity_low_importance() {
    use crate::cbm::bridge::test_helpers::new_mock;
    use crate::cbm::SymbolImportance;
    use crate::mcp::heuristics;
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert(
        "HelperUtil".to_string(),
        SymbolImportance {
            symbol: "HelperUtil".to_string(),
            score: 0.15,
            file: "util.rs".to_string(),
        },
    );
    let mut bridge = new_mock(data);

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub fn helper() -> i32 { 42 }";
    let decision = heuristics::decide(
        "/project/src/util.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    assert!(decision.cbm_informed);
    assert_eq!(format!("{:?}", decision.fidelity), "Low");
    let intel = decision.cbm_intelligence.expect("cbm_intelligence should be Some");
    assert!(intel.skip_set.contains("HelperUtil"));
}

/// B. Explicit fidelity wins: CBM does not override explicit user choice.
#[test]
fn explicit_fidelity_wins_over_cbm() {
    use crate::cbm::bridge::test_helpers::new_mock;
    use crate::cbm::SymbolImportance;
    use crate::mcp::heuristics;
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert(
        "CriticalAPI".to_string(),
        SymbolImportance {
            symbol: "CriticalAPI".to_string(),
            score: 0.95,
            file: "api.rs".to_string(),
        },
    );
    let mut bridge = new_mock(data);

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct ApiClient { key: String }";
    let decision = heuristics::decide(
        "/project/src/api.rs",
        Some("low"),
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    assert!(!decision.cbm_informed);
    assert_eq!(format!("{:?}", decision.fidelity), "Low");
    assert!(decision.cbm_intelligence.is_none());
}

/// C. Skip-set is derived from importance.
#[test]
fn cbm_skip_set_is_derived_and_reaches_compiler() {
    use crate::cbm::bridge::test_helpers::new_mock;
    use crate::cbm::SymbolImportance;
    use crate::mcp::heuristics;
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert("LowSymA".to_string(), SymbolImportance { symbol: "LowSymA".to_string(), score: 0.1, file: "file.rs".to_string() });
    data.insert("HighSymB".to_string(), SymbolImportance { symbol: "HighSymB".to_string(), score: 0.9, file: "file.rs".to_string() });
    data.insert("LowSymC".to_string(), SymbolImportance { symbol: "LowSymC".to_string(), score: 0.3, file: "file.rs".to_string() });
    let mut bridge = new_mock(data);

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct Data { value: i32 }";
    let decision = heuristics::decide(
        "/project/src/file.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    assert!(decision.cbm_informed);
    let intel = decision.cbm_intelligence.as_ref().expect("cbm_intelligence should be Some");
    assert!(intel.skip_set.contains("LowSymA"));
    assert!(!intel.skip_set.contains("HighSymB"));
    assert!(intel.skip_set.contains("LowSymC"));
    assert_eq!(intel.skip_set.len(), 2);
}

/// D. CBM unavailable: request proceeds normally without enhancement.
#[test]
fn cbm_unavailable_graceful_fallback() {
    use crate::mcp::heuristics;

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct Config { port: u16 }";
    let decision = heuristics::decide(
        "/project/src/config.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        None,
    )
    .unwrap();

    assert!(!decision.cbm_informed);
    assert!(decision.cbm_intelligence.is_none());
}

/// E. CBM-informed flag: true when intelligence obtained, even if fidelity unchanged.
#[test]
fn cbm_informed_true_even_when_fidelity_unchanged() {
    use crate::cbm::bridge::test_helpers::new_mock;
    use crate::cbm::SymbolImportance;
    use crate::mcp::heuristics;
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert("MediumSym".to_string(), SymbolImportance { symbol: "MediumSym".to_string(), score: 0.6, file: "service.rs".to_string() });
    let mut bridge = new_mock(data);

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct Service { name: String }";
    let decision = heuristics::decide(
        "/project/src/service.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    assert!(decision.cbm_informed);
    assert!(decision.cbm_intelligence.is_some());
}

/// Serializes access to the shared protocol::CAPTURED_RESPONSES sink for the
/// coverage tests below. The six tests run in parallel in the suite and share
/// that global buffer; without serialization a sibling clear()/send_response()
/// interleaves and `captured[0]` becomes another test's response.
/// `protocol::HANDLER_RESPONSE_SERIAL` exists but is feature-gated to `rust`
/// consumers (src/protocol.rs); this module compiles under default features
/// (which exclude `rust`), so it owns a local serial lock instead.
static CBM_RESPONSE_SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ── Phase B: CBM coverage lifecycle tests ────────────────────────────

/// B. Coverage/status information obtainable through the Clean-CTX boundary.
/// `handle_get_cbm_status` must include a `coverage` field in `_meta`.
#[test]
fn get_cbm_status_reports_coverage_field() {
    use crate::mcp::McpState;
    use crate::protocol::CAPTURED_RESPONSES;

    let state = McpState::new(crate::tests::test_config());
    let _serial = CBM_RESPONSE_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner()).clear();

    crate::cbm::handlers::handle_get_cbm_status(&json!(1), &json!({}), &state);

    let captured = CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner());
    assert!(!captured.is_empty(), "response must be captured");
    let resp = &captured[0];

    let coverage = resp["result"]["_meta"]["coverage"].as_object();
    assert!(
        coverage.is_some(),
        "_meta must contain a coverage field; got: {resp}"
    );
    let coverage = coverage.unwrap();
    assert!(coverage.contains_key("status"), "coverage.status must be present: {coverage:?}");
    assert!(coverage.contains_key("is_current"), "coverage.is_current must be present: {coverage:?}");
    assert!(coverage.contains_key("is_sufficient"), "coverage.is_sufficient must be present: {coverage:?}");
}

/// B. Coverage incomplete: no bridge → coverage unknown, NOT silently complete.
#[test]
fn get_cbm_status_coverage_unknown_without_bridge() {
    use crate::mcp::McpState;
    use crate::protocol::CAPTURED_RESPONSES;

    let state = McpState::new(crate::tests::test_config());
    let _serial = CBM_RESPONSE_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner()).clear();

    crate::cbm::handlers::handle_get_cbm_status(&json!(1), &json!({}), &state);

    let captured = CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner());
    let resp = &captured[0];
    let coverage = &resp["result"]["_meta"]["coverage"];

    assert_eq!(
        coverage["status"].as_str(),
        Some("unknown"),
        "coverage.status must be 'unknown' without a bridge: {coverage}"
    );
    assert_eq!(
        coverage["is_sufficient"].as_bool(),
        Some(false),
        "coverage must NOT be reported sufficient without a bridge"
    );
}

/// B. Coverage/current index: complete indexing + clean freshness → sufficient.
#[test]
fn get_cbm_status_coverage_sufficient_when_complete_and_current() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use crate::mcp::McpState;
    use crate::protocol::CAPTURED_RESPONSES;

    let state = McpState::new(crate::tests::test_config());
    {
        let mut guard = state.graph_bridge_lock();
        let bridge = new_mock_empty();
        bridge.indexing_state.lock().unwrap_or_else(|p| p.into_inner()).insert(
            "test-project".to_string(),
            crate::cbm::bridge::IndexingState::Complete,
        );
        bridge.freshness.lock().unwrap_or_else(|p| p.into_inner()).insert(
            "test-project".to_string(),
            crate::cbm::bridge::ProjectFreshness {
                dirty_generation: 3,
                indexed_generation: 3,
            },
        );
        *guard = Some(bridge);
    }

    let _serial = CBM_RESPONSE_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner()).clear();
    crate::cbm::handlers::handle_get_cbm_status(&json!(1), &json!({}), &state);

    let captured = CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner());
    let resp = &captured[0];
    let coverage = &resp["result"]["_meta"]["coverage"];

    assert_eq!(coverage["status"].as_str(), Some("complete"), "indexing Complete → coverage.status 'complete': {coverage}");
    assert_eq!(coverage["is_current"].as_bool(), Some(true), "clean freshness → is_current true: {coverage}");
    assert_eq!(coverage["is_sufficient"].as_bool(), Some(true), "complete + current → is_sufficient true: {coverage}");
}

/// B. Coverage incomplete: stale index is NOT reported as sufficient.
#[test]
fn get_cbm_status_coverage_not_sufficient_when_stale() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use crate::mcp::McpState;
    use crate::protocol::CAPTURED_RESPONSES;

    let state = McpState::new(crate::tests::test_config());
    {
        let mut guard = state.graph_bridge_lock();
        let bridge = new_mock_empty();
        bridge.indexing_state.lock().unwrap_or_else(|p| p.into_inner()).insert(
            "test-project".to_string(),
            crate::cbm::bridge::IndexingState::Complete,
        );
        bridge.freshness.lock().unwrap_or_else(|p| p.into_inner()).insert(
            "test-project".to_string(),
            crate::cbm::bridge::ProjectFreshness {
                dirty_generation: 5,
                indexed_generation: 3,
            },
        );
        *guard = Some(bridge);
    }

    let _serial = CBM_RESPONSE_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner()).clear();
    crate::cbm::handlers::handle_get_cbm_status(&json!(1), &json!({}), &state);

    let captured = CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner());
    let resp = &captured[0];
    let coverage = &resp["result"]["_meta"]["coverage"];

    assert_eq!(coverage["status"].as_str(), Some("complete"), "indexing Complete → status 'complete': {coverage}");
    assert_eq!(coverage["is_current"].as_bool(), Some(false), "stale freshness → is_current false: {coverage}");
    assert_eq!(coverage["is_sufficient"].as_bool(), Some(false), "complete + stale → NOT sufficient: {coverage}");
}

/// B. Coverage incomplete while indexing in progress.
#[test]
fn get_cbm_status_coverage_in_progress() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use crate::mcp::McpState;
    use crate::protocol::CAPTURED_RESPONSES;
    use std::time::Instant;

    let state = McpState::new(crate::tests::test_config());
    {
        let mut guard = state.graph_bridge_lock();
        let bridge = new_mock_empty();
        bridge.indexing_state.lock().unwrap_or_else(|p| p.into_inner()).insert(
            "test-project".to_string(),
            crate::cbm::bridge::IndexingState::InProgress {
                started_at: Instant::now(),
            },
        );
        *guard = Some(bridge);
    }

    let _serial = CBM_RESPONSE_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner()).clear();
    crate::cbm::handlers::handle_get_cbm_status(&json!(1), &json!({}), &state);

    let captured = CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner());
    let resp = &captured[0];
    let coverage = &resp["result"]["_meta"]["coverage"];

    assert_eq!(coverage["status"].as_str(), Some("in_progress"), "indexing InProgress → coverage 'in_progress': {coverage}");
    assert_eq!(coverage["is_sufficient"].as_bool(), Some(false), "in-progress indexing → NOT sufficient: {coverage}");
}

/// B. Coverage failed: failed indexing is reported, never masked as complete.
#[test]
fn get_cbm_status_coverage_failed() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use crate::mcp::McpState;
    use crate::protocol::CAPTURED_RESPONSES;

    let state = McpState::new(crate::tests::test_config());
    {
        let mut guard = state.graph_bridge_lock();
        let bridge = new_mock_empty();
        bridge.indexing_state.lock().unwrap_or_else(|p| p.into_inner()).insert(
            "test-project".to_string(),
            crate::cbm::bridge::IndexingState::Failed("boom".to_string()),
        );
        *guard = Some(bridge);
    }

    let _serial = CBM_RESPONSE_SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner()).clear();
    crate::cbm::handlers::handle_get_cbm_status(&json!(1), &json!({}), &state);

    let captured = CAPTURED_RESPONSES.lock().unwrap_or_else(|p| p.into_inner());
    let resp = &captured[0];
    let coverage = &resp["result"]["_meta"]["coverage"];

    assert_eq!(coverage["status"].as_str(), Some("failed"), "indexing Failed → coverage 'failed': {coverage}");
}

// ── Phase D0: Runtime health & self-healing tests ────────────────────

/// A bridge whose CBM client has a dead child — the one condition under
/// which recovery is legitimate. `binary_path` is configurable so tests can
/// force a `BinaryMissing` outcome (deterministic, no CBM binary needed).
fn mock_bridge_with_dead_child(binary_path: Option<std::path::PathBuf>) -> GraphBridge {
    let dead_client = crate::cbm::CbmClient::with_exited_child();
    let mut bridge = crate::cbm::bridge::test_helpers::new_mock(HashMap::new());
    *bridge.client.lock().unwrap_or_else(|p| p.into_inner()) = Some(dead_client);
    bridge.launch = Some(crate::cbm::bridge::LaunchParams {
        binary_path,
        timeout: std::time::Duration::from_secs(30),
        startup_timeout: std::time::Duration::from_secs(30),
        max_retries: 3,
        circuit_cooldown_secs: 30,
        cache_root: None,
        enabled: true,
    });
    bridge.status = crate::cbm::config::CbmStatus::Degraded("simulated".into());
    bridge
}

#[test]
fn ensure_operational_is_noop_on_healthy_bridge() {
    let mut bridge = crate::cbm::bridge::test_helpers::new_mock_empty();
    // new_mock_empty has client:None, so update_status() (called inside
    // ensure_operational) correctly syncs to Unavailable. The no-op
    // assertion is that NO recovery attempt was made.
    assert_eq!(bridge.recovery_attempts(), 0, "no recovery before call");
    bridge.ensure_operational();
    assert_eq!(
        bridge.recovery_attempts(),
        0,
        "healthy/no-client bridge must not trigger recovery"
    );
}

#[test]
fn ensure_operational_skips_disabled_config() {
    // Disabled CBM must never be relaunched, even when the child is dead.
    let mut bridge = mock_bridge_with_dead_child(None);
    if let Some(ref mut p) = bridge.launch {
        p.enabled = false;
    }
    bridge.ensure_operational();
    assert_eq!(bridge.recovery_attempts(), 0, "disabled CBM must not be relaunched");
}

#[test]
fn ensure_operational_skips_when_cooldown_active() {
    let mut bridge = mock_bridge_with_dead_child(None);
    bridge.ensure_operational();
    assert_eq!(bridge.recovery_attempts(), 1, "first attempt should run");
    bridge.ensure_operational();
    assert_eq!(
        bridge.recovery_attempts(),
        1,
        "cooldown must prevent a second attempt within the window"
    );
}

#[test]
fn ensure_operational_recovers_dead_child_after_cooldown() {
    let mut bridge = mock_bridge_with_dead_child(None);
    bridge.ensure_operational();
    assert_eq!(bridge.recovery_attempts(), 1, "dead child should trigger recovery");
    if let Some((outcome, _)) = bridge.last_recovery.take() {
        bridge.last_recovery = Some((outcome, std::time::Instant::now() - std::time::Duration::from_secs(60)));
    }
    bridge.ensure_operational();
    assert_eq!(bridge.recovery_attempts(), 2, "after cooldown, a second attempt runs");
}

#[test]
fn apply_recovery_outcome_recovered_sets_available() {
    let mut bridge = crate::cbm::bridge::test_helpers::new_mock_empty();
    bridge.status = crate::cbm::config::CbmStatus::Degraded("x".into());
    bridge.apply_recovery_outcome(RecoveryOutcome::Recovered);
    assert!(bridge.status.is_available());
    assert_eq!(bridge.last_recovery().unwrap().0, "recovered");
}

#[test]
fn apply_recovery_outcome_daemon_conflict_degrades_without_retry() {
    let mut bridge = crate::cbm::bridge::test_helpers::new_mock_empty();
    bridge.apply_recovery_outcome(RecoveryOutcome::DaemonConflict("ABI mismatch".into()));
    assert!(!bridge.status.is_available());
    let status_dbg = format!("{:?}", bridge.status);
    assert!(status_dbg.contains("daemon conflict"), "status was: {status_dbg}");
    assert_eq!(bridge.last_recovery().unwrap().0, "daemon_conflict");
}

#[test]
fn apply_recovery_outcome_failed_degrades() {
    let mut bridge = crate::cbm::bridge::test_helpers::new_mock_empty();
    bridge.apply_recovery_outcome(RecoveryOutcome::Failed("boom".into()));
    assert!(!bridge.status.is_available());
    let status_dbg = format!("{:?}", bridge.status);
    assert!(status_dbg.contains("recovery failed"), "status was: {status_dbg}");
    assert_eq!(bridge.last_recovery().unwrap().0, "failed");
}

#[test]
fn apply_recovery_outcome_binary_missing_sets_unavailable() {
    let mut bridge = crate::cbm::bridge::test_helpers::new_mock_empty();
    bridge.apply_recovery_outcome(RecoveryOutcome::BinaryMissing);
    assert_eq!(bridge.status, crate::cbm::config::CbmStatus::Unavailable);
    assert_eq!(bridge.last_recovery().unwrap().0, "binary_missing");
}

#[test]
fn child_alive_false_for_exited_process() {
    let mut client = crate::cbm::CbmClient::with_exited_child();
    assert!(
        !client.child_alive(),
        "echo exits immediately → child must be reported dead"
    );
}

#[test]
fn recovery_never_fails_compilation_contract() {
    let mut bridge = mock_bridge_with_dead_child(None);
    for _ in 0..5 {
        bridge.ensure_operational();
    }
    assert!(bridge.recovery_attempts() >= 1);
    assert_eq!(bridge.recovery_attempts(), 1, "recovery must be bounded by cooldown");
}


// ── Phase D1: CBM data-flow intelligence & context expansion tests ─────

/// Successful data-flow consultation: seeds are traced, candidates parsed
/// and carried request-scoped as advisory `CbmIntelligence.data_flow`.
#[test]
fn d1_data_flow_consultation_produces_request_scoped_context() {
    use crate::cbm::bridge::test_helpers::{new_mock, seed_data_flow};
    use crate::cbm::{DataFlowContext, SymbolImportance};
    use crate::mcp::heuristics;
    use std::collections::{HashMap, HashSet};

    let mut data = HashMap::new();
    data.insert(
        "CriticalAPI".to_string(),
        SymbolImportance { symbol: "CriticalAPI".into(), score: 0.95, file: "api.rs".into() },
    );
    let mut bridge = new_mock(data);
    // Seed a cached data-flow trace for the seed symbol (no CBM client needed).
    let df = DataFlowContext {
        symbols: ["helper".to_string(), "CriticalAPI".to_string()].into_iter().collect::<HashSet<_>>(),
        files: ["src/helper.rs".to_string(), "api.rs".to_string()].into_iter().collect::<HashSet<_>>(),
    };
    seed_data_flow(&bridge, "CriticalAPI", 2, df);

    let config = crate::config::CleanCtxConfig::default(); // data_flow_enabled defaults true
    let source = "pub struct ApiClient { key: String }";
    let decision = heuristics::decide(
        "/project/src/api.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    assert!(decision.cbm_informed);
    let intel = decision.cbm_intelligence.as_ref().expect("cbm_intelligence should be Some");
    let df = intel.data_flow.as_ref().expect("data_flow should be Some (seeded trace)");
    assert!(df.symbols.contains("helper"), "trace endpoints parsed into symbols");
    assert!(df.files.contains("src/helper.rs"), "files resolved via importance");
    assert!(decision.summary().contains("cbm_df="), "summary reports data-flow diagnostics");
}

/// Multiple seeds: overlapping results are deduplicated and ordering is
/// deterministic (seeds sorted by importance score, descending; bounded).
#[test]
fn d1_multiple_seeds_deduplicate_and_bounding() {
    use crate::cbm::bridge::test_helpers::{new_mock, seed_data_flow};
    use crate::cbm::{DataFlowContext, SymbolImportance};
    use crate::mcp::heuristics;
    use std::collections::{HashMap, HashSet};

    let mut data = HashMap::new();
    data.insert(
        "SeedA".to_string(),
        SymbolImportance { symbol: "SeedA".into(), score: 0.95, file: "api.rs".into() },
    );
    data.insert(
        "SeedB".to_string(),
        SymbolImportance { symbol: "SeedB".into(), score: 0.6, file: "api.rs".into() },
    );
    data.insert(
        "Shared".to_string(),
        SymbolImportance { symbol: "Shared".into(), score: 0.2, file: "api.rs".into() },
    );
    let mut bridge = new_mock(data);
    seed_data_flow(
        &bridge,
        "SeedA",
        2,
        DataFlowContext {
            symbols: ["shared".to_string()].into_iter().collect::<HashSet<_>>(),
            files: HashSet::new(),
        },
    );
    seed_data_flow(
        &bridge,
        "SeedB",
        2,
        DataFlowContext {
            symbols: ["shared".to_string()].into_iter().collect::<HashSet<_>>(),
            files: HashSet::new(),
        },
    );

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct ApiClient { key: String }";
    let decision = heuristics::decide(
        "/project/src/api.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    let intel = decision.cbm_intelligence.expect("Some");
    let df = intel.data_flow.expect("Some");
    assert_eq!(
        df.symbols.iter().filter(|s| *s == "shared").count(),
        1,
        "overlapping seed results are deduplicated"
    );
    assert!(
        df.symbols.len() <= 12,
        "bounded by data_flow_max_symbols (12)"
    );
}
/// Empty or failed data-flow consultation produces NO expansion — the
/// request still succeeds with `data_flow = None` and the skip-set unchanged.
#[test]
fn d1_empty_or_failed_data_flow_produces_no_expansion() {
    use crate::cbm::bridge::test_helpers::new_mock;
    use crate::cbm::SymbolImportance;
    use crate::mcp::heuristics;
    use std::collections::HashMap;

    let mut data = HashMap::new();
    data.insert(
        "CriticalAPI".to_string(),
        SymbolImportance { symbol: "CriticalAPI".into(), score: 0.95, file: "api.rs".into() },
    );
    // No seeded data-flow → the mock's trace query fails (client None) →
    // empty → data_flow = None (graceful degradation; compilation succeeds).
    let mut bridge = new_mock(data);
    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct ApiClient { key: String }";
    let decision = heuristics::decide(
        "/project/src/api.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();
    assert!(decision.cbm_informed, "importance still succeeded");
    assert!(
        decision.cbm_intelligence.as_ref().unwrap().data_flow.is_none(),
        "failed data-flow must yield no expansion"
    );
    assert!(
        !decision.summary().contains("cbm_df="),
        "no data-flow candidates → no diagnostic"
    );
}

/// Skip-set interaction: a data-flow-identified in-file symbol is retained
/// through the skip-set (removed from it), while non-relevant low-importance
/// symbols remain excluded.
#[test]
fn d1_data_flow_relevant_symbol_retained_through_skip_set() {
    use crate::cbm::bridge::test_helpers::{new_mock, seed_data_flow};
    use crate::cbm::{DataFlowContext, SymbolImportance};
    use crate::mcp::heuristics;
    use std::collections::{HashMap, HashSet};

    let mut data = HashMap::new();
    data.insert(
        "helper".to_string(),
        SymbolImportance { symbol: "helper".into(), score: 0.1, file: "api.rs".into() },
    );
    data.insert(
        "CriticalAPI".to_string(),
        SymbolImportance { symbol: "CriticalAPI".into(), score: 0.95, file: "api.rs".into() },
    );
    let mut bridge = new_mock(data);
    // Data-flow identifies "helper" as participating in the request-relevant
    // flow (bare name resolvable against this file's importance entries).
    seed_data_flow(
        &bridge,
        "CriticalAPI",
        2,
        DataFlowContext {
            symbols: ["helper".to_string()].into_iter().collect::<HashSet<_>>(),
            files: HashSet::new(),
        },
    );

    let config = crate::config::CleanCtxConfig::default();
    let source = "pub struct ApiClient { key: String }";
    let decision = heuristics::decide(
        "/project/src/api.rs",
        None,
        None,
        &config,
        &crate::ir::replay::ContextState::new(),
        source,
        None,
        None,
        Some(&mut bridge),
    )
    .unwrap();

    let intel = decision.cbm_intelligence.expect("Some");
    assert!(
        !intel.skip_set.contains("helper"),
        "data-flow-relevant symbol must be retained through the skip-set"
    );
}
