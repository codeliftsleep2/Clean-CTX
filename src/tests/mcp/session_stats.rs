// src/tests/mcp/session_stats.rs
//
// Tests for SessionStats

use crate::mcp::session_stats::{SessionStats, render_dashboard_text, render_dashboard_json};

#[test]
fn test_session_stats_record() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    
    let file_stats = stats.file_stats("/test/file.ts");
    assert!(file_stats.is_some());
    let fs = file_stats.unwrap();
    assert_eq!(fs.raw_tokens, 1000);
    assert_eq!(fs.compressed_tokens, 250);
    assert!((fs.savings_pct - 75.0).abs() < 0.01);
    assert_eq!(fs.version, 1);
    assert_eq!(fs.strategy, "full");
    assert!(fs.has_llm_savings());
}

#[test]
fn test_session_stats_summary() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/a.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    stats.record_compression("/test/b.ts", 2000, 500, "medium", true, "delta", Some(2000), "ir_compression");
    
    let summary = stats.summary();
    assert_eq!(summary.total_files, 2);
    assert_eq!(summary.total_raw_tokens, 3000); // 1000 + 2000
    assert_eq!(summary.total_compressed_tokens, 750); // 250 + 500
    assert_eq!(summary.full_compress_count, 1);
    assert_eq!(summary.delta_count, 1);
    // b.ts is delta: 500 delta tokens vs 2000 full compress = 75% efficiency
    assert!(summary.avg_delta_efficiency_pct.is_some());
    assert!((summary.avg_delta_efficiency_pct.unwrap() - 75.0).abs() < 0.1);
}

#[test]
fn test_empty_stats() {
    let stats = SessionStats::new();
    let summary = stats.summary();
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.total_raw_tokens, 0);
    assert_eq!(summary.total_compressed_tokens, 0);
    assert!(summary.avg_delta_efficiency_pct.is_none());
}

#[test]
fn test_render_dashboard_text() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    let text = render_dashboard_text(&stats);
    assert!(text.contains("Clean-CTX Dashboard"));
    assert!(text.contains("1,000"));
    assert!(text.contains("250"));
}

#[test]
fn test_render_dashboard_json() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    let json = render_dashboard_json(&stats);
    assert_eq!(json["session"]["total_files"], 1);
    assert_eq!(json["session"]["total_raw_tokens"], 1000);
    assert_eq!(json["files"].as_array().unwrap().len(), 1);
}

#[test]
fn test_multiple_records_same_file() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    stats.record_compression("/test/file.ts", 1000, 200, "low", false, "delta", Some(250), "ir_compression");
    
    let fs = stats.file_stats("/test/file.ts").unwrap();
    assert_eq!(fs.version, 2);
    assert_eq!(fs.delta_count, 1);
    assert!(!fs.has_llm_savings());
    // Delta efficiency: (250 - 200) / 250 = 20%
    assert!(fs.delta_efficiency_pct.is_some());
    assert!((fs.delta_efficiency_pct.unwrap() - 20.0).abs() < 0.1);
}

// ── Delta Efficiency Tests ─────────────────────────────────────────

#[test]
fn test_delta_efficiency_with_full_compressed_tokens() {
    let mut stats = SessionStats::new();
    // First a full compression
    stats.record_compression("/test/f.ts", 5000, 500, "low", false, "full", None, "ir_compression");
    // Then a delta: 120 tokens vs 500 full = 76% efficiency
    stats.record_compression("/test/f.ts", 5000, 120, "low", false, "delta", Some(500), "ir_compression");
    
    let fs = stats.file_stats("/test/f.ts").unwrap();
    assert_eq!(fs.strategy, "delta");
    assert!(!fs.has_llm_savings());
    assert!(fs.delta_efficiency_pct.is_some());
    let eff = fs.delta_efficiency_pct.unwrap();
    assert!((eff - 76.0).abs() < 0.1, "expected ~76%, got {}", eff);
    assert!(fs.full_compressed_tokens == Some(500));
}

#[test]
fn test_delta_efficiency_when_delta_larger_than_full() {
    // Edge case: delta output is larger than full compression
    // (should report None for efficiency since there's no savings)
    let mut stats = SessionStats::new();
    stats.record_compression("/test/f.ts", 5000, 500, "low", false, "full", None, "ir_compression");
    stats.record_compression("/test/f.ts", 5000, 600, "low", false, "delta", Some(500), "ir_compression");
    
    let fs = stats.file_stats("/test/f.ts").unwrap();
    assert_eq!(fs.strategy, "delta");
    assert!(fs.delta_efficiency_pct.is_none(), "no savings when delta > full");
}

#[test]
fn test_delta_efficiency_no_baseline() {
    // Delta without full_compressed_tokens (missing baseline)
    let mut stats = SessionStats::new();
    stats.record_compression("/test/f.ts", 5000, 200, "low", false, "delta", None, "ir_compression");
    
    let fs = stats.file_stats("/test/f.ts").unwrap();
    assert_eq!(fs.strategy, "delta");
    assert!(fs.delta_efficiency_pct.is_none());
}

// ── Render Tests for Delta Files ──────────────────────────────────

#[test]
fn test_render_dashboard_shows_na_for_delta_savings() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/a.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    stats.record_compression("/test/b.ts", 2000, 80, "medium", false, "delta", Some(500), "ir_compression");
    
    let text = render_dashboard_text(&stats);
    assert!(text.contains("75.0%"), "full compress file shows savings");
    assert!(text.contains("N/A"), "delta file shows N/A for savings");
    assert!(text.contains("Δ eff"), "delta file shows efficiency");
}

#[test]
fn test_render_dashboard_json_delta_fields() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/d.ts", 3000, 90, "high", false, "delta", Some(300), "ir_compression");
    
    let json = render_dashboard_json(&stats);
    let file = &json["files"][0];
    assert!(file["savings_pct"].is_null(), "delta file savings should be null");
    assert_eq!(file["delta_efficiency_pct"], 70.0, "efficiency: (300-90)/300=70%");
    assert_eq!(file["full_compressed_tokens"], 300);
    assert!(!json["session"]["avg_delta_efficiency_pct"].is_null(),
        "avg_delta_efficiency_pct should not be null for delta files");
}

// ── Phase 3 Merge Fix Tests ─────────────────────────────────────

#[test]
fn test_merge_in_memory_wins_for_existing_file() {
    // In-memory data should never be overwritten by DB-recovered data
    let mut in_memory = SessionStats::new();
    in_memory.record_compression("/test/shared.ts", 5000, 1000, "low", false, "full", None, "ir_compression");

    let mut db = SessionStats::new();
    db.record_compression("/test/shared.ts", 100, 30, "low", false, "delta", None, "ir_compression");

    in_memory.merge(&db);

    let shared = in_memory.file_stats("/test/shared.ts").unwrap();
    // In-memory values should be preserved
    assert_eq!(shared.raw_tokens, 5000, "in-memory raw_tokens should win");
    assert_eq!(shared.compressed_tokens, 1000, "in-memory compressed_tokens should win");
    assert_eq!(shared.strategy, "full", "in-memory strategy should win");
    assert_eq!(shared.fidelity, "low", "in-memory fidelity should win");
    // Version should be max of both versions
    assert_eq!(shared.version, 1);
}

#[test]
fn test_merge_db_only_file_imported() {
    // Files that only exist in DB should be imported
    let mut in_memory = SessionStats::new();
    in_memory.record_compression("/test/in_memory.ts", 100, 20, "low", false, "full", None, "ir_compression");

    let mut db = SessionStats::new();
    db.record_compression("/test/db_only.ts", 200, 40, "medium", true, "delta", None, "ir_compression");

    in_memory.merge(&db);

    assert!(in_memory.file_stats("/test/db_only.ts").is_some(), "DB-only file should appear");
    let db_only = in_memory.file_stats("/test/db_only.ts").unwrap();
    assert_eq!(db_only.raw_tokens, 200);
    assert_eq!(db_only.compressed_tokens, 40);
    assert_eq!(db_only.fidelity, "medium");

    // In-memory file should still be there
    assert!(in_memory.file_stats("/test/in_memory.ts").is_some());
}

#[test]
fn test_merge_delta_count_accumulates() {
    // Delta counts should merge even for overlapping files
    let mut in_memory = SessionStats::new();
    in_memory.record_compression("/test/file.ts", 1000, 200, "low", false, "full", None, "ir_compression");

    let mut db = SessionStats::new();
    db.record_compression("/test/file.ts", 0, 0, "low", false, "delta", None, "ir_compression");

    let in_memory_delta = in_memory.file_stats("/test/file.ts").unwrap().delta_count;
    let db_delta = db.file_stats("/test/file.ts").unwrap().delta_count;

    in_memory.merge(&db);

    let merged = in_memory.file_stats("/test/file.ts").unwrap();
    assert_eq!(merged.delta_count, in_memory_delta + db_delta,
        "delta counts should accumulate across sessions");
}

#[test]
fn test_merge_version_max() {
    let mut a = SessionStats::new();
    a.record_compression("/test/f.ts", 500, 100, "low", false, "full", None, "ir_compression");
    a.record_compression("/test/f.ts", 600, 120, "low", false, "delta", Some(100), "ir_compression");

    let mut b = SessionStats::new();
    b.record_compression("/test/f.ts", 1000, 200, "low", false, "full", None, "ir_compression"); // version=1 in b

    // After a: version = 2. After merge: max(2, 1) = 2
    a.merge(&b);
    let fs = a.file_stats("/test/f.ts").unwrap();
    assert_eq!(fs.version, 2, "version should be max of both");
}

// ── Strategy Label Tests ────────────────────────────────────────

#[test]
fn test_strategy_labels() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/f1.ts", 100, 20, "low", false, "diff", None, "ir_compression");
    stats.record_compression("/test/f2.rs", 200, 40, "medium", true, "restore", None, "ir_compression");
    stats.record_compression("/test/f3.cs", 300, 60, "high", false, "workspace", None, "ir_compression");
    stats.record_compression("/test/f4.js", 400, 80, "low", false, "workspace_gsym", None, "ir_compression");

    assert_eq!(stats.file_stats("/test/f1.ts").unwrap().strategy, "diff");
    assert_eq!(stats.file_stats("/test/f2.rs").unwrap().strategy, "restore");
    assert_eq!(stats.file_stats("/test/f3.cs").unwrap().strategy, "workspace");
    assert_eq!(stats.file_stats("/test/f4.js").unwrap().strategy, "workspace_gsym");

    let summary = stats.summary();
    assert_eq!(summary.total_files, 4);
}

// ── Multiple Files Dashboard Test ───────────────────────────────

#[test]
fn test_multiple_files_in_dashboard() {
    let mut stats = SessionStats::new();
    let files = vec![
        "/src/a.ts",
        "/src/b.rs",
        "/src/c.ts",
        "/src/d.cs",
    ];
    for (i, f) in files.iter().enumerate() {
        stats.record_compression(f, (i + 1) * 1000, (i + 1) * 200, "low", false, "full", None, "ir_compression");
    }

    let text = render_dashboard_text(&stats);
    for f in &files {
        let display = if f.len() > 38 {
            format!("...{}", &f[f.len()-37..])
        } else {
            f.to_string()
        };
        assert!(text.contains(&display[..std::cmp::min(display.len(), 10)]),
            "dashboard should contain each file path");
    }

    let json = render_dashboard_json(&stats);
    assert_eq!(json["files"].as_array().unwrap().len(), 4);
    assert_eq!(json["session"]["total_files"], 4);
    assert_eq!(json["session"]["total_raw_tokens"], 1000 + 2000 + 3000 + 4000);

    let summary = stats.summary();
    assert_eq!(summary.full_compress_count, 4);
    assert_eq!(summary.delta_count, 0);
}

// ── Large Token Count Formatting ───────────────────────────────

#[test]
fn test_large_token_counts() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/large.ts", 1_500_000, 250_000, "low", false, "full", None, "ir_compression");

    let text = render_dashboard_text(&stats);
    assert!(text.contains("1,500,000"));
    assert!(text.contains("250,000"));

    let json = render_dashboard_json(&stats);
    assert_eq!(json["session"]["total_raw_tokens"], 1_500_000);
}

// ── Phase 4: CacheMetrics Sync Tests ───────────────────────────

#[test]
fn test_sync_cache_metrics_populates_prompt_cache_domain() {
    let mut stats = SessionStats::new();
    // Pre-condition: no prompt_cache domain yet
    assert!(stats.domain_breakdown().get("prompt_cache").is_none());

    let metrics = crate::mcp::cache_hints::CacheMetrics {
        hits: 5,
        misses: 2,
        tokens_saved: 1200,
        breakpoints: std::collections::HashMap::new(),
    };
    stats.sync_cache_metrics(&metrics);

    let domain = stats.domain_breakdown().get("prompt_cache");
    assert!(domain.is_some(), "prompt_cache domain should exist after sync");
    let dm = domain.unwrap();
    assert_eq!(dm.cache_hits, Some(5));
    assert_eq!(dm.cache_misses, Some(2));
    assert_eq!(dm.total_raw_tokens, 1200);
    assert_eq!(dm.total_compressed_tokens, 0);
    assert_eq!(dm.savings_pct, 100.0);
}

#[test]
fn test_sync_cache_metrics_overwrites_existing() {
    let mut stats = SessionStats::new();
    let metrics1 = crate::mcp::cache_hints::CacheMetrics {
        hits: 3,
        misses: 1,
        tokens_saved: 500,
        breakpoints: std::collections::HashMap::new(),
    };
    stats.sync_cache_metrics(&metrics1);

    let metrics2 = crate::mcp::cache_hints::CacheMetrics {
        hits: 10,
        misses: 4,
        tokens_saved: 3500,
        breakpoints: std::collections::HashMap::new(),
    };
    stats.sync_cache_metrics(&metrics2);

    let domain = stats.domain_breakdown().get("prompt_cache").unwrap();
    assert_eq!(domain.cache_hits, Some(10));
    assert_eq!(domain.cache_misses, Some(4));
    assert_eq!(domain.total_raw_tokens, 3500);
}

#[test]
fn test_sync_cache_metrics_zero_activity() {
    let mut stats = SessionStats::new();
    let metrics = crate::mcp::cache_hints::CacheMetrics::default();
    stats.sync_cache_metrics(&metrics);

    let domain = stats.domain_breakdown().get("prompt_cache").unwrap();
    assert_eq!(domain.cache_hits, Some(0));
    assert_eq!(domain.cache_misses, Some(0));
    assert_eq!(domain.total_raw_tokens, 0);
    assert_eq!(domain.savings_pct, 0.0);
}

#[test]
fn test_dashboard_text_includes_prompt_cache_after_sync() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/a.ts", 1000, 250, "low", false, "full", None, "ir_compression");

    let metrics = crate::mcp::cache_hints::CacheMetrics {
        hits: 7,
        misses: 3,
        tokens_saved: 2100,
        breakpoints: std::collections::HashMap::new(),
    };
    stats.sync_cache_metrics(&metrics);

    let text = render_dashboard_text(&stats);
    assert!(text.contains("Prompt Cache"), "dashboard should show Prompt Cache domain");
    assert!(text.contains("2,100"), "dashboard should show tokens saved from cache");
}

#[test]
fn test_dashboard_json_includes_prompt_cache_after_sync() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/a.ts", 1000, 250, "low", false, "full", None, "ir_compression");

    let metrics = crate::mcp::cache_hints::CacheMetrics {
        hits: 4,
        misses: 1,
        tokens_saved: 800,
        breakpoints: std::collections::HashMap::new(),
    };
    stats.sync_cache_metrics(&metrics);

    let json = render_dashboard_json(&stats);
    let pc = &json["session"]["domain_breakdown"]["prompt_cache"];
    assert_eq!(pc["cache_hits"], 4);
    assert_eq!(pc["cache_misses"], 1);
    assert_eq!(pc["total_raw_tokens"], 800);
}
