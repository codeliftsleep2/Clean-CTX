use super::*;
use crate::config::CleanCtxConfig;

#[test]
fn resolve_fidelity_prefers_explicit_arg() {
    let config = CleanCtxConfig::default();
    assert_eq!(
        resolve_fidelity(Some("high"), Some("ts"), &config),
        Fidelity::High
    );
}

#[test]
fn resolve_fidelity_uses_extension_override() {
    let mut config = CleanCtxConfig::default();
    config
        .fidelity_overrides
        .insert("ts".to_string(), "high".to_string());
    assert_eq!(
        resolve_fidelity(None, Some("ts"), &config),
        Fidelity::High
    );
}

#[test]
fn resolve_fidelity_falls_back_to_default() {
    let mut config = CleanCtxConfig::default();
    config.default_fidelity = "medium".to_string();
    assert_eq!(
        resolve_fidelity(None, Some("cs"), &config),
        Fidelity::Medium
    );
}

#[test]
fn resolve_fidelity_hard_fallback_to_low() {
    let config = CleanCtxConfig::default();
    assert_eq!(resolve_fidelity(None, None, &config), Fidelity::Low);
}

#[test]
fn parse_fidelity_arg_rejects_typo() {
    let params = serde_json::json!({
        "arguments": { "fidelity": "hihg" }
    });
    let result = parse_fidelity_arg(&Value::Null, &params);
    assert!(result.is_err());
}