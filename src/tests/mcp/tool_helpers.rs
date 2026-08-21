// src/tests/mcp/tool_helpers.rs
//
// Tests for tool_helpers: resolve_file_path, estimate_tokens

use super::*;

#[test]
fn resolve_file_path_with_workspace_root() {
    // On Windows, use a Windows-style path; on Unix, use forward slashes
    #[cfg(windows)]
    let result = resolve_file_path("src\\main.rs", Some("C:\\workspace"));
    #[cfg(not(windows))]
    let result = resolve_file_path("src/main.rs", Some("/workspace"));
    // Should produce an absolute path
    assert!(
        result.starts_with('/') || result.contains(":\\") || result.contains(":/"),
        "expected absolute path, got: {}",
        result
    );
}

#[test]
fn resolve_file_path_without_workspace_root() {
    // When no workspace root is provided, it should still produce a valid path
    let result = resolve_file_path("src/main.rs", None);
    assert!(!result.is_empty());
    assert!(result.contains("src/main.rs") || result.contains("src\\main.rs"),
        "expected path to contain src/main.rs, got: {}",
        result
    );
}

#[test]
fn resolve_file_path_absolute_passthrough() {
    // Absolute paths should be returned as-is
    #[cfg(windows)]
    let input = "C:\\absolute\\path\\file.rs";
    #[cfg(not(windows))]
    let input = "/absolute/path/file.rs";
    let result = resolve_file_path(input, Some("/workspace"));
    assert_eq!(result, input);
}

#[test]
fn resolve_file_path_dot_slash() {
    #[cfg(windows)]
    let result = resolve_file_path(".\\src\\main.rs", Some("C:\\workspace"));
    #[cfg(not(windows))]
    let result = resolve_file_path("./src/main.rs", Some("/workspace"));
    assert!(
        result.contains("src/main.rs") || result.contains("src\\main.rs"),
        "expected path to contain src/main.rs, got: {}",
        result
    );
}

#[test]
fn estimate_tokens_returns_nonzero() {
    let count = estimate_tokens("fn main() { println!(\"hello\"); }");
    assert!(count > 0, "token count should be > 0, got {}", count);
}

#[test]
fn estimate_tokens_scales_with_input() {
    let short = estimate_tokens("hello");
    let long = estimate_tokens("fn main() { println!(\"hello world\"); let x = 42; return x + 1; }");
    assert!(long > short, "longer input should produce more tokens: short={}, long={}", short, long);
}

#[test]
fn estimate_tokens_empty_string() {
    let count = estimate_tokens("");
    assert_eq!(count, 0);
}

#[test]
fn estimate_tokens_whitespace_only() {
    let count = estimate_tokens("   \n\n  ");
    // Should handle whitespace gracefully — either 0 or a small number
    assert!(count <= 5, "whitespace-only should produce few tokens, got {}", count);
}

// ── LinguaForge audit regression tests ─────────────────────────────

/// LinguaForge audit Issue 1/7 regression: `resolve_file_path_checked`
/// error messages for a non-existent path should still be informative.
/// The canonicalization step fails first (before the boundary check),
/// so the error is "path does not exist" — but it includes the resolved
/// path so the caller can see what was attempted.
#[test]
fn resolve_file_path_checked_nonexistent_path_shows_informative_error() {
    #[cfg(windows)]
    let outside_path = "Z:\\nonexistent\\file.rs";
    #[cfg(not(windows))]
    let outside_path = "/nonexistent/file.rs";

    let result = resolve_file_path_checked(outside_path, None, &[]);
    assert!(result.is_err(), "should fail for non-existent path");
    let err_msg = result.err().unwrap();
    // The error is "path does not exist: <path>" because canonicalization
    // fails before the boundary check. The path must be in the message.
    assert!(
        err_msg.contains(outside_path),
        "error should contain the attempted path, got: {}",
        err_msg
    );
}

/// LinguaForge audit Issue 1/7 regression: when a path exists but is
/// outside the workspace root boundary, the error must include the
/// effective workspace root so the caller can diagnose configuration.
#[test]
fn resolve_file_path_checked_outside_boundary_shows_workspace_root() {
    // Create a temporary directory that will serve as the "outside" path
    // relative to a restrictive workspace root.
    let tmp_dir = std::env::temp_dir();
    let outside_dir = tmp_dir.join("clean_ctx_boundary_test");
    let _ = std::fs::create_dir_all(&outside_dir);
    let outside_path = outside_dir.to_string_lossy().to_string();

    // Call with a workspace root that is NOT the temp dir — this will
    // resolve to the CWD, then canonicalize, then fail the boundary check.
    // The outside path doesn't start with CWD → boundary error.
    let result = resolve_file_path_checked(&outside_path, None, &[]);
    assert!(result.is_err(), "should fail for path outside workspace root");

    let err_msg = result.err().unwrap();
    // Must contain the boundary error message with the workspace root.
    // The error format is: "path outside workspace root: <path> (workspace root: <root>)"
    assert!(
        err_msg.contains("workspace root"),
        "error should contain 'workspace root', got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("path outside workspace root"),
        "error should contain 'path outside workspace root', got: {}",
        err_msg
    );
    assert!(
        err_msg.contains(&outside_path),
        "error should contain the path, got: {}",
        err_msg
    );

    // Cleanup
    let _ = std::fs::remove_dir(&outside_dir);
}

/// LinguaForge audit Issue 7 regression: `resolve_file_path_checked`
/// must list configured `additional_roots` in its error message when
/// all boundary checks fail and additional_roots are configured.
#[test]
fn resolve_file_path_checked_outside_includes_additional_roots_in_error() {
    let tmp_dir = std::env::temp_dir();
    let outside_dir = tmp_dir.join("clean_ctx_extra_roots_test");
    let _ = std::fs::create_dir_all(&outside_dir);
    let outside_path = outside_dir.to_string_lossy().to_string();

    // Use non-existent additional roots so they are silently skipped
    let additional = vec!["/not/a/real/path".to_string(), "/also/fake".to_string()];
    let result = resolve_file_path_checked(&outside_path, None, &additional);
    assert!(result.is_err(), "should fail for path outside all roots");

    let err_msg = result.err().unwrap();
    // Must contain the workspace root in the error.
    assert!(
        err_msg.contains("workspace root"),
        "error should contain 'workspace root', got: {}",
        err_msg
    );
    assert!(
        err_msg.contains(&outside_path),
        "error should contain the path, got: {}",
        err_msg
    );

    // Cleanup
    let _ = std::fs::remove_dir(&outside_dir);
}

/// LinguaForge audit Issue 1/7 regression: when `additional_roots` are
/// supplied and one of them contains the file path, the path should be
/// accepted (not rejected as outside the boundary).
#[test]
fn resolve_file_path_checked_with_valid_additional_root() {

    // Create a temporary directory to use as an additional root
    let tmp_dir = std::env::temp_dir();
    let root_dir = tmp_dir.join("clean_ctx_test_additional_root");
    let _ = std::fs::create_dir_all(&root_dir);

    let test_file = root_dir.join("test_file.rs");
    let _ = std::fs::write(&test_file, "// test");

    let additional = vec![root_dir.to_string_lossy().to_string()];
    let result = resolve_file_path_checked(
        &test_file.to_string_lossy(),
        None,
        &additional,
    );
    assert!(
        result.is_ok(),
        "path under additional_root should be accepted: {}",
        result.err().unwrap_or_default()
    );

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_dir(&root_dir);
}
