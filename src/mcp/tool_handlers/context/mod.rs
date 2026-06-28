// src/mcp/tool_handlers/context/mod.rs
//
// Context history handler — queries in-memory context store and
// session-level stats to show per-file versioning and cache metrics.
//
// v0.3.0: Separated from core handlers for Single Responsibility.

use serde_json::Value;
use crate::mcp::McpState;
use crate::mcp::context_store::ContextStore;
use crate::protocol::send_response;

/// Handle `context_history` — shows per-file versioning, delta history,
/// and cache efficiency metrics. With no file path, shows all tracked files.
pub(crate) fn handle_context_history(
    id: &Value,
    params: &Value,
    state: &McpState,
) {
    let file_path = params["arguments"]["filePath"].as_str();

    if let Some(fp) = file_path {
        let path_alias = state.get_or_create_alias(fp.to_string());
        let version = state.text_delta_lock().file_version(&path_alias);
        let has_ir = state.file_version(&path_alias).is_some();
        let store_meta = state.context_store.load_latest(fp).ok().flatten();

        let mut lines = Vec::new();
        lines.push(format!("File: {}", fp));
        lines.push(format!("  Text Delta Versions: {}", version));
        lines.push(format!("  IR Baseline: {}", if has_ir { "yes" } else { "no" }));
        lines.push(format!("  Context Store: {}", if store_meta.is_some() { "yes" } else { "no" }));
        if let Some(meta) = store_meta {
            lines.push(format!("  Context Version: {}", meta.version));
        }

        let metrics = state.cache_metrics_lock();
        let total = metrics.hits + metrics.misses;
        lines.push(format!("  Cache Hit Rate: {}/{} ({}%)",
            metrics.hits,
            total,
            if total > 0 {
                (metrics.hits as f64 / total as f64 * 100.0) as usize
            } else {
                0
            },
        ));
        lines.push(format!("  Cache Tokens Saved: {}", metrics.tokens_saved));
        drop(metrics);

        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": lines.join("\n") }]
            }
        }));
    } else {
        let stats = state.session_stats_lock();
        let file_stats = stats.all_file_stats();
        if file_stats.is_empty() {
            drop(stats);
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{ "type": "text", "text": "No tracked files yet. Call `provide_code_context` first." }]
                }
            }));
            return;
        }

        let mut output = String::from("Tracked Files:\n");
        for (path, fstats) in file_stats {
            output.push_str(&format!(
                "  {} — v{}, {} deltas, {:.1}% savings\n",
                path, fstats.version, fstats.delta_count, fstats.savings_pct
            ));
        }
        drop(stats);

        let metrics = state.cache_metrics_lock();
        let hit_rate = if metrics.hits + metrics.misses > 0 {
            metrics.hits as f64 / (metrics.hits + metrics.misses) as f64
        } else {
            0.0
        };
        output.push_str(&format!(
            "── Cache ──\n  Hits: {} | Misses: {} | Hit Rate: {:.0}% | Tokens Saved: {}\n",
            metrics.hits,
            metrics.misses,
            hit_rate * 100.0,
            metrics.tokens_saved,
        ));
        drop(metrics);

        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{ "type": "text", "text": output }]
            }
        }));
    }
}