// src/mcp/cache_hints.rs
//
// Prompt cache optimization module — injects `_meta.cache_hints` into
// JSON-RPC responses so Anthropic API clients can set `cache_control`
// breakpoints on stable content regions.
//
// Four breakpoint regions:
//   "system_prompt"     — opcode/marker vocabulary (stable across sessions)
//   "tools"             — MCP tool definitions (stable across sessions)
//   "baseline"          — persisted workspace baselines (stable until file hash changes)
//   "tail"              — rolling dynamic content (never cached across turns)
//
// Each injection is deduplicated via `state.emitted_breakpoints` to
// avoid paying the 2.0× write multiplier on already-emitted breakpoints.
//
// Cache metrics are accumulated in `state.cache_metrics` and surfaced
// in the `context_stats` dashboard.
//
// R-19 + Phase 3: `inject_cache_breakpoints` now accepts an optional
// `&dyn Tokenizer` for accurate token savings estimates. On cache hits,
// the real response JSON is tokenized instead of using the rough chars/4
// estimate, giving precise token savings in the dashboard.

use std::collections::HashMap;
use serde::Serialize;
use sha2::Digest;

/// Tracks cache efficiency metrics for the session.
///
/// Surfaced in the `context_stats` dashboard (both text and JSON).
#[derive(Debug, Clone, Serialize, Default)]
pub struct CacheMetrics {
    /// Number of cache hits (deduped breakpoint already emitted).
    pub hits: usize,
    /// Number of cache misses (first-time breakpoint emission).
    pub misses: usize,
    /// Estimated tokens saved by cache hits (chars/4 estimate, or real tokenizer).
    pub tokens_saved: usize,
    /// Per-region status: "hit", "miss", "ephemeral", or "disabled".
    pub breakpoints: HashMap<String, String>,
}

/// Cache breakpoint metadata injected into JSON-RPC `_meta` fields.
#[derive(Debug, Clone, Serialize)]
pub struct CacheHints {
    /// List of active breakpoints for this response.
    pub breakpoints: Vec<CacheBreakpoint>,
}

/// A single cache breakpoint hint for the client.
#[derive(Debug, Clone, Serialize)]
pub struct CacheBreakpoint {
    /// Logical region identifier:
    /// "system_prompt" | "tools" | "baseline" | "workspace_baseline" | "tail"
    pub region: String,
    /// Suggested TTL string (e.g., "1h", "5m").
    pub ttl: String,
    /// Breaker key — invalidates cached content when this value changes.
    /// For baselines: SHA-256 of the compressed output (prefix "bl_").
    /// For stable regions: semantic version string (e.g., "v1").
    /// For tail: always "rolling".
    pub breaker: String,
}

/// Compute a baseline breaker key from compressed content.
///
/// Returns a short stable hash string prefixed with "bl_" that changes
/// when the compressed output changes. Used as the cache breaker for
/// persisted workspace baselines.
pub fn compute_baseline_breaker(compressed_text: &str) -> String {
    let hash = sha2::Sha256::digest(compressed_text.as_bytes());
    format!("bl_{:x}", hash)
}

/// Compute a workspace composite breaker from a set of file hashes.
///
/// Combines all file hashes into a single composite hash so one breaker
/// covers the entire baseline block rather than one per file.
///
/// Used by the `compress_workspace` tool handler to inject a single
/// baseline cache breakpoint for the entire workspace manifest.
///
/// The `compress_workspace` handler currently hashes the full manifest
/// string directly rather than collecting per-file hashes. This function
/// is retained for potential per-file incremental workspace caching.
pub fn compute_workspace_breaker(file_hashes: &[String]) -> String {
    let combined = file_hashes.join(",");
    let hash = sha2::Sha256::digest(combined.as_bytes());
    format!("ws_{:x}", hash)
}

/// Inject cache breakpoints into a JSON-RPC response's `_meta` field.
///
/// **IMPORTANT**: The MCP spec requires `_meta` to live *inside* the `result`
/// object, NOT at the top level of the JSON-RPC response. This function
/// auto-detects whether it was called with a full JSON-RPC envelope (has
/// `"jsonrpc"` field) or with a bare `result` sub-object, and routes the
/// `_meta` field accordingly.
///
/// In practice:
/// - `handlers.rs` / `tools.rs`: pass the full response → `_meta` goes into
///   `response["result"]["_meta"]`.
/// - `tool_handlers.rs`: pass `response.get_mut("result")` directly →
///   `_meta` goes into `response["_meta"]` (which IS the result).
///
/// # Rules
///
/// * SKIPS injection when `state.config.cache.enabled == false`.
/// * SKIPS emission if the same `{region}::{breaker}` combo has already
///   been emitted this session (dedup via `state.emitted_breakpoints`).
/// * On dedup hit: increments `state.cache_metrics.hits` and returns an
///   estimated `tokens_saved` value. When `tokenizer` is provided, the
///   response JSON is tokenized for an accurate savings estimate;
///   otherwise falls back to the rough `breaker.len() / 4` heuristic.
/// * On first emission: increments `state.cache_metrics.misses` and
///   records the breakpoint in `_meta.cache_hints`.
///
/// # Returns
///
/// An estimated token savings count if this was a cache hit, or 0.
pub fn inject_cache_breakpoints(
    response: &mut serde_json::Value,
    state: &mut crate::mcp::McpState,
    region: &str,
    ttl: &str,
    breaker: &str,
    tokenizer: Option<&dyn crate::tokenizer::Tokenizer>,
) -> usize {
    if !state.config.cache.enabled {
        return 0;
    }

    // Dedup: skip if this exact region+breaker combo was already emitted
    let dedup_key = format!("{}::{}", region, breaker);
    if state.emitted_breakpoints.contains(&dedup_key) {
        // Cache hit — count it
        state.cache_metrics.hits += 1;
        state
            .cache_metrics
            .breakpoints
            .insert(region.to_string(), "hit".to_string());

        // Token savings estimate when a cache hit occurs.
        // We tokenize just the breakpoint metadata (not the full response)
        // since only the breakpoint portion was deduplicated.
        let saved = if let Some(tok) = tokenizer {
            let hint_json = serde_json::to_string(&CacheHints {
                breakpoints: vec![CacheBreakpoint {
                    region: region.to_string(),
                    ttl: ttl.to_string(),
                    breaker: breaker.to_string(),
                }],
            }).unwrap_or_default();
            tok.count_tokens(&hint_json)
        } else {
            // Fallback: rough chars/4 estimate of the breakpoint metadata
            let hint_len = region.len() + ttl.len() + breaker.len() + 16; // overhead for JSON structure
            hint_len / 4
        };
        state.cache_metrics.tokens_saved += saved;
        return saved;
    }

    // Mark as emitted
    state.emitted_breakpoints.insert(dedup_key);

    // Build the cache hints payload
    let hints = CacheHints {
        breakpoints: vec![CacheBreakpoint {
            region: region.to_string(),
            ttl: ttl.to_string(),
            breaker: breaker.to_string(),
        }],
    };

    // Route to the correct sub-object:
    // - If `response` is a full JSON-RPC envelope (has "jsonrpc"), inject into
    //   `response["result"]["_meta"]["cache_hints"]`.
    // - If `response` IS already the result sub-object, inject into
    //   `response["_meta"]["cache_hints"]`.
    let target = if response.get("jsonrpc").is_some() {
        // Full JSON-RPC response — _meta MUST live inside `result`
        if !response.get("result").map_or(false, |v| v.is_object()) {
            // No result object yet — create one (shouldn't happen in practice)
            response["result"] = serde_json::json!({});
        }
        response.get_mut("result").unwrap()
    } else {
        // Bare result sub-object — operate directly
        response
    };

    // Inject into target._meta.cache_hints
    if let Some(meta) = target.get_mut("_meta") {
        if let Some(obj) = meta.as_object_mut() {
            obj.insert(
                "cache_hints".to_string(),
                serde_json::to_value(&hints).unwrap_or_default(),
            );
        }
    } else {
        // Create _meta if not present
        target["_meta"] = serde_json::json!({ "cache_hints": hints });
    }

    // Record as miss (first emission)
    state.cache_metrics.misses += 1;
    state
        .cache_metrics
        .breakpoints
        .insert(region.to_string(), "miss".to_string());
    0
}

/// Update cache metrics for the rolling tail (always ephemeral).
pub fn mark_tail_ephemeral(state: &mut crate::mcp::McpState) {
    state
        .cache_metrics
        .breakpoints
        .insert("tail".to_string(), "ephemeral".to_string());
}

/// Render the cache metrics section for the text dashboard.
///
/// Returns `None` if cache is disabled or has zero activity
/// (no breakpoints ever emitted). This avoids showing a misleading
/// "0% hit rate" section when the cache isn't actually in use.
///
/// Returns `Some(text)` with the cache section when cache has real activity.
pub fn render_cache_text(metrics: &CacheMetrics, enabled: bool) -> Option<String> {
    if !enabled && metrics.hits == 0 && metrics.misses == 0 {
        return None; // disabled and never active — don't show
    }
    let status = if enabled { "enabled" } else { "disabled" };
    let total = metrics.hits + metrics.misses;
    if total == 0 {
        // Enabled but no activity yet — show status only
        return Some(format!(
            "── Prompt Cache (LLM) ──\n  Status: {} (no activity yet)\n",
            status,
        ));
    }
    let hit_rate = metrics.hits as f64 / total as f64;
    Some(format!(
        "── Prompt Cache (LLM token savings) ──\n  Status: {}\n  Hits: {} | Misses: {} | Hit Rate: {:.0}%\n  LLM Tokens Saved: {}\n",
        status,
        metrics.hits,
        metrics.misses,
        hit_rate * 100.0,
        metrics.tokens_saved,
    ))
}

/// Render the cache metrics as a JSON value for the structured dashboard.
///
/// Returns the enabled/disabled status regardless of activity level.
pub fn render_cache_json(metrics: &CacheMetrics, enabled: bool) -> serde_json::Value {
    let total = metrics.hits + metrics.misses;
    let hit_rate = if total > 0 {
        metrics.hits as f64 / total as f64
    } else {
        0.0
    };
    serde_json::json!({
        "enabled": enabled,
        "active": total > 0,
        "hits": metrics.hits,
        "misses": metrics.misses,
        "hit_rate": (hit_rate * 100.0).round() / 100.0,
        "llm_tokens_saved": metrics.tokens_saved,
        "breakpoints": metrics.breakpoints,
    })
}

/// Generate the opcode/marker vocabulary text for the `clean-ctx-vocabulary`
/// prompt resource. Reads from the canonical opcode definitions to avoid
/// hardcoding a duplicate list.
///
/// This function is used by the `prompts/get` MCP handler.
pub fn generate_vocabulary_text() -> String {
    let lines = vec![
        "Clean-CTX Opcode/Marker Vocabulary",
        "===================================",
        "",
        "$c   → class",
        "$i   → interface",
        "⊕    → decorator/annotation",
        "⊕guard → conditional branch (guard clause)",
        "Φ    → Angular composite operation",
        "Φcmp → Angular component",
        "Φdir → Angular directive",
        "Φpipe → Angular pipe",
        "Φserv → Angular service",
        "Φmod → Angular module",
        "Φguard → Angular route guard",
        "Φres → Angular resolver",
        "⊕Input → @Input() decorator",
        "⊕Output → @Output() decorator",
        "⊕HostListener → @HostListener() decorator",
        "⊕Inject → @Inject() decorator",
        "⊕ViewChild → @ViewChild() decorator",
        "⊕ViewChildren → @ViewChildren() decorator",
        "[ → array (list literal/type)",
        "~→ function/method",
        "-> return type",
        "?→ nullable/optional",
        "!→ non-null assertion",
        "=→ default value/equality",
        "γ→ type parameter",
        "→ type annotation/mapping",
        "∞→ literal type/value",
        "_→ unused/ignored",
        "",
        "Use this vocabulary to read compressed code context output",
        "from the compress_code_context and provide_code_context tools.",
    ];
    lines.join("\n")
}

#[cfg(test)]
#[path = "../tests/mcp/cache_hints.rs"]
mod tests;