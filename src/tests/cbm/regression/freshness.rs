// These tests verify the per-project dirty/lazy reindex behavior.
// They use mock GraphBridge instances (no real CBM needed).

#[test]
fn mark_project_dirty_creates_freshness_entry() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use std::path::Path;

    let mut bridge = new_mock_empty();
    bridge.mark_project_dirty(Path::new("."));

    let f_map = bridge.freshness.lock().unwrap_or_else(|p| p.into_inner());
    let entry = f_map
        .get("test-project")
        .expect("freshness entry must exist");
    assert_eq!(entry.dirty_generation, 1, "first dirty mark bumps to 1");
    assert_eq!(entry.indexed_generation, 0, "indexed must remain 0");
}

#[test]
fn mark_project_dirty_bumps_generation_on_second_call() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use std::path::Path;

    let mut bridge = new_mock_empty();
    bridge.mark_project_dirty(Path::new("."));
    bridge.mark_project_dirty(Path::new("."));

    let f_map = bridge.freshness.lock().unwrap_or_else(|p| p.into_inner());
    let entry = f_map
        .get("test-project")
        .expect("freshness entry must exist");
    assert_eq!(entry.dirty_generation, 2, "second dirty mark bumps to 2");
    assert_eq!(entry.indexed_generation, 0, "indexed must remain 0");
}

#[test]
fn is_project_dirty_returns_true_when_dirty_gen_exceeds_indexed() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use std::path::Path;

    let mut bridge = new_mock_empty();
    bridge.mark_project_dirty(Path::new("."));

    assert!(
        bridge.is_project_dirty("test-project"),
        "must be dirty after mark"
    );
}

#[test]
fn is_project_dirty_returns_false_when_generations_equal() {
    use crate::cbm::bridge::ProjectFreshness;
    use crate::cbm::bridge::test_helpers::new_mock_empty;

    let bridge = new_mock_empty();
    {
        let mut f_map = bridge.freshness.lock().unwrap_or_else(|p| p.into_inner());
        f_map.insert(
            "test-project".to_string(),
            ProjectFreshness {
                dirty_generation: 3,
                indexed_generation: 3,
            },
        );
    }
    assert!(
        !bridge.is_project_dirty("test-project"),
        "must be clean when equal"
    );
}

#[test]
fn is_project_dirty_returns_false_when_no_entry() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;

    let bridge = new_mock_empty();
    assert!(
        !bridge.is_project_dirty("unknown-project"),
        "unknown must be clean"
    );
}

#[test]
fn mark_project_dirty_falls_back_to_active_root() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use std::path::Path;

    let mut bridge = new_mock_empty();
    bridge.mark_project_dirty(Path::new("Z:\\nonexistent\\path\\file.rs"));

    let f_map = bridge.freshness.lock().unwrap_or_else(|p| p.into_inner());
    let entry = f_map.get("test-project").expect("must fall back to active");
    assert_eq!(entry.dirty_generation, 1, "fallback must create entry");
}

#[test]
fn multi_project_independent_freshness() {
    use crate::cbm::bridge::ProjectFreshness;
    use crate::cbm::bridge::test_helpers::new_mock_empty;

    let bridge = new_mock_empty();
    {
        let mut f_map = bridge.freshness.lock().unwrap_or_else(|p| p.into_inner());
        f_map.insert(
            "project-a".to_string(),
            ProjectFreshness {
                dirty_generation: 3,
                indexed_generation: 3,
            },
        );
        f_map.insert(
            "project-b".to_string(),
            ProjectFreshness {
                dirty_generation: 2,
                indexed_generation: 1,
            },
        );
    }
    assert!(
        !bridge.is_project_dirty("project-a"),
        "project-a must be clean"
    );
    assert!(
        bridge.is_project_dirty("project-b"),
        "project-b must be dirty"
    );
}

#[test]
fn ensure_indexed_skips_reindex_when_project_not_dirty() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;

    let mut bridge = new_mock_empty();
    let result = bridge.ensure_indexed();
    assert!(
        result.is_ok(),
        "clean project ensure_indexed must not trigger reindex"
    );
    match result.unwrap() {
        crate::cbm::bridge::IndexingStatus::Ready => {}
        other => panic!("expected Ready, got: {other:?}"),
    }
}

#[test]
fn dirty_flag_survives_reindex_failure() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use std::path::Path;

    let mut bridge = new_mock_empty();
    bridge.mark_project_dirty(Path::new("."));
    assert!(
        bridge.is_project_dirty("test-project"),
        "must be dirty before"
    );

    let result = bridge.reindex_active_project("fast");
    assert!(result.is_err(), "reindex must fail with no CBM client");

    assert!(
        bridge.is_project_dirty("test-project"),
        "must remain dirty after failure"
    );
    let f_map = bridge.freshness.lock().unwrap_or_else(|p| p.into_inner());
    let entry = f_map.get("test-project").expect("entry must exist");
    assert_eq!(entry.indexed_generation, 0, "must not advance on failure");
}

#[test]
fn apply_edit_marks_dirty_instead_of_reindexing() {
    use crate::cbm::bridge::test_helpers::new_mock_empty;
    use std::path::Path;

    let mut bridge = new_mock_empty();
    assert!(
        !bridge.is_project_dirty("test-project"),
        "must be clean before edit"
    );

    bridge.mark_project_dirty(Path::new("."));

    assert!(
        bridge.is_project_dirty("test-project"),
        "must be dirty after mark"
    );
}
