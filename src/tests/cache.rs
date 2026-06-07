use super::*;

#[test]
fn hash_detects_change() {
    let mut c = LocalStateCache::new();
    let h1 = c.compute_hash(b"hello");
    assert!(c.update_and_verify("/tmp/a.ts", &h1));
    assert!(!c.update_and_verify("/tmp/a.ts", &h1));
    let h2 = c.compute_hash(b"world");
    assert!(c.update_and_verify("/tmp/a.ts", &h2));
}

/// F-14: raw-token count round-trip.
#[test]
fn raw_token_count_round_trip() {
    let mut c = LocalStateCache::new();
    let hash = c.compute_hash(b"source code");
    assert_eq!(c.get_raw_token_count(&hash), None);
    c.store_raw_token_count(&hash, 42);
    assert_eq!(c.get_raw_token_count(&hash), Some(42));
    // Same hash → same count (content-addressed).
    assert_eq!(c.get_raw_token_count(&hash), Some(42));
}

/// F-14: clear() must wipe raw-token counts too.
#[test]
fn clear_removes_raw_token_counts() {
    let mut c = LocalStateCache::new();
    let hash = c.compute_hash(b"test");
    c.store_raw_token_count(&hash, 10);
    assert_eq!(c.get_raw_token_count(&hash), Some(10));
    c.clear();
    assert_eq!(c.get_raw_token_count(&hash), None);
}

#[test]
fn baseline_round_trip() {
    let mut c = LocalStateCache::new();
    let snap = CapturedStructure {
        imports: vec!["A".into()],
        classes: vec![],
        orphan_fields: vec![],
    };
    c.store_baseline("k".into(), snap.clone());
    assert_eq!(c.get_baseline("k"), Some(&snap));
    c.invalidate_baseline("k");
    assert!(c.get_baseline("k").is_none());
}