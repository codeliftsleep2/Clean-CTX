use super::*;
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

    let result1 = compress_file(path.clone(), &mut dict, &mut cache, fidelity);
    assert!(result1.is_ok(), "compress_file (miss) should succeed, got: {:?}", result1);

    let result2 = compress_file(path.clone(), &mut dict, &mut cache, fidelity);
    assert!(result2.is_ok(), "compress_file (hit) should succeed");
    let output2 = result2.unwrap();

    assert!(
        output2.contains("[CACHE_HIT]"),
        "expected CACHE_HIT notice, got: {}",
        output2
    );

    create_ts_file(&dir, "test.ts", "export class Bar {}");
    let result3 = compress_file(path, &mut dict, &mut cache, fidelity);
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

    let result1 = compress_file(path.clone(), &mut dict, &mut cache, fidelity);
    assert!(result1.is_ok(), "compress_file (first call) should succeed");

    let result2 = compress_file(path.clone(), &mut dict, &mut cache, fidelity);
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

    let result = compress_file(path, &mut dict, &mut cache, Fidelity::Low);
    assert!(result.is_err(), "should reject oversized file");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("File too large"),
        "error should mention 'File too large', got: {}",
        err_msg
    );
}
