use super::*;

#[test]
fn hash_detects_change() {
    let mut c = LocalStateCache::new();
    let h1 = c.compute_hash(b"hello");
    assert!(c.update_and_verify("/tmp/a.ts".into(), h1.clone()));
    assert!(!c.update_and_verify("/tmp/a.ts".into(), h1));
    assert!(c.update_and_verify("/tmp/a.ts".into(), c.compute_hash(b"world")));
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