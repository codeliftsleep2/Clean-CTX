// src/mcp/tool_handlers/stats/mod.rs
//
// Dashboard/stats handler — renders session-level compression stats,
// cache efficiency, and persistence status as text or JSON.
//
// v0.3.0: Separated from core handlers for Single Responsibility.

use serde_json::Value;
use crate::mcp::McpState;
use crate::mcp::cache_hints::{render_cache_text, render_cache_json};
use crate::protocol::send_response;

/// Handle `context_stats` — renders the Clean-CTX dashboard.
/// Shows per-file or session-level token savings, compression stats,
/// and cache/persistence status. Supports `text` (default) and `json` formats.
pub(crate) fn handle_context_stats(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    state.flush_persistence();

    // Rebuild stats from DB if persistence is enabled
    let db_stats = {
        let guard = state.persistence_store_lock();
        guard.as_ref().and_then(|store| {
            store.sqlite().and_then(|sql_guard| sql_guard.rebuild_stats().ok())
        })
    };

    let mut merged = if let Some(ref db) = db_stats {
        let stats = state.session_stats_lock();
        let mut m = stats.clone();
        m.merge(db);
        m
    } else {
        let stats = state.session_stats_lock();
        stats.clone()
    };

    // Fetch proxy stats if available and apply to BOTH live stats and merged clone
    if state.proxy_port > 0 {
        let proxy_stats = crate::mcp::proxy_stats::fetch_proxy_stats(state.proxy_port);
        if let Some(ref ps) = proxy_stats {
            let mut stats_guard = state.session_stats_lock();
            crate::mcp::proxy_stats::record_proxy_filter_stats(&mut stats_guard, ps);
            // Also apply to the merged clone so the current dashboard call shows them
            crate::mcp::proxy_stats::record_proxy_filter_stats(&mut merged, ps);
        }
    }

    {
        let metrics = state.cache_metrics_lock();
        merged.sync_cache_metrics(&metrics);
    }

    let file_path = params["arguments"]["filePath"].as_str();
    let format = params["arguments"]["format"].as_str().unwrap_or("text");

    if let Some(fp) = file_path {
        let stats = merged.file_stats(fp);
        match stats {
            Some(fs) => {
                let mut text = format!(
                    "File: {}\n  Raw: {} → Compressed: {} ({:.1}% savings)\n  Version: {}, Deltas: {}, Fidelity: {}\n  Angular: {}, Strategy: {}",
                    fs.file_path,
                    fs.raw_tokens,
                    fs.compressed_tokens,
                    fs.savings_pct,
                    fs.version,
                    fs.delta_count,
                    fs.fidelity,
                    fs.is_angular,
                    fs.strategy,
                );
                if db_stats.is_some() {
                    text.push_str("\n  Persistence: enabled");
                } else {
                    text.push_str("\n  Persistence: disabled");
                }
                if format == "json" {
                    let mut json = serde_json::json!(fs);
                    json["persistence"] = serde_json::json!({"enabled": db_stats.is_some()});
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&json).unwrap_or_default() }]
                        }
                    }));
                } else {
                    send_response(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [{ "type": "text", "text": text }]
                        }
                    }));
                }
            }
            None => {
                let mut text = format!("No stats for file: {}", fp);
                if db_stats.is_some() {
                    text.push_str("\n  Persistence: enabled (no data for this file)");
                } else {
                    text.push_str("\n  Persistence: disabled");
                }
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": text }]
                    }
                }));
            }
        }
    } else {
        let mut text = crate::mcp::session_stats::render_dashboard_text(&merged);

        let active_tokenizer = state.config.tokenizer.to_string();
        text.push_str(&format!("── Tokenizer ──\n  Active: {} (config: {})\n", active_tokenizer, active_tokenizer));

        if let Some(ref db_summary) = db_stats.as_ref().map(|db| db.summary()) {
            text.push_str(&format!(
                "── Persistence (SQLite) ──\n  Status: enabled\n  DB Files: {}\n  DB Compressions: {}\n  DB Deltas: {}\n",
                db_summary.total_files,
                db_summary.full_compress_count,
                db_summary.delta_count,
            ));
        } else {
            text.push_str("── Persistence (SQLite) ──\n  Status: disabled\n");
        }

        let metrics = state.cache_metrics_lock();
        if let Some(cache_text) = render_cache_text(&metrics, state.config.cache.enabled) {
            text.push_str(&cache_text);
        }

        if format == "json" {
            let mut json = crate::mcp::session_stats::render_dashboard_json(&merged);
            json["tokenizer"] = serde_json::json!(state.config.tokenizer.to_string());
            json["persistence"] = serde_json::json!({"enabled": db_stats.is_some()});
            json["cache"] = render_cache_json(&metrics, state.config.cache.enabled);
            drop(metrics);
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": serde_json::to_string_pretty(&json).unwrap_or_default() }]
                }
            }));
        } else {
            drop(metrics);
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": text }]
                }
            }));
        }
    }
}