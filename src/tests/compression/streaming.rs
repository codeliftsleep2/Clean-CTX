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
fn streaming_callback_receives_initial_phase() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "class Simple { foo():void {} }\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let phases = std::sync::Mutex::new(Vec::new());
    let result = compress_file_streaming(
        path,
        &mut dict,
        &mut cache,
        Fidelity::Low,
        4096,
        None,
        |progress: CompressionProgress| {
            phases.lock().unwrap().push(progress.phase.clone());
            Ok(())
        },
    );
    assert!(result.is_ok(), "streaming compress should succeed, got: {:?}", result);

    let recorded = phases.lock().unwrap();
    assert!(!recorded.is_empty(), "should have emitted at least one progress event");
    assert_eq!(recorded[0], "reading", "first phase should be 'reading'");
}

#[test]
fn streaming_callback_sees_done_phase_at_end() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "class Simple { foo():void {} }\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let phases = std::sync::Mutex::new(Vec::new());
    let result = compress_file_streaming(
        path,
        &mut dict,
        &mut cache,
        Fidelity::Low,
        4096,
        None,
        |progress: CompressionProgress| {
            phases.lock().unwrap().push(progress.phase.clone());
            Ok(())
        },
    );

    assert!(result.is_ok(), "streaming compress should succeed");
    let recorded = phases.lock().unwrap();
    let last = recorded.last().expect("should have at least one progress event");
    assert_eq!(last, "done", "last phase should be 'done', got: {}", last);
}

#[test]
fn streaming_callback_progress_monotonic() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "class Simple { foo():void {} }\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let vals = std::sync::Mutex::new(Vec::new());
    let result = compress_file_streaming(
        path,
        &mut dict,
        &mut cache,
        Fidelity::Low,
        4096,
        None,
        |progress: CompressionProgress| {
            vals.lock().unwrap().push(progress.progress);
            Ok(())
        },
    );

    assert!(result.is_ok(), "streaming compress should succeed");
    let recorded = vals.lock().unwrap();
    for window in recorded.windows(2) {
        assert!(
            window[0] <= window[1],
            "progress values must be non-decreasing: {} > {}",
            window[0],
            window[1]
        );
    }
    assert!(
        (*recorded.last().unwrap() - 1.0_f64).abs() < 1e-9,
        "final progress should be ~1.0, got: {}",
        recorded.last().unwrap()
    );
}

#[test]
fn streaming_callback_error_stops_pipeline() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "class Simple { foo():void {} }\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let result = compress_file_streaming(
        path,
        &mut dict,
        &mut cache,
        Fidelity::Low,
        4096,
        None,
        |_: CompressionProgress| Err("user-aborted".into()),
    );

    assert!(result.is_err(), "streaming compress should propagate callback error");
    let err = result.unwrap_err();
    assert!(
        format!("{:?}", err).contains("user-aborted"),
        "error should contain the callback message"
    );
}

#[test]
fn streaming_cache_hit_receives_cache_hit_phase() {
    let dir = TempDir::new().unwrap();
    let path = create_ts_file(&dir, "test.ts", "class Simple { foo():void {} }\n");

    let mut dict = crate::dictionary::PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let _first = compress_file_streaming(
        path.clone(),
        &mut dict,
        &mut cache,
        Fidelity::Low,
        4096,
        None,
        |_: CompressionProgress| Ok(()),
    );

    let phases2 = std::sync::Mutex::new(Vec::new());
    let result = compress_file_streaming(
        path,
        &mut dict,
        &mut cache,
        Fidelity::Low,
        4096,
        None,
        |progress: CompressionProgress| {
            phases2.lock().unwrap().push(progress.phase.clone());
            Ok(())
        },
    );

    assert!(result.is_ok(), "streaming compress (cache hit) should succeed");
    let recorded = phases2.lock().unwrap();
    assert!(
        recorded.contains(&"cache-hit".to_string()),
        "expected 'cache-hit' phase in {:?}",
        recorded
    );
}