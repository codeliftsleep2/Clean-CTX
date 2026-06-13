# Prompt Cache Optimization — Implementation Plan

## Overview

Inject Anthropic prompt cache breakpoints into Clean-CTX MCP responses so the LLM never re-pays the 1.25× write multiplier on stable content (opcode vocabulary, tool definitions, persisted baselines). Four breakpoint regions, each with configurable TTL, deterministic breaker keys, and an `emitted` dedup tracker.

**Savings target:** Cache hits on the ~24k-token tools block + system prompt vocabulary + stable baselines, on top of the existing 96.6% marker-notation compression.

---

## Phase 1: Configuration Layer

### File: `src/config.rs`

Add `CacheConfig` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Master switch for prompt cache optimization annotations.
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// TTL for the system prompt / opcode vocabulary.
    /// Stable across every session. Default: "1h".
    #[serde(default = "default_stable_ttl")]
    pub system_prompt_ttl: String,

    /// TTL for the MCP tool definitions block (~24k tokens).
    /// Stable across every session. Default: "1h".
    #[serde(default = "default_stable_ttl")]
    pub tools_ttl: String,

    /// TTL for persisted workspace baselines (unchanged files).
    /// Stable until file hash changes. Default: "1h".
    #[serde(default = "default_stable_ttl")]
    pub baseline_ttl: String,

    /// TTL for the rolling tail (dynamic content that changes each turn).
    /// Matches Anthropic's 5-minute default fallback. Default: "5m".
    #[serde(default = "default_tail_ttl")]
    pub tail_ttl: String,

    /// Semantic version of the opcode vocabulary.
    /// Bumped only when opcodes/markers change. Default: "v1".
    #[serde(default = "default_vocab_version")]
    pub vocab_version: String,

    /// Semantic version of the tool definitions.
    /// Bumped only when tools are added/removed. Default: "v1".
    #[serde(default = "default_tool_version")]
    pub tool_defs_version: String,
}
```

Helper functions:
```rust
fn default_cache_enabled() -> bool { true }
fn default_stable_ttl() -> String { "1h".to_string() }
fn default_tail_ttl() -> String { "5m".to_string() }
fn default_vocab_version() -> String { "v1".to_string() }
fn default_tool_version() -> String { "v1".to_string() }
```

Add `cache: CacheConfig` field to `CleanCtxConfig`:
```rust
#[serde(default)]
pub cache: CacheConfig,
```

---

## Phase 2: Cache Hints Module

### New file: `src/mcp/cache_hints.rs`

### Types

```rust
use std::collections::HashMap;
use serde::Serialize;

/// Tracks cache efficiency metrics for the session.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CacheMetrics {
    /// Number of cache hits (baseline breaker matched).
    pub hits: usize,
    /// Number of cache misses (breaker changed).
    pub misses: usize,
    /// Estimated tokens saved by cache hits.
    pub tokens_saved: usize,
    /// Per-region status: "hit", "miss", "ephemeral", "disabled"
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
    /// For baselines: SHA-256 of the compressed output.
    /// For stable regions: semantic version string (e.g., "v1").
    pub breaker: String,
}
```

### Functions

```rust
/// Compute a baseline breaker key from compressed content.
pub fn compute_baseline_breaker(compressed_text: &str) -> String {
    let hash = sha2::Sha256::digest(compressed_text.as_bytes());
    format!("bl_{:x}", hash)
}

/// Compute a workspace composite breaker from a set of file hashes.
pub fn compute_workspace_breaker(file_hashes: &[String]) -> String {
    let mut combined = file_hashes.join(",");
    let hash = sha2::Sha256::digest(combined.as_bytes());
    format!("ws_{:x}", hash)
}

/// Inject cache breakpoints into a JSON-RPC response's `_meta` field.
///
/// SKIPS injection when `state.config.cache.enabled == false`.
/// SKIPS emission if the same region has already been emitted this session
///   (tracks via `state.emitted_breakpoints`).
/// SKIPS for excluded files, errors, or contexts that don't match the
///   "deterministically reproducible" rule.
///
/// Returns `tokens_saved` estimate if this was a cache hit, or 0.
pub fn inject_cache_breakpoints(
    response: &mut serde_json::Value,
    state: &mut crate::mcp::McpState,
    region: &str,
    ttl: &str,
    breaker: &str,
) -> usize {
    if !state.config.cache.enabled {
        return 0;
    }

    // Dedup: skip if this exact region+breaker combo was already emitted
    let dedup_key = format!("{}::{}", region, breaker);
    if state.emitted_breakpoints.contains(&dedup_key) {
        // Cache hit — count it
        state.cache_metrics.hits += 1;
        state.cache_metrics.breakpoints.insert(region.to_string(), "hit".to_string());
        // Estimate tokens saved (approx size of the compressed content)
        let saved = breaker.len() * 4; // rough chars/4 estimate
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

    // Inject into _meta.cache_hints
    if let Some(meta) = response.get_mut("_meta") {
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("cache_hints".to_string(), serde_json::to_value(&hints).unwrap_or_default());
        }
    } else {
        // Create _meta if not present
        let meta_obj = serde_json::json!({ "cache_hints": hints });
        response["_meta"] = meta_obj;
    }

    // Record as miss (first emission)
    state.cache_metrics.misses += 1;
    state.cache_metrics.breakpoints.insert(region.to_string(), "miss".to_string());
    0
}

/// Update cache metrics for the rolling tail (always ephemeral).
pub fn mark_tail_ephemeral(state: &mut crate::mcp::McpState) {
    state.cache_metrics.breakpoints.insert("tail".to_string(), "ephemeral".to_string());
}

/// Render the cache metrics section for the dashboard.
pub fn render_cache_text(metrics: &CacheMetrics) -> String {
    let hit_rate = if metrics.hits + metrics.misses > 0 {
        metrics.hits as f64 / (metrics.hits + metrics.misses) as f64
    } else {
        0.0
    };
    format!(
        "── Prompt Cache ──\n  Status: enabled\n  Hits: {} | Misses: {} | Hit Rate: {:.0}%\n  Tokens Saved (est): {}\n",
        metrics.hits,
        metrics.misses,
        hit_rate * 100.0,
        metrics.tokens_saved,
    )
}

/// Render the cache metrics as a JSON value.
pub fn render_cache_json(metrics: &CacheMetrics) -> serde_json::Value {
    let hit_rate = if metrics.hits + metrics.misses > 0 {
        metrics.hits as f64 / (metrics.hits + metrics.misses) as f64
    } else {
        0.0
    };
    serde_json::json!({
        "enabled": true,
        "hits": metrics.hits,
        "misses": metrics.misses,
        "hit_rate": (hit_rate * 100.0).round() / 100.0,
        "tokens_saved": metrics.tokens_saved,
        "breakpoints": metrics.breakpoints,
    })
}
```

---

## Phase 3: MCP Prompts Resource

### File: `src/mcp/tools.rs` (add dispatch)

Add two new MCP methods: `prompts/list` and `prompts/get`.

### `prompts/list` handler

Returns the available loadable prompt resources:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "prompts": [
      {
        "name": "clean-ctx-vocabulary",
        "description": "Clean-CTX opcode/marker vocabulary for reading compressed code context. Contains all recognized opcodes ($c, ⊕guard, Φcmp, etc.) and their meanings.",
        "arguments": []
      }
    ],
    "_meta": {
      "cache_hints": {
        "breakpoints": [
          { "region": "system_prompt", "ttl": "1h", "breaker": "vocab-v1" }
        ]
      }
    }
  }
}
```

### `prompts/get` handler for `clean-ctx-vocabulary`

Returns the full opcode/marker vocabulary text:

```
$c   → class
$i   → interface  
⊕    → decorator/annotation
⊕guard → conditional branch (guard clause)
Φ    → Angular composite operation
Φcmp → Angular component
Φdir → Angular directive
Φpipe → Angular pipe
Φserv → Angular service
Φmod → Angular module
Φguard → Angular route guard
Φres → Angular resolver
⊕Input → @Input() decorator
⊕Output → @Output() decorator  
⊕HostListener → @HostListener() decorator
⊕Inject → @Inject() decorator
⊕ViewChild → @ViewChild() decorator
⊕ViewChildren → @ViewChildren() decorator
[→ array (list literal/type)
~→ function/method
-> return type
?→ nullable/optional
!→ non-null assertion
=→ default value/equality
γ→ type parameter
→ type annotation/mapping
∞→ literal type/value
_→ unused/ignored
```

This text is generated from the opcode definitions already in `src/ir/opcodes.rs` (not hardcoded — use a generator function that reads from the canonical sources).

### Dispatch in `tools.rs`

```rust
// In dispatch_tools_call, or a separate dispatch entry:
match method {
    "prompts/list" => {
        handle_list_prompts(id, state);
    }
    "prompts/get" => {
        let name = params["arguments"]["name"].as_str().unwrap_or("");
        handle_get_prompt(id, name, state);
    }
    // ... existing tools/call dispatch
}
```

Note: The actual MCP JSON-RPC dispatch likely goes through `router.rs` or `server.rs`. We need to add the `prompts/list` and `prompts/get` methods at the router level, not inside `tools/call`.

---

## Phase 4: Tools List Annotations

### File: `src/mcp/tools.rs` — modify `tool_list()` response wrapper

The `tools/list` method handler currently returns the raw tool array. When `cache.enabled`, wrap with `_meta.cache_hints`:

```rust
pub(crate) fn handle_list_tools(id: &Value, state: &mut McpState) {
    let mut response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": tool_list()
        }
    });
    if state.config.cache.enabled {
        let breaker = format!("tools-{}", state.config.cache.tool_defs_version);
        inject_cache_breakpoints(
            &mut response,
            state,
            "tools",
            &state.config.cache.tools_ttl,
            &breaker,
        );
    }
    send_response(&response);
}
```

---

## Phase 5: `provide_code_context` Baseline Caching

### File: `src/mcp/tool_handlers.rs` — modify `handle_provide_code_context`

### Baseline Response (first compression for a file, or content unchanged)

```rust
// Before sending the full-compress response:
let mut response = serde_json::json!({
    "jsonrpc": "2.0",
    "id": id,
    "result": {
        "content": [{ "type": "text", "text": &compressed_text }],
        "_meta": {
            "fidelity": format!("{:?}", decision.fidelity).to_lowercase(),
            "strategy": "full_compress",
            // ... existing meta fields
        }
    }
});

// Inject baseline cache breakpoint
if state.config.cache.enabled {
    let breaker = compute_baseline_breaker(&compressed_text);
    inject_cache_breakpoints(
        &mut response,
        state,
        "baseline",
        &state.config.cache.baseline_ttl,
        &breaker,
    );
}
send_response(&response);
```

### Delta Response (subsequent edits with changes)

```rust
// Inject tail breakpoint (always 5m, ephemeral)
if state.config.cache.enabled {
    inject_cache_breakpoints(
        &mut response,
        state,
        "tail",
        &state.config.cache.tail_ttl,
        "rolling", // always "rolling" — never cache across turns
    );
    mark_tail_ephemeral(state);
}
```

### Workspace Baseline (compress_workspace)

When `compress_workspace` completes successfully, compute a composite workspace breaker:

```rust
let file_hashes: Vec<String> = /* collect hashes from workspace result */;
let ws_breaker = compute_workspace_breaker(&file_hashes);
inject_cache_breakpoints(&mut response, state, "workspace_baseline", &state.config.cache.baseline_ttl, &ws_breaker);
```

### Excluded / Error responses — skip cache hints (no change, just don't call `inject_cache_breakpoints`)

### `restore_context` responses — EMIT baseline hints (it's stable persisted state)

---

## Phase 6: `context_stats` Cache Section

### File: `src/mcp/tool_handlers.rs` — modify `handle_context_stats`

### Text Dashboard

Add cache section after the tokenizer line:

```rust
// After existing dashboard text rendering
if state.config.cache.enabled {
    text.push_str(&render_cache_text(&state.cache_metrics));
}
```

### JSON Dashboard

Add `cache` field:

```rust
if format == "json" {
    let mut json = render_dashboard_json(&merged);
    json["tokenizer"] = serde_json::json!(state.config.tokenizer.to_string());
    json["persistence"] = serde_json::json!({"enabled": db_stats.is_some()});
    if state.config.cache.enabled {
        json["cache"] = render_cache_json(&state.cache_metrics);
    }
    // ... send response
}
```

---

## Phase 7: State Changes

### File: `src/mcp/state.rs`

Add to `McpState`:

```rust
/// Tracks which cache breakpoints have already been emitted this session.
/// Key format: "{region}::{breaker}" — e.g., "tools::tools-v1", "system_prompt::vocab-v1"
/// Deduplication prevents paying the 2.0× write multiplier on re-emission.
pub emitted_breakpoints: HashSet<String>,

/// Cache efficiency metrics for the dashboard.
pub cache_metrics: CacheMetrics,
```

Initialize in `McpState::new()`:

```rust
emitted_breakpoints: HashSet::new(),
cache_metrics: CacheMetrics::default(),
```

### File: `src/mcp/mod.rs`

Add:
```rust
pub mod cache_hints;
pub use cache_hints::{CacheHints, CacheBreakpoint, CacheMetrics,
    inject_cache_breakpoints, compute_baseline_breaker, compute_workspace_breaker,
    mark_tail_ephemeral, render_cache_text, render_cache_json};
```

---

## Phase 8: Tests

### New file: `src/tests/mcp/cache_hints.rs`

Test cases:

| # | Test | Description |
|---|---|---|
| 1 | `test_cache_config_defaults` | `CacheConfig::default()` returns expected values |
| 2 | `test_cache_config_serde` | Serialize/deserialize round-trip preserves all fields |
| 3 | `test_cache_disabled_skips_injection` | `enabled: false` suppresses all hints |
| 4 | `test_inject_system_prompt_hint` | Region="system_prompt", ttl="1h", breaker="vocab-v1" |
| 5 | `test_inject_tools_hint` | Region="tools", ttl="1h", breaker="tools-v1" |
| 6 | `test_inject_baseline_hint` | Region="baseline", ttl="1h", breaker="bl_<hash>" |
| 7 | `test_inject_tail_hint` | Region="tail", ttl="5m", breaker="rolling" |
| 8 | `test_emitted_dedup` | Same region+breaker not injected twice |
| 9 | `test_workspace_breaker` | Composite hash from file hash list |
| 10 | `test_cache_metrics_accumulate` | Hits, misses, tokens_saved increment correctly |
| 11 | `test_cache_dashboard_text` | Text dashboard includes cache section |
| 12 | `test_cache_dashboard_json` | JSON dashboard includes cache field |
| 13 | `test_restore_context_emits_hint` | Restore context response gets baseline hint |
| 14 | `test_excluded_file_skips_hint` | Excluded files do NOT get cache hints |
| 15 | `test_error_response_skips_hint` | Error responses do NOT get cache hints |

---

## Non-Cacheable Edge Cases (per user feedback)

| Scenario | Action |
|---|---|
| Excluded file (in `state.config.exclude_patterns`) | Skip `inject_cache_breakpoints` entirely |
| Error response (any `error` field in response) | Skip `inject_cache_breakpoints` entirely |
| `restore_context` response | EMIT baseline cache hints (it's stable state) |
| Partial workspace (some files excluded) | Emit hints only on the stable included portion |
| `compress_workspace` with errors | Skip — workspace may be incomplete |
| Rolling tail (delta content) | Always emit with `breaker: "rolling"` and `ttl: "5m"` |

**Guiding principle:** Cache iff content is deterministically reproducible from the same inputs. If it depends on runtime state, error conditions, or user-defined filters that might change, don't.

---

## Default `.clean-ctx.json` Config

The `generate_default_config()` in `src/main.rs` will include:

```json
"cache": {
    "enabled": true,
    "system_prompt_ttl": "1h",
    "tools_ttl": "1h",
    "baseline_ttl": "1h",
    "tail_ttl": "5m",
    "vocab_version": "v1",
    "tool_defs_version": "v1"
}
```

---

## Files Changed Summary

| Action | File | Phase |
|---|---|---|
| MODIFY | `src/config.rs` | 1 |
| CREATE | `src/mcp/cache_hints.rs` | 2 |
| MODIFY | `src/mcp/mod.rs` | 2 |
| MODIFY | `src/mcp/state.rs` | 2, 7 |
| MODIFY | `src/mcp/tools.rs` | 3, 4 |
| MODIFY | `src/mcp/tool_handlers.rs` | 5, 6 |
| MODIFY | `src/mcp/router.rs` (or `server.rs`) | 3 |
| MODIFY | `src/main.rs` | 1 |
| MODIFY | `.clean-ctx.json` | 1 |
| CREATE | `src/tests/mcp/cache_hints.rs` | 8 |
| MODIFY | `docs/CHANGELOG.md` | docs |
| MODIFY | `docs/DEVELOPER_DOCUMENTATION.md` | docs |
| MODIFY | `docs/ROADMAP.md` | docs |

---

## Execution Order

1. Phase 1: Configuration (`config.rs`, `main.rs`, `.clean-ctx.json`)
2. Phase 2: Cache hints module (`cache_hints.rs`, `mod.rs`, `state.rs`)
3. Phase 3: MCP prompts (`tools.rs`, `router.rs`)
4. Phase 4: Tools annotations (`tools.rs`)
5. Phase 5: Baseline caching (`tool_handlers.rs`)
6. Phase 6: Dashboard (`tool_handlers.rs`)
7. Phase 7: State changes (`state.rs`)
8. Phase 8: Tests (`cache_hints.rs`)
9. Documentation (`CHANGELOG.md`, `DEVELOPER_DOCUMENTATION.md`, `ROADMAP.md`)
10. Build & verify all tests pass

---

## Verification

- `cargo build` — compiles without errors
- `cargo test` — all 991 + ~15 new tests pass (1006+ total)
- `cargo clippy` — no new warnings
- Manual verification: run MCP server, call `prompts/list`, verify `_meta.cache_hints` in response
- Manual verification: call `provide_code_context` twice on same file, verify second call deduplicates breakpoint emission
- Manual verification: call `context_stats format=json`, verify `cache` field present