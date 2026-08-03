use super::*;
use serial_test::serial;

#[test]
fn test_default_config() {
    let config = CleanCtxConfig::default();
    assert_eq!(config.default_fidelity, crate::compression::Fidelity::Low);
    assert!(config.diff_compression);
    assert!(config.type_aliases.is_empty());
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
    assert!(CleanCtxConfig::is_ci_environment(), "Should detect GITHUB_ACTIONS");
    
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
    assert!(CleanCtxConfig::is_ci_environment(), "Should detect TF_BUILD");
    
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
    let vars_to_clear = ["CI", "TF_BUILD", "GITHUB_ACTIONS", "GITLAB_CI", "JENKINS_URL", "CIRCLECI", "TRAVIS"];
    let originals: Vec<Option<String>> = vars_to_clear.iter()
        .map(|var| std::env::var(var).ok())
        .collect();
    
    unsafe {
        for var in &vars_to_clear {
            std::env::remove_var(var);
        }
    }
    
    assert!(!CleanCtxConfig::is_ci_environment(), "Should return false when no CI env vars are set");
    
    // Restore original env vars
    unsafe {
        for (var, original) in vars_to_clear.iter().zip(originals) {
            if let Some(val) = original {
                std::env::set_var(var, val);
            }
        }
    }
}

#[serial]
#[test]
fn test_ci_detection_integration() {
    // Test that CI detection works correctly with different env var combinations
    let vars_to_clear = ["CI", "TF_BUILD", "GITHUB_ACTIONS", "GITLAB_CI", "JENKINS_URL", "CIRCLECI", "TRAVIS"];
    let originals: Vec<Option<String>> = vars_to_clear.iter()
        .map(|var| std::env::var(var).ok())
        .collect();
    
    unsafe {
        // Clear all CI vars
        for var in &vars_to_clear {
            std::env::remove_var(var);
        }
        
        // Test 1: No CI vars set → not CI
        assert!(!CleanCtxConfig::is_ci_environment(), "Should not detect CI when no vars set");
        
        // Test 2: CI=true → CI detected
        std::env::set_var("CI", "true");
        assert!(CleanCtxConfig::is_ci_environment(), "Should detect CI=true");
        
        // Test 3: CI=false → not CI (only "true" counts)
        std::env::set_var("CI", "false");
        assert!(!CleanCtxConfig::is_ci_environment(), "CI=false should not trigger CI detection");
        
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
