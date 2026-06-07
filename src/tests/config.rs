use super::*;

#[test]
fn test_default_config() {
    let config = CleanCtxConfig::default();
    assert_eq!(config.default_fidelity, "low");
    assert!(config.diff_compression);
    assert!(config.type_aliases.is_empty());
}

#[test]
fn test_exclusion() {
    let mut config = CleanCtxConfig::default();
    config.exclude_patterns.push("node_modules".to_string());
    config.exclude_patterns.push(".test.".to_string());

    assert!(config.is_excluded("src/node_modules/file.ts"));
    assert!(config.is_excluded("src/file.test.ts"));
    assert!(!config.is_excluded("src/file.ts"));
}