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
fn test_record_cbm_proxy_accumulates() {
    // CBM pipe-level interceptions are DISTINCT events that must ACCUMULATE
    // in session totals (unlike per-file compression which overwrites).
    let mut stats = SessionStats::new();

    // 3 calls to the same CBM tool, each intercepting ~5000 raw tokens → ~1100
    stats.record_cbm_proxy("graph_search", 5000, 1100);
    stats.record_cbm_proxy("graph_search", 5000, 1100);
    stats.record_cbm_proxy("graph_search", 5000, 1100);

    // Session totals should be the SUM across all 3 calls (not just the last)
    let summary = stats.summary();
    assert_eq!(summary.total_raw_tokens, 15000, "should accumulate raw tokens across calls");
    assert_eq!(summary.total_compressed_tokens, 3300, "should accumulate compressed tokens across calls");
    assert_eq!(summary.full_compress_count, 3, "each CBM call is a full compression event");

    // Per-tool file entry should accumulate
    let fs = stats.file_stats("cbm://graph_search").expect("cbm://graph_search should be tracked");
    assert_eq!(fs.raw_tokens, 15000);
    assert_eq!(fs.compressed_tokens, 3300);
    assert_eq!(fs.version, 3, "version should increment per call");
    // 78% savings: (15000-3300)/15000
    assert!((fs.savings_pct - 78.0).abs() < 0.1);

    // Domain breakdown should accumulate. `file_count` counts UNIQUE tools
    // (3 calls to the same tool = 1 unique tool), consistent with
    // `ir_compression`'s unique-file semantics.
    let domain = stats.domain_breakdown().get("cbm_filter").expect("cbm_filter domain");
    assert_eq!(domain.total_raw_tokens, 15000);
    assert_eq!(domain.total_compressed_tokens, 3300);
    assert_eq!(domain.file_count, 1, "3 calls to same tool = 1 unique tool");
}

#[test]
fn test_record_cbm_proxy_distinct_tools() {
    // Different CBM tools should create separate per-tool entries but
    // all contribute to the same cbm_filter domain.
    let mut stats = SessionStats::new();
    stats.record_cbm_proxy("graph_search", 5000, 1100);
    stats.record_cbm_proxy("graph_trace", 3000, 800);

    let summary = stats.summary();
    assert_eq!(summary.total_files, 2, "two distinct CBM tools = two file entries");
    assert_eq!(summary.total_raw_tokens, 8000);
    assert_eq!(summary.total_compressed_tokens, 1900);

    // Both tools in the cbm_filter domain
    let domain = stats.domain_breakdown().get("cbm_filter").expect("cbm_filter domain");
    assert_eq!(domain.total_raw_tokens, 8000);
    assert_eq!(domain.total_compressed_tokens, 1900);
    assert_eq!(domain.file_count, 2);
}

// ── Regression: CBM proxy events survive merge (persistence restore) ──
//
// REGRESSION GUARD: `merge()` recalculates `full_compress_count` from unique
// file entries. Each CBM tool creates ONE file entry regardless of call count,
// so without the dedicated `cbm_proxy_events` counter, a persistence restore
// would understate CBM activity (10 calls to graph_search → count=1). This
// test locks in the fix.

#[test]
fn test_merge_preserves_cbm_proxy_event_count() {
    // In-memory session: 3 CBM proxy calls to graph_search
    let mut in_memory = SessionStats::new();
    in_memory.record_cbm_proxy("graph_search", 5000, 1100);
    in_memory.record_cbm_proxy("graph_search", 5000, 1100);
    in_memory.record_cbm_proxy("graph_search", 5000, 1100);

    // DB-recovered session: 2 CBM proxy calls to graph_trace
    let mut db = SessionStats::new();
    db.record_cbm_proxy("graph_trace", 3000, 800);
    db.record_cbm_proxy("graph_trace", 3000, 800);

    // Merge DB into in-memory (simulates persistence restore)
    in_memory.merge(&db);

    let summary = in_memory.summary();
    // 3 + 2 = 5 CBM proxy events must survive the merge
    assert_eq!(
        summary.full_compress_count, 5,
        "CBM proxy event count must survive merge (3 graph_search + 2 graph_trace)"
    );

    // Token totals accumulate across both sessions
    assert_eq!(summary.total_raw_tokens, 5000*3 + 3000*2, "raw tokens accumulate across merge");
    assert_eq!(summary.total_compressed_tokens, 1100*3 + 800*2, "compressed tokens accumulate across merge");

    // Both tools present as file entries
    assert!(in_memory.file_stats("cbm://graph_search").is_some());
    assert!(in_memory.file_stats("cbm://graph_trace").is_some());
}

// ── Regression: cbm_filter domain file_count = unique tools, not calls ──
//
// REGRESSION GUARD: `record_cbm_proxy` must count UNIQUE tools in the
// `cbm_filter` domain `file_count` (consistent with `ir_compression`'s
// unique-file semantics), not the number of calls. Without the `is_new_tool`
// check, 3 calls to the same tool would inflate file_count to 3.

#[test]
fn test_cbm_filter_domain_file_count_is_unique_tools() {
    let mut stats = SessionStats::new();
    // 3 calls to graph_search, 1 call to graph_trace
    stats.record_cbm_proxy("graph_search", 5000, 1100);
    stats.record_cbm_proxy("graph_search", 5000, 1100);
    stats.record_cbm_proxy("graph_search", 5000, 1100);
    stats.record_cbm_proxy("graph_trace", 3000, 800);

    let domain = stats.domain_breakdown().get("cbm_filter").expect("cbm_filter domain");
    assert_eq!(
        domain.file_count, 2,
        "file_count should count unique tools (graph_search + graph_trace), not 4 calls"
    );
    // But token totals reflect ALL calls
    assert_eq!(domain.total_raw_tokens, 5000*3 + 3000);
    assert_eq!(domain.total_compressed_tokens, 1100*3 + 800);
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
fn test_render_dashboard_shows_angular_template_domain() {
    // ANGULAR_HTML_COMPRESSION_PLAN Phase 3: `.component.html` files
    // record to the `angular_template` domain. The dashboard must show
    // this domain in the per-domain breakdown.
    let mut stats = SessionStats::new();
    stats.record_compression(
        "/test/dashboard.component.html", 917, 300, "high", true, "full", None, "angular_template",
    );
    let text = render_dashboard_text(&stats);
    assert!(
        text.contains("Angular Templates"),
        "dashboard should show Angular Templates domain, got:\n{text}"
    );
    assert!(text.contains("917"), "dashboard should show raw token count");
    assert!(text.contains("300"), "dashboard should show compressed token count");

    // JSON dashboard should also carry the domain breakdown.
    let json = render_dashboard_json(&stats);
    let at = &json["session"]["domain_breakdown"]["angular_template"];
    assert_eq!(at["total_raw_tokens"], 917);
    assert_eq!(at["total_compressed_tokens"], 300);
    assert_eq!(at["file_count"], 1);
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
    // R-02 FAANG: delta after a full PRESERVES the full's LLM savings.
    assert!(fs.has_llm_savings(), "delta after full should preserve LLM savings");
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
    // R-02 FAANG: delta after a full PRESERVES the full's LLM savings.
    assert!(fs.has_llm_savings(), "delta after full should preserve LLM savings");
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
fn test_sync_cache_metrics_accumulates_existing() {
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
    // ACCUMULATES hits/misses: 3+10=13, 1+4=5
    assert_eq!(domain.cache_hits, Some(13), "hits should accumulate across syncs");
    assert_eq!(domain.cache_misses, Some(5), "misses should accumulate across syncs");
    // ACCUMULATES: 500 + 3500 = 4000. Real proxy cache-read tokens
    // (recorded via `record_cache_hit`) are preserved alongside MCP-side
    // dedup savings — they must NOT be overwritten.
    assert_eq!(domain.total_raw_tokens, 4000);
}

#[test]
fn test_sync_cache_metrics_preserves_proxy_hits() {
    // Real proxy cache hits (recorded via `record_cache_hit`) must be
    // preserved when MCP-side metrics are synced.
    let mut stats = SessionStats::new();
    // Proxy records 1 real hit with 5000 tokens saved
    stats.record_cache_hit(5000);

    // MCP-side metrics sync with 3 hits, 2 misses, 1200 tokens
    let metrics = crate::mcp::cache_hints::CacheMetrics {
        hits: 3,
        misses: 2,
        tokens_saved: 1200,
        breakpoints: std::collections::HashMap::new(),
    };
    stats.sync_cache_metrics(&metrics);

    let domain = stats.domain_breakdown().get("prompt_cache").unwrap();
    // Proxy hit (1) + MCP hits (3) = 4
    assert_eq!(domain.cache_hits, Some(4), "proxy hits must be preserved");
    assert_eq!(domain.cache_misses, Some(2));
    // Proxy tokens (5000) + MCP tokens (1200) = 6200
    assert_eq!(domain.total_raw_tokens, 6200, "proxy tokens must be preserved");
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

// ── R-02 FAANG: Delta preserves prior full-compression savings ──
//
// When a delta is recorded for a file that previously had a FULL
// compression, the full compression's token counts and LLM token
// savings must be PRESERVED. Delta ops are local CPU events — they
// do not change the file's LLM-visible token profile. Previously the
// delta recording erased the full compression's savings (dashboard
// showed N/A).

#[test]
fn test_delta_preserves_full_compression_savings() {
    let mut stats = SessionStats::new();

    // 1. Full compression: 1000 raw → 250 compressed (75% savings)
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    let fs = stats.file_stats("/test/file.ts").unwrap();
    assert_eq!(fs.raw_tokens, 1000);
    assert_eq!(fs.compressed_tokens, 250);
    assert!((fs.savings_pct - 75.0).abs() < 0.01);
    assert!(fs.has_llm_savings());

    // 2. Delta arrives (handler passes 0/0 for raw/comp, prev full compressed = 250)
    stats.record_compression("/test/file.ts", 0, 0, "low", false, "delta", Some(250), "ir_compression");

    // The full compression's savings must be PRESERVED
    let fs = stats.file_stats("/test/file.ts").unwrap();
    assert_eq!(fs.raw_tokens, 1000, "raw_tokens should be preserved from full compression");
    assert_eq!(fs.compressed_tokens, 250, "compressed_tokens should be preserved from full compression");
    assert!((fs.savings_pct - 75.0).abs() < 0.01, "savings_pct should be preserved");
    assert!(fs.has_llm_savings(), "delta file with preserved savings should still show LLM savings");
    assert_eq!(fs.strategy, "delta");
    assert_eq!(fs.delta_count, 1);

    // Session totals should still reflect the full compression's tokens
    let summary = stats.summary();
    assert_eq!(summary.total_raw_tokens, 1000, "session raw should reflect full compression");
    assert_eq!(summary.total_compressed_tokens, 250, "session compressed should reflect full compression");
    assert!((summary.total_savings_pct - 75.0).abs() < 0.01, "session savings should be preserved");
    // full_compress_count stays 1 (the file still counts as a full compression)
    assert_eq!(summary.full_compress_count, 1);
    assert_eq!(summary.delta_count, 1);
}

#[test]
fn test_delta_preserves_savings_in_dashboard() {
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 1000, 250, "low", false, "full", None, "ir_compression");
    stats.record_compression("/test/file.ts", 0, 0, "low", false, "delta", Some(250), "ir_compression");

    // Dashboard should show the real savings %, not N/A
    let text = render_dashboard_text(&stats);
    assert!(text.contains("75.0%"), "dashboard should show 75.0% savings, got: {}", text);
    assert!(!text.contains("N/A"), "dashboard should NOT show N/A for a delta file with preserved savings");

    // JSON dashboard should show numeric savings_pct, not null
    let json = render_dashboard_json(&stats);
    let file = &json["files"][0];
    assert!(file["savings_pct"].is_number(), "JSON savings_pct should be numeric, got: {}", file["savings_pct"]);
    assert_eq!(file["savings_pct"], 75.0);
}

#[test]
fn test_delta_without_prior_full_still_shows_na() {
    // A delta with NO prior full compression has no LLM savings to preserve.
    let mut stats = SessionStats::new();
    stats.record_compression("/test/file.ts", 0, 0, "low", false, "delta", None, "ir_compression");

    let fs = stats.file_stats("/test/file.ts").unwrap();
    assert_eq!(fs.raw_tokens, 0);
    assert_eq!(fs.compressed_tokens, 0);
    assert_eq!(fs.savings_pct, 0.0);
    assert!(!fs.has_llm_savings(), "delta with no prior full should NOT have LLM savings");

    let text = render_dashboard_text(&stats);
    assert!(text.contains("N/A"), "delta with no prior full should show N/A");
}
