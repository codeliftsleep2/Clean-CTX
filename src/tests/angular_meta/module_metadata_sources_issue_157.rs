use super::Contributors;

#[test]
fn issue_157_snapshot_hash_certifies_consumed_bytes_and_repeated_reads_are_stable() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("leaf.ts");
    let before = "export const VALUE = 1;";
    std::fs::write(&path, before).unwrap();
    let mut contributors = Contributors::default();
    assert_eq!(contributors.read(&path).unwrap().as_ref(), before);
    std::fs::write(&path, "export const VALUE = 2;").unwrap();
    assert_eq!(contributors.read(&path).unwrap().as_ref(), before);
    let dependencies = contributors.dependencies();
    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].source_hash.as_deref(),
        Some(crate::cache::content_hash(before.as_bytes()).as_str())
    );
    assert!(!dependencies[0].is_current());
}
