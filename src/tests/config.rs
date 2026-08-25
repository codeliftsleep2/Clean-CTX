use super::*;
use serial_test::serial;

#[test]
fn test_default_config() {
    let config = CleanCtxConfig::default();
    assert_eq!(config.default_fidelity, crate::compression::Fidelity::Low);
    assert!(config.diff_compression);
    assert!(config.type_aliases.is_empty());
}

// ── Angular Ecosystem Deepening sub-layer configs (Phase 6) ───────

#[test]
fn test_meta_layer_config_defaults() {
    let config = MetaLayerConfig::default();
    assert!(config.enabled);
    assert!(config.rxjs.enabled);
    assert_eq!(config.rxjs.min_pipe_operators, 2);
    assert!(config.ngrx.enabled);
    assert!(config.ngrx.include_dispatch_sites);
    assert!(config.ngrx.include_select_sites);
    assert!(config.ngrx.entity_selectors);
    assert!(config.ngrx.cross_layer_cbm);
    assert!(config.signals.enabled);
    assert!(config.routing.enabled);
}

#[test]
fn test_meta_layer_config_json_round_trip() {
    let config = MetaLayerConfig {
        enabled: true,
        rxjs: RxJsConfig {
            enabled: true,
            min_pipe_operators: 3,
        },
        ngrx: NgRxConfig {
            enabled: true,
            include_dispatch_sites: false,
            include_select_sites: true,
            entity_selectors: false,
            cross_layer_cbm: true,
        },
        signals: SignalsConfig { enabled: true },
        routing: RoutingConfig { enabled: false },
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let parsed: MetaLayerConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.rxjs.min_pipe_operators, 3);
    assert!(!parsed.ngrx.include_dispatch_sites);
    assert!(!parsed.ngrx.entity_selectors);
    assert!(!parsed.routing.enabled);
}

#[test]
fn test_meta_layer_config_backward_compatible_partial_json() {
    // A legacy .clean-ctx.json with only `{ "enabled": false }` still
    // parses — the new sub-layer fields are `#[serde(default)]`.
    let json = r#"{ "enabled": false }"#;
    let parsed: MetaLayerConfig = serde_json::from_str(json).expect("parse");
    assert!(!parsed.enabled);
    assert!(parsed.rxjs.enabled);
    assert_eq!(parsed.rxjs.min_pipe_operators, 2);
    assert!(parsed.ngrx.enabled);
    assert!(parsed.signals.enabled);
    assert!(parsed.routing.enabled);
}

/// F-12 (FAANG audit): the old substring check matched `"dist"` inside
/// `"distribute"`. The new glob matcher must NOT do that.
#[test]
fn test_exclusion_segment_match() {
    let mut config = CleanCtxConfig::default();
    config.exclude_patterns.push("dist".to_string());

    // "dist" as a standalone segment should match.
    assert!(config.is_excluded("src/dist/utils.ts"));
    // "distribute" is NOT the same segment — must NOT match.
    assert!(
        !config.is_excluded("src/distribute/utils.ts"),
        "substring 'dist' inside 'distribute' should not match an exact-segment pattern"
    );
}

#[test]
fn test_exclusion_wildcard() {
    let mut config = CleanCtxConfig::default();
    config.exclude_patterns.push("*.test.ts".to_string());

    assert!(config.is_excluded("src/foo.test.ts"));
    assert!(!config.is_excluded("src/foo_spec.ts"));
    assert!(!config.is_excluded("src/foo.test.js"));
}

#[test]
fn test_exclusion_question_mark_glob() {
    let mut config = CleanCtxConfig::default();
    config.exclude_patterns.push("file?.ts".to_string());

    assert!(config.is_excluded("src/file1.ts"));
    assert!(!config.is_excluded("src/file.ts"));
    assert!(!config.is_excluded("src/file12.ts"));
}

#[test]
fn test_exclusion_node_modules_and_hidden() {
    let mut config = CleanCtxConfig::default();
    config.exclude_patterns.push("node_modules".to_string());
    config.exclude_patterns.push(".test.".to_string());

    assert!(config.is_excluded("src/node_modules/file.ts"));
    assert!(config.is_excluded("src/file.test.ts"));
    assert!(!config.is_excluded("src/file.ts"));
}

/// F-11 (FAANG audit): `find_config` caches its result in a process-global
/// `OnceLock`. Two calls from the same start_dir must return the same path.
#[test]
fn test_find_config_caches_result() {
    let dir = std::path::Path::new(".");
    let p1 = CleanCtxConfig::find_config(dir);
    let p2 = CleanCtxConfig::find_config(dir);
    assert_eq!(p1, p2);
}

// A-14: CI/CD awareness tests
//
// These tests mutate process-global env vars. They MUST run serially
// to avoid data races with parallel test execution (a concurrent test
// reading CI=true while this test temporarily clears it would flap).
#[serial]
#[test]
fn test_is_ci_environment_detects_ci_var() {
    // Save original env vars
    let original_ci = std::env::var("CI").ok();

    unsafe {
        // Clear all CI vars first
        std::env::remove_var("CI");
        std::env::remove_var("GITHUB_ACTIONS");
        std::env::remove_var("TF_BUILD");
        std::env::remove_var("GITLAB_CI");
        std::env::remove_var("JENKINS_URL");
        std::env::remove_var("CIRCLECI");
        std::env::remove_var("TRAVIS");

        // Test CI=true
        std::env::set_var("CI", "true");
    }
    assert!(CleanCtxConfig::is_ci_environment(), "Should detect CI=true");

    // Cleanup
    unsafe {
        std::env::remove_var("CI");
        if let Some(val) = original_ci {
            std::env::set_var("CI", val);
        }
    }
}

#[serial]
#[test]
fn test_is_ci_environment_detects_github_actions() {
    let original_gha = std::env::var("GITHUB_ACTIONS").ok();

    unsafe {
        std::env::remove_var("GITHUB_ACTIONS");
        std::env::remove_var("CI");
        std::env::set_var("GITHUB_ACTIONS", "true");
    }
    assert!(
        CleanCtxConfig::is_ci_environment(),
        "Should detect GITHUB_ACTIONS"
    );

    unsafe {
        std::env::remove_var("GITHUB_ACTIONS");
        if let Some(val) = original_gha {
            std::env::set_var("GITHUB_ACTIONS", val);
        }
    }
}

#[serial]
#[test]
fn test_is_ci_environment_detects_tf_build() {
    let original_tf = std::env::var("TF_BUILD").ok();

    unsafe {
        std::env::remove_var("TF_BUILD");
        std::env::remove_var("CI");
        std::env::set_var("TF_BUILD", "true");
    }
    assert!(
        CleanCtxConfig::is_ci_environment(),
        "Should detect TF_BUILD"
    );

    unsafe {
        std::env::remove_var("TF_BUILD");
        if let Some(val) = original_tf {
            std::env::set_var("TF_BUILD", val);
        }
    }
}

#[serial]
#[test]
fn test_is_ci_environment_returns_false_when_not_in_ci() {
    // Save and clear all CI env vars
    let vars_to_clear = [
        "CI",
        "TF_BUILD",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "JENKINS_URL",
        "CIRCLECI",
        "TRAVIS",
    ];
    let originals: Vec<Option<String>> = vars_to_clear
        .iter()
        .map(|var| std::env::var(var).ok())
        .collect();

    unsafe {
        for var in &vars_to_clear {
            std::env::remove_var(var);
        }
    }

    assert!(
        !CleanCtxConfig::is_ci_environment(),
        "Should return false when no CI env vars are set"
    );

    // Restore original env vars
    unsafe {
        for (var, original) in vars_to_clear.iter().zip(originals) {
            if let Some(val) = original {
                std::env::set_var(var, val);
            }
        }
    }
}

#[test]
fn proxy_config_missing_block_uses_defaults() {
    let json = serde_json::json!({});
    let config: CleanCtxConfig = serde_json::from_value(json).unwrap();
    assert!(!config.proxy.auto_start);
    assert_eq!(config.proxy.port, 8787);
    assert_eq!(config.proxy.tail_ttl, "5m");
    assert_eq!(config.proxy.rate_limit_rps, 60.0);
    assert_eq!(config.proxy.rate_limit_burst, 10.0);
}

#[test]
fn proxy_config_partial_block_uses_defaults() {
    let json = serde_json::json!({
        "proxy": { "auto_start": true }
    });
    let config: CleanCtxConfig = serde_json::from_value(json).unwrap();
    assert!(config.proxy.auto_start);
    // Missing fields fall back to defaults.
    assert_eq!(config.proxy.port, 8787);
    assert!(!config.proxy.auto_cache);
    assert_eq!(config.proxy.tail_ttl, "5m");
    assert!(config.proxy.drop_tools.is_empty());
    assert!(!config.proxy.strip_ansi);
    assert!(!config.proxy.trim_bash_git);
    assert!(config.proxy.model_override.is_none());
    assert!(!config.proxy.scrub_secrets);
    assert!(!config.proxy.tool_filters);
    assert!(config.proxy.upstream_url.is_none());
    assert!(config.proxy.api_key.is_none());
    assert_eq!(config.proxy.rate_limit_rps, 60.0);
    assert_eq!(config.proxy.rate_limit_burst, 10.0);
}

// ── Proxy auto-start config tests ─────────────────────────────────

#[test]
fn proxy_auto_start_defaults_false() {
    let config = CleanCtxConfig::default();
    assert!(!config.proxy.auto_start);
}

#[test]
fn proxy_config_parses_all_fields() {
    let json = serde_json::json!({
        "proxy": {
            "auto_start": true,
            "port": 9999,
            "auto_cache": true,
            "tail_ttl": "10m",
            "drop_tools": ["NotebookEdit", "CronCreate"],
            "strip_ansi": true,
            "trim_bash_git": true,
            "model_override": "claude-opus-4-6",
            "scrub_secrets": true,
            "tool_filters": true,
            "upstream_url": "http://127.0.0.1:4141",
            "api_key": "secret-key",
            "rate_limit_rps": 30.0,
            "rate_limit_burst": 5.0
        }
    });
    let config: CleanCtxConfig = serde_json::from_value(json).unwrap();
    assert!(config.proxy.auto_start);
    assert_eq!(config.proxy.port, 9999);
    assert!(config.proxy.auto_cache);
    assert_eq!(config.proxy.tail_ttl, "10m");
    assert_eq!(config.proxy.drop_tools, vec!["NotebookEdit", "CronCreate"]);
    assert!(config.proxy.strip_ansi);
    assert!(config.proxy.trim_bash_git);
    assert_eq!(
        config.proxy.model_override.as_deref(),
        Some("claude-opus-4-6")
    );
    assert!(config.proxy.scrub_secrets);
    assert!(config.proxy.tool_filters);
    assert_eq!(
        config.proxy.upstream_url.as_deref(),
        Some("http://127.0.0.1:4141")
    );
    assert_eq!(config.proxy.api_key.as_deref(), Some("secret-key"));
    assert_eq!(config.proxy.rate_limit_rps, 30.0);
    assert_eq!(config.proxy.rate_limit_burst, 5.0);
}

#[serial]
#[test]
fn test_ci_detection_integration() {
    // Test that CI detection works correctly with different env var combinations
    let vars_to_clear = [
        "CI",
        "TF_BUILD",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "JENKINS_URL",
        "CIRCLECI",
        "TRAVIS",
    ];
    let originals: Vec<Option<String>> = vars_to_clear
        .iter()
        .map(|var| std::env::var(var).ok())
        .collect();

    unsafe {
        // Clear all CI vars
        for var in &vars_to_clear {
            std::env::remove_var(var);
        }

        // Test 1: No CI vars set → not CI
        assert!(
            !CleanCtxConfig::is_ci_environment(),
            "Should not detect CI when no vars set"
        );

        // Test 2: CI=true → CI detected
        std::env::set_var("CI", "true");
        assert!(CleanCtxConfig::is_ci_environment(), "Should detect CI=true");

        // Test 3: CI=false → not CI (only "true" counts)
        std::env::set_var("CI", "false");
        assert!(
            !CleanCtxConfig::is_ci_environment(),
            "CI=false should not trigger CI detection"
        );

        // Cleanup
        std::env::remove_var("CI");
    }

    // Restore original env vars
    unsafe {
        for (var, original) in vars_to_clear.iter().zip(originals) {
            if let Some(val) = original {
                std::env::set_var(var, val);
            }
        }
    }
}

#[test]
fn proxy_config_serializes_roundtrip() {
    let mut config = CleanCtxConfig::default();
    config.proxy.auto_start = true;
    config.proxy.port = 9999;
    config.proxy.auto_cache = true;
    config.proxy.tail_ttl = "10m".to_string();
    config.proxy.drop_tools = vec!["NotebookEdit".to_string()];
    config.proxy.strip_ansi = true;
    config.proxy.trim_bash_git = true;
    config.proxy.model_override = Some("claude-opus-4-6".to_string());
    config.proxy.scrub_secrets = true;
    config.proxy.tool_filters = true;
    config.proxy.upstream_url = Some("http://127.0.0.1:4141".to_string());
    config.proxy.api_key = Some("secret-key".to_string());
    config.proxy.rate_limit_rps = 30.0;
    config.proxy.rate_limit_burst = 5.0;

    let json = serde_json::to_string(&config).unwrap();
    let roundtrip: CleanCtxConfig = serde_json::from_str(&json).unwrap();
    assert!(roundtrip.proxy.auto_start);
    assert_eq!(roundtrip.proxy.port, 9999);
    assert!(roundtrip.proxy.auto_cache);
    assert_eq!(roundtrip.proxy.tail_ttl, "10m");
    assert_eq!(roundtrip.proxy.drop_tools, vec!["NotebookEdit"]);
    assert!(roundtrip.proxy.strip_ansi);
    assert!(roundtrip.proxy.trim_bash_git);
    assert_eq!(
        roundtrip.proxy.model_override.as_deref(),
        Some("claude-opus-4-6")
    );
    assert!(roundtrip.proxy.scrub_secrets);
    assert!(roundtrip.proxy.tool_filters);
    assert_eq!(
        roundtrip.proxy.upstream_url.as_deref(),
        Some("http://127.0.0.1:4141")
    );
    assert_eq!(roundtrip.proxy.api_key.as_deref(), Some("secret-key"));
    assert_eq!(roundtrip.proxy.rate_limit_rps, 30.0);
    assert_eq!(roundtrip.proxy.rate_limit_burst, 5.0);
}
