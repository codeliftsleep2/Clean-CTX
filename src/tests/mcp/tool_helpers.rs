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