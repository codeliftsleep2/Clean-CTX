use super::*;

#[test]
fn test_default_config() {
    let config = CleanCtxConfig::default();
    assert_eq!(config.default_fidelity, "low");
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
