use super::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_ts_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    let mut f = std::fs::File::create(&path).unwrap();
    write!(f, "{}", content).unwrap();
    path
}

#[test]
fn assemble_body_uses_semicolon_at_low() {
    let lines = vec!["a".to_string(), "b".to_string()];
    assert_eq!(assemble_body(&lines, Fidelity::Low), "a;b");
}

#[test]
fn assemble_body_uses_newline_at_medium() {
    let lines = vec!["a".to_string(), "b".to_string()];
    assert_eq!(assemble_body(&lines, Fidelity::Medium), "a\nb");
}

#[test]
fn assemble_body_uses_newline_at_high() {
    let lines = vec!["a".to_string(), "b".to_string()];
    assert_eq!(assemble_body(&lines, Fidelity::High), "a\nb");
}

#[test]
fn compress_file_cache_hit_returns_notice() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "export class Foo {}\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();
    let fidelity = Fidelity::Low;

    let result1 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(result1.is_ok(), "compress_file (miss) should succeed, got: {:?}", result1);

    let result2 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(result2.is_ok(), "compress_file (hit) should succeed");
    let output2 = result2.unwrap();

    assert!(
        output2.contains("[CACHE_HIT]"),
        "expected CACHE_HIT notice, got: {}",
        output2
    );

    create_ts_file(&dir, "test.ts", "export class Bar {}");
    let result3 = compress_file(path, &mut dict, &mut cache, fidelity, None);
    assert!(result3.is_ok(), "compress_file (modified) should succeed");
    let output3 = result3.unwrap();
    assert!(
        !output3.contains("[CACHE_HIT]"),
        "modified file should NOT hit cache, got: {}",
        output3
    );

    assert!(
        output3.contains("Compacted Layout"),
        "expected a full compacted output after modification, got: {}",
        output3
    );
}

#[test]
fn compress_file_cache_hit_vs_miss_output_differ() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "class A { foo():void {} }\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();
    let fidelity = Fidelity::Medium;

    let result1 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(result1.is_ok(), "compress_file (first call) should succeed");

    let result2 = compress_file(path.clone(), &mut dict, &mut cache, fidelity, None);
    assert!(result2.is_ok());
    let second = result2.unwrap();

    assert!(
        second.contains("[CACHE_HIT]"),
        "Second call with same content should hit cache"
    );
}

// ---------- F-18: file size guard ----------

#[test]
fn compress_file_rejects_file_larger_than_max() {
    // Create a .ts file that exceeds the 10 MB limit.
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("huge.ts");
    {
        let mut f = std::fs::File::create(&path).unwrap();
        // Write just over 10 MB of content (valid UTF-8).
        let chunk = "export class X {}\n";
        let repetitions = (10 * 1024 * 1024 / chunk.len()) + 2;
        for _ in 0..repetitions {
            f.write_all(chunk.as_bytes()).unwrap();
        }
    }

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let result = compress_file(path, &mut dict, &mut cache, Fidelity::Low, None);
    assert!(result.is_err(), "should reject oversized file");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("File too large"),
        "error should mention 'File too large', got: {}",
        err_msg
    );
}

// ---------- R-02: Type-alias integration tests ----------

fn make_aliases(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn compress_text_with_aliases_medium_fidelity() {
    // At Medium fidelity, type names in method signatures should be
    // substituted with alias tokens.
    let source = "class UserService { getUser(id: string): Promise<User> {} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let result = compress_text(source, "ts", Fidelity::Medium, "α1", Some(&aliases));
    assert!(result.is_ok(), "compress_text with aliases should succeed");
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        full_output.contains("$uid"),
        "output should contain $uid alias, got: {}",
        full_output
    );
    assert!(
        full_output.contains("§TA"),
        "output should contain §TA footer, got: {}",
        full_output
    );
}

#[test]
fn compress_text_with_aliases_high_fidelity() {
    let source = "class UserService { getUser(id: string): Promise<User> {} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let result = compress_text(source, "ts", Fidelity::High, "α1", Some(&aliases));
    assert!(result.is_ok());
    let (_body_lines, full_output) = result.unwrap();
    assert!(full_output.contains("$uid"), "output should contain $uid alias");
    assert!(full_output.contains("§TA"), "output should contain §TA footer");
}

#[test]
fn compress_text_without_aliases_no_footer() {
    // When no aliases are configured, no §TA footer should appear.
    let source = "class UserService { getUser(id: string): Promise<User> {} }";
    let result = compress_text(source, "ts", Fidelity::Medium, "α1", None);
    assert!(result.is_ok());
    let (_body_lines, full_output) = result.unwrap();
    assert!(
        !full_output.contains("§TA"),
        "output should NOT contain §TA footer when no aliases configured"
    );
}

#[test]
fn compress_text_aliases_deterministic() {
    // Same input + same aliases → same output (delta consistency).
    let source = "class Service { get(id: string): Promise<User> {} }";
    let aliases = make_aliases(&[("User", "$uid")]);
    let r1 = compress_text(source, "ts", Fidelity::Medium, "α1", Some(&aliases)).unwrap();
    let r2 = compress_text(source, "ts", Fidelity::Medium, "α1", Some(&aliases)).unwrap();
    assert_eq!(r1.0, r2.0, "body lines should be identical");
    assert_eq!(r1.1, r2.1, "full output should be identical");
}
