// src/tests/mcp/session_stats.rs
//
// Tests for SessionStats

use crate::mcp::session_stats::{SessionStats, render_dashboard_text, render_dashboard_json};

#[test]
fn test_session_stats_record() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full");
    
    let file_stats = stats.file_stats("/test/file.ts");
    assert!(file_stats.is_some());
    let fs = file_stats.unwrap();
    assert_eq!(fs.raw_tokens, 1000);
    assert_eq!(fs.compressed_tokens, 250);
    assert!((fs.savings_pct - 75.0).abs() < 0.01);
    assert_eq!(fs.version, 1);
    assert_eq!(fs.strategy, "full");
}

#[test]
fn test_session_stats_summary() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/a.ts", 1000, 250, "low", false, "full");
    stats.record_compression("/test/b.ts", 2000, 500, "medium", true, "delta");
    
    let summary = stats.summary();
    assert_eq!(summary.total_files, 2);
    assert_eq!(summary.total_raw_tokens, 3000); // 1000 + 2000
    assert_eq!(summary.total_compressed_tokens, 750); // 250 + 500
    assert!(summary.total_savings_pct > 0.0);
    assert_eq!(summary.full_compress_count, 1);
    assert_eq!(summary.delta_count, 1);
    assert_eq!(summary.delta_hit_rate, 50.0);
}

#[test]
fn test_empty_stats() {
    let stats = SessionStats::new();
    let summary = stats.summary();
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.total_raw_tokens, 0);
    assert_eq!(summary.total_compressed_tokens, 0);
    assert_eq!(summary.delta_hit_rate, 0.0);
}

#[test]
fn test_render_dashboard_text() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full");
    let text = render_dashboard_text(&stats);
    assert!(text.contains("Clean-CTX Dashboard"));
    assert!(text.contains("1,000"));
    assert!(text.contains("250"));
}

#[test]
fn test_render_dashboard_json() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full");
    let json = render_dashboard_json(&stats);
    assert_eq!(json["session"]["total_files"], 1);
    assert_eq!(json["session"]["total_raw_tokens"], 1000);
    assert_eq!(json["files"].as_array().unwrap().len(), 1);
}

#[test]
fn test_multiple_records_same_file() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full");
    stats.record_compression("/test/file.ts", 1000, 200, "low", false, "delta");
    
    let fs = stats.file_stats("/test/file.ts").unwrap();
    assert_eq!(fs.version, 2);
    assert_eq!(fs.delta_count, 1);
}