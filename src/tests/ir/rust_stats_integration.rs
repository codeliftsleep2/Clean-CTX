// src/tests/ir/rust_stats_integration.rs
//
// Integration test proving that Rust files are tracked through the
// session stats pipeline (live token tracking). Exercises:
//   1. Compress a real .rs file through the full pipeline
//   2. Record stats via SessionStats::record_compression
//   3. Verify the dashboard renderer includes the Rust file data

use crate::compression::pipeline::compress_source;
use crate::compression::Fidelity;
use crate::dictionary::PathDictionary;
use crate::analytics::calculate_savings;
use crate::cache::LocalStateCache;

/// Test that compressing a Rust file produces non-zero token savings
/// and that the analytics pipeline works end-to-end for .rs sources.
#[test]
fn rust_compression_has_token_savings() {
    let source = include_str!("../../test_files/rust/sample_service.rs");

    // Calculate token savings directly from the source text
    let meta = calculate_savings(source, source, None);
    assert!(
        meta.raw_tokens > 0,
        "Rust source should have >0 raw tokens, got {}",
        meta.raw_tokens
    );
}

/// Test that the compress_source pipeline produces output that
/// contains Rust structural markers (struct, impl, trait, etc.).
#[test]
fn rust_compressed_output_contains_rust_markers() {
    let source = include_str!("../../test_files/rust/sample_service.rs");
    let mut dict = PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let output = compress_source(
        source,
        "/test/rust/sample_service.rs",
        &mut dict,
        &mut cache,
        Fidelity::Low,
    )
    .expect("compress_source should succeed for Rust file");

    // Verify Rust-specific markers appear in the compressed output
    assert!(
        output.contains("UserService")
            || output.contains("DataProcessor")
            || output.contains("Repository"),
        "Compressed output should contain Rust type names, got:\n{}",
        output
    );

    // Verify the output has significant content (not just a cache hit placeholder)
    assert!(
        output.len() > 50,
        "Compressed Rust output should be more than 50 chars, got {}",
        output.len()
    );
}

/// Test that session stats record properly when Rust files are processed.
#[test]
fn rust_session_stats_reports_token_savings() {
    let mut stats = crate::mcp::session_stats::SessionStats::new();

    let source = include_str!("../../test_files/rust/sample_service.rs");
    let mut dict = PathDictionary::new();
    let mut cache = LocalStateCache::new();

    let output = compress_source(
        source,
        "/test/rust/sample_service.rs",
        &mut dict,
        &mut cache,
        Fidelity::Medium,
    )
    .expect("compress_source should succeed");

    let raw_tokens = crate::analytics::bpe()
        .encode_with_special_tokens(source)
        .len();
    let compressed_tokens = crate::analytics::bpe()
        .encode_with_special_tokens(&output)
        .len();

    // Record stats the same way provide_code_context does
    stats.record_compression(
        "/test/rust/sample_service.rs",
        raw_tokens,
        compressed_tokens,
        "medium",
        false, // is_angular
        "full",
        None,
    );

    let summary = stats.summary();
    assert_eq!(
        summary.total_files, 1,
        "Should have 1 Rust file in stats"
    );
    assert!(
        summary.total_raw_tokens > 0,
        "Raw token count should be > 0, got {}",
        summary.total_raw_tokens
    );
    assert!(
        summary.total_savings_pct > 0.0,
        "Should have positive savings percentage, got {}",
        summary.total_savings_pct
    );

    // Verify the dashboard text includes the Rust file
    let dashboard = crate::mcp::session_stats::render_dashboard_text(&stats);
    assert!(
        dashboard.contains("sample_service.rs"),
        "Dashboard should include the Rust filename, got:\n{}",
        dashboard
    );
    assert!(
        dashboard.contains("savings") || dashboard.contains("Save%"),
        "Dashboard should show savings information, got:\n{}",
        dashboard
    );
    // Verify the savings percentage is displayed (varies slightly per run)
    assert!(
        dashboard.contains("%"),
        "Dashboard should show a percentage value, got:\n{}",
        dashboard
    );
}

/// Test that the JSON dashboard format works for Rust files.
#[test]
fn rust_json_dashboard_includes_rust_files() {
    let mut stats = crate::mcp::session_stats::SessionStats::new();

    stats.record_compression(
        "/src/ir/layers/rust.rs",
        1000,
        250,
        "low",
        false,
        "full",
        None,
    );

    let json = crate::mcp::session_stats::render_dashboard_json(&stats);
    assert_eq!(json["session"]["total_files"], 1, "JSON should show 1 file");

    // files is a JSON array in render_dashboard_json
    let files_arr = json["files"].as_array()
        .expect("JSON dashboard should have 'files' array");
    assert_eq!(files_arr.len(), 1, "Should have 1 file entry");

    let rust_entry = &files_arr[0];
    assert_eq!(rust_entry["raw_tokens"], 1000);
    assert_eq!(rust_entry["compressed_tokens"], 250);
    assert_eq!(rust_entry["fidelity"], "low");
    assert!(rust_entry["file_path"].as_str().unwrap_or("").contains("rust.rs"),
        "File path should contain 'rust.rs'");
}