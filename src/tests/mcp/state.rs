// src/tests/mcp/state.rs
//
// Tests for McpState: creation, warnings, source cache

use super::*;

#[test]
fn state_new_creates_empty_registries() {
    let state = McpState::new(crate::tests::test_config());
    // Warnings should be empty
    assert!(state.drain_warnings().is_empty());
}

#[test]
fn push_and_drain_warnings() {
    let state = McpState::new(crate::tests::test_config());
    state.push_warning("test warning 1");
    state.push_warning("test warning 2");
    assert_eq!(state.warnings.lock().unwrap().len(), 2);
    let drained = state.drain_warnings();
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0], "test warning 1");
    assert_eq!(drained[1], "test warning 2");
    // After draining, warnings should be empty
    assert!(state.warnings.lock().unwrap().is_empty());
}

#[test]
fn drain_warnings_on_empty_returns_empty() {
    let state = McpState::new(crate::tests::test_config());
    let drained = state.drain_warnings();
    assert!(drained.is_empty());
}

#[test]
fn read_source_caches_file_content() {
    use std::sync::Arc;
    let state = McpState::new(crate::tests::test_config());
    // Read a known file
    let result = state.read_source("src/lib.rs");
    assert!(result.is_ok());
    let content = result.unwrap();
    assert!(!content.is_empty());
    // Reading again should come from cache
    let content2 = state.read_source("src/lib.rs").unwrap();
    // Both should be the same Arc (pointer equality means cache hit)
    assert!(Arc::ptr_eq(&content, &content2));
}

#[test]
fn read_source_nonexistent_file_returns_error() {
    let state = McpState::new(crate::tests::test_config());
    let result = state.read_source("/nonexistent/file/path.rs");
    assert!(result.is_err());
}

#[test]
fn state_accessor_mut_methods() {
    let state = McpState::new(crate::tests::test_config());
    // Verify accessor methods return the correct types
    let _dict = state.dict_lock().get_or_create_alias("test.rs".to_string());
    let _cache = state.cache_write();
    let _ir = state.ir_context_lock();
    let _td = &state.text_delta;
}

// ── Non-CBM Tool Audit 2026-08-25, finding #3 ────────────────────────
//
// Alias identity invariant: ONE physical file must map to ONE stable
// alias regardless of the path form the caller supplies. Previously the
// alias key was the raw caller string, so an absolute path and an
// equivalent path containing a redundant segment produced two separate
// aliases (visible as duplicate `α` entries in `§PATHMAP`) and silently
// fragmented every alias-keyed state (IR context, text-delta baselines,
// LLM text cache).

#[test]
fn alias_identity_absolute_and_redundant_segment_forms_converge() {
    use tempfile::TempDir;

    let tmp = TempDir::new().expect("tempdir");
    let dir = tmp.path().join("src");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.rs");
    std::fs::write(&file, "fn main() {}\n").unwrap();

    let state = McpState::new(crate::tests::test_config());

    // Form A: plain absolute path.
    let abs_form = file.to_string_lossy().to_string();
    // Form B: same physical file reached through a redundant segment.
    //
    // The intermediate `placeholder` directory MUST actually exist:
    // POSIX `realpath()` (what `fs::canonicalize` uses on Linux/macOS)
    // resolves EVERY intermediate component and fails with ENOENT on
    // phantom ones, while Windows path normalization collapses `..\`
    // lexically. A real directory keeps the fixture cross-platform and
    // matches the realistic caller shape (root-joined relative paths).
    let placeholder_dir = dir.join("placeholder");
    std::fs::create_dir_all(&placeholder_dir).unwrap();
    let redundant_form = placeholder_dir
        .join("..")
        .join("main.rs")
        .to_string_lossy()
        .to_string();
    assert_ne!(
        abs_form, redundant_form,
        "fixture forms must differ as strings"
    );

    let alias_a = state.get_or_create_alias(abs_form);
    let alias_b = state.get_or_create_alias(redundant_form);
    assert_eq!(
        alias_a, alias_b,
        "one physical file must have one stable alias"
    );
}

#[test]
fn alias_identity_unresolvable_paths_fall_back_to_raw_key() {
    // Documented fallback: when canonicalization is impossible (path does
    // not exist), the raw string remains the key — distinct strings stay
    // distinct aliases instead of colliding or panicking.
    let state = McpState::new(crate::tests::test_config());
    let a = state.get_or_create_alias("/nonexistent/alpha.rs".to_string());
    let b = state.get_or_create_alias("/nonexistent/beta.rs".to_string());
    assert_ne!(a, b);
    assert_eq!(
        a,
        state.get_or_create_alias("/nonexistent/alpha.rs".to_string())
    );
}
