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
    let config = CleanCtxConfig {
        default_fidelity: "medium".to_string(),
        ..Default::default()
    };
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

// ---------- F-21: diff_code_context cache-hit fast path ----------

/// F-21: calling `diff_code_context` on an unchanged file should
/// return a "No changes" message without re-parsing the source.
#[test]
fn diff_code_context_unchanged_file_skips_reparse() {
    use std::io::Write;
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("sample.ts");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "export class Foo {{ bar(): void {{}} }}").unwrap();
    }

    let mut cache = crate::cache::LocalStateCache::new();

    // Read source for the diff handler (A-08: source_cache integration)
    let source1 = std::fs::read_to_string(&path).unwrap();

    // First call: stores baseline.
    let result1 = diff_code_context_handler(
        path.clone(),
        &source1,
        &mut cache,
        Fidelity::Low,
    )
    .expect("first diff call should succeed");
    assert!(
        result1.contains("No baseline snapshot"),
        "first call should store baseline, got: {}",
        result1
    );

    // Second call (unchanged file): should short-circuit.
    let source2 = std::fs::read_to_string(&path).unwrap();
    let result2 = diff_code_context_handler(
        path.clone(),
        &source2,
        &mut cache,
        Fidelity::Low,
    )
    .expect("second diff call should succeed");
    assert!(
        result2.contains("No changes"),
        "second call on unchanged file should say 'No changes', got: {}",
        result2
    );
}

/// F-21: calling `diff_code_context` after modifying the file should
/// produce a real diff (not the "No changes" fast path).
#[test]
fn diff_code_context_changed_file_produces_diff() {
    use std::io::Write;
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("change.ts");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(f, "export class Alpha {{ run(): void {{}} }}").unwrap();
    }

    let mut cache = crate::cache::LocalStateCache::new();

    let source_before = std::fs::read_to_string(&path).unwrap();

    // First call: stores baseline.
    let _ = diff_code_context_handler(
        path.clone(),
        &source_before,
        &mut cache,
        Fidelity::Low,
    )
    .expect("first diff call should succeed");

    // Modify the file.
    {
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "export class Alpha {{ run(): void {{}} }}\nexport class Beta {{ go(): string {{ return ''; }} }}"
        )
        .unwrap();
    }

    let source_after = std::fs::read_to_string(&path).unwrap();

    // Second call (changed file): should produce a real diff.
    let result = diff_code_context_handler(
        path,
        &source_after,
        &mut cache,
        Fidelity::Low,
    )
    .expect("diff call on changed file should succeed");
    assert!(
        result.contains("AST Diff") && !result.contains("No changes"),
        "changed file should produce a real diff, not a no-change message, got: {}",
        result
    );
}