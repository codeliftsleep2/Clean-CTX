# CBM Filter-First Architecture Plan

**Status:** 📋 Plan Ready
**Created:** 2026-06-21
**Target Release:** v0.2.0
**Deprecates:** Post-compression CBM enrichment pattern

---

## Executive Summary

The current CBM integration adds compressed enrichment data **after** compression, wasting tokens. This plan restructures the pipeline so CBM symbol importance scores filter **which** code structures survive compression — low-importance symbols are dropped, high-importance symbols get full detail.

**Net effect:** CBM reduces token output instead of increasing it. The enrichment step is removed entirely.

---

## The Problem

### Current (Broken) Pipeline

```
1. Read file source
2. CBM Intel → adjusts global fidelity only (Low/Med/High for whole file)
3. Compress entire file at that fidelity
4. CBM Enrichment → adds MORE tokens (cbm_enrichment metadata)
5. send_response()

Result: CBM ADDS ~50 tokens per file. This is backward.
```

### Target Pipeline

```
1. Read file source
2. CBM Intel:
   → Gets per-symbol importance scores from graph bridge
   → Builds skip_set: all symbols with score < 0.4
   → Stores skip_set in McpState
3. Heuristics engine → decides strategy
4. Compression:
   → capture pipeline checks skip_set per capture (class, method, field)
   → Low-importance symbols: process() returns None (dropped)
   → High-importance symbols: included with full detail
   → CBM's effect is embedded in WHAT gets compressed
5. ❌ No post-compression enrichment (removed)
6. Prompt cache breakpoints
7. send_response()

Result: CBM REDUCES token output by 30-50% for noisy files.
```

---

## Architecture Changes

### New: `CbmFilterSet` in `McpState`

```rust
/// Per-file CBM filter state: symbols to skip during compression.
/// Populated by the CBM Intelligence Layer before compression runs.
/// The `HashSet<String>` contains symbol names (e.g., "UserService",
/// "parseHelper") that should be excluded from the compressed output.
pub struct CbmFilterState {
    /// Symbol names to skip for the current file being processed.
    /// Keyed by file path, value is the set of low-importance symbol names.
    pub skip_sets: HashMap<String, HashSet<String>>,
}
```

### Modified: `src/intelligence/fidelity.rs`

Current `cbm_informed_fidelity()` returns a `FidelityRecommendation` (ForceHigh/ForceLow/NoRecommendation). Add:

```rust
/// Build a skip set of low-importance symbols for a file.
/// Returns symbol names with score < 0.4 mapped to file path.
/// Returns empty set if CBM is unavailable or no low-importance symbols found.
pub fn build_cbm_skip_set(
    file_path: &str,
    symbol_importance: &HashMap<String, SymbolImportance>,
) -> HashSet<String> {
    let mut skip = HashSet::new();
    for info in symbol_importance.values() {
        if info.score < 0.4 {
            // Check if this symbol's file matches our target
            let path_match = file_path.contains(&info.file) || info.file.contains(file_path);
            if path_match {
                skip.insert(info.symbol.clone());
            }
        }
    }
    skip
}
```

### Modified: Capture pipeline — `src/compression/pipeline.rs`

In `build_output_lines()` and `compress_text()`, the `process` closure now receives access to the `CbmFilterState` skip set:

```rust
// Before emitting a class/method/field, check if it's in the CBM skip set
// (only when CBM intelligence is enabled)
if should_skip_capture(&cap.text, state) {
    continue; // or return None from process closure
}
```

`should_skip_capture()` checks if the capture name (class name, method name) is in the current file's skip set.

### Modified: IR Compiler — `src/ir/compiler.rs`

Similar check in `IRCompiler::compile()`: before emitting `DefClass`, `DefMethod`, or `DefField`, check the skip set.

### Removed: `enrich_with_cbm()` and `enrich_workspace_with_cbm()`

These functions in `src/mcp/tool_handlers.rs` added post-compression CBM metadata. Remove them. Their token savings tracking moves to `SessionStats` with domain `"cbm_filter"`.

---

## Phase Plan

### Phase 1: Core Filter Architecture (4-5 days)

**Goal:** Wire CBM skip set into the compression pipeline. Remove enrichment. Add domain-tagged dashboard.

| Step | File(s) | Change | Tests |
|------|---------|--------|-------|
| 1.1 | `src/mcp/state.rs` | Add `CbmFilterState` to `McpState` with `skip_sets: HashMap<String, HashSet<String>>` | Unit: filter state CRUD |
| 1.2 | `src/intelligence/fidelity.rs` | Add `build_cbm_skip_set()` function | Unit: skip set building |
| 1.3 | `src/mcp/tool_handlers.rs` | Replace enrichment block with skip-set population; remove `enrich_with_cbm()` calls (lines 858, 1024) | Integration: CBM filter produces correct skip set |
| 1.4 | `src/compression/pipeline.rs` | Wire `should_skip_capture()` into `build_output_lines()` and `compress_text()` | Unit: low-importance capture dropped |
| 1.5 | `src/ir/compiler.rs` | Wire skip check before `DefClass`/`DefMethod`/`DefField` emission | Unit: IR excludes low-importance methods |
| 1.6 | `src/mcp/tool_helpers.rs` | Thread `&CbmFilterState` through `compress_text_body()` | Integration: full pipe |
| 1.7 | `src/mcp/session_stats.rs` | Add `domain` field to `record_compression()`; update dashboard text/JSON renderers | Unit: domain grouping |

### Phase 2: Proxy Filter Stats + Dashboard (3-4 days)

**Goal:** Wire proxy `FilterStats` into the MCP server dashboard. Add `GET /stats` endpoint to proxy.

| Step | File(s) | Change | Tests |
|------|---------|--------|-------|
| 2.1 | `proxy/src/server.rs` | Add `GET /stats` HTTP endpoint returning `FilterStats` + `CacheStats` as JSON | E2E: curl /stats |
| 2.2 | `src/mcp/tool_handlers.rs` | In `handle_context_stats()`, fetch proxy stats via HTTP | Integration: dashboard includes proxy stats |
| 2.3 | `src/mcp/session_stats.rs` | Add tool-filtering section to dashboard text/JSON renderers | Unit: proxy section formatting |

### Phase 3: Cleanup (2-3 days)

**Goal:** Remove dead code, update docs, deprecate old enrichment pattern.

| Step | File(s) | Change | Tests |
|------|---------|--------|-------|
| 3.1 | `src/mcp/tool_handlers.rs` | Remove `enrich_with_cbm()` and `enrich_workspace_with_cbm()` functions entirely | Verify no callers |
| 3.2 | `src/cbm/json_compress.rs` | Remove `compress_cbm_response()` if no longer called | Verify no callers |
| 3.3 | `src/tests/mcp/tool_handlers.rs` | Remove or update enrichment tests | Tests pass |
| 3.4 | `docs/CBM_INTEGRATION_PLAN.md` | Update to reflect filter-first architecture | N/A |
| 3.5 | `docs/ARCHITECTURE_OVERVIEW.md` | Update pipeline diagram | N/A |

### Phase 4: Per-Domain Token Savings Dashboard (2-3 days)

**Goal:** Complete the dashboard so each domain shows individual savings with a clean total.

| Step | File(s) | Change | Tests |
|------|---------|--------|-------|
| 4.1 | `src/mcp/session_stats.rs` | Per-domain summary in `SessionSummary` (add `domain_breakdown: HashMap<String, DomainStats>`) | Unit: domain aggregation |
| 4.2 | `src/mcp/session_stats.rs` | `render_dashboard_text()` shows per-domain section | Unit: text output format |
| 4.3 | `src/mcp/session_stats.rs` | `render_dashboard_json()` includes `"domain_breakdown"` key | Unit: JSON output format |
| 4.4 | Proxy stats endpoint | Wire proxy `FilterStats` into domain breakdown | E2E: full dashboard |
| 4.5 | Cache metrics | Wire `CacheMetrics` into domain breakdown | Integration: cache domain |

---

## Data Model: Domain-Tagged `record_compression()`

```rust
/// Domain identifier for token savings tracking.
/// Each domain operates on a completely separate token stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SavingsDomain {
    /// Compiler-IR compression (source code → compressed IR).
    /// This is the primary savings domain. CBM filtering is embedded here
    /// (low-importance symbols are dropped during compression).
    IrCompression,
    /// CBM filtering (symbols dropped before compression).
    /// Only recorded when the CBM skip set actually excluded symbols.
    /// Measured as: estimated tokens of dropped captures.
    CbmFilter,
    /// Prompt cache breakpoint dedup (tokens NOT re-sent to LLM).
    /// Hits represent tokens that the Anthropic API did not re-encode.
    PromptCache,
    /// Proxy tool output filtering (noisy CLI stdout → filtered).
    /// Reduces tokens sent through the Anthropic API.
    ToolFilter,
}
```

### No-Duplication Guarantee

Each `record_compression()` call MUST use exactly one domain. The domains are naturally disjoint:

| Domain | Input Tokens | Output Tokens | Source |
|--------|-------------|---------------|--------|
| `IrCompression` | raw source code | compressed IR text | `session_stats.record_compression()` in handlers |
| `CbmFilter` | estimated tokens of dropped captures | 0 (dropped) | Computed at skip-set build time |
| `PromptCache` | N/A (hit-based) | N/A | `CacheMetrics.tokens_saved` |
| `ToolFilter` | raw CLI stdout | filtered stdout | Proxy `FilterStats.total_tokens_saved` |

**Enforcement:** Add a debug-assertion test that no file has overlapping domain entries. Each `FileStats` entry can only have one domain.

---

## Dashboard Output (Final)

### Text Format

```
═══════════════════════════════════════════════════════════════
  Clean-CTX Dashboard — Session Stats
═══════════════════════════════════════════════════════════════
  Session Duration: 12m 34s
  Files Tracked: 47

── Per-Domain LLM Token Savings ──
  IR Compression:      245,000 →  67,000   (72.7%↓)
  CBM Filter:                    4,200 tokens removed
  Prompt Cache:                  3,200 tokens saved (hits)
  Tool Filtering:       98,000 →  14,000   (85.7%↓)
  ─────────────────────────────────────────────
  Total to LLM:        343,000 →  84,200   (75.5%↓)

── Per-File Breakdown ──
  ...
```

### JSON Format

```json
{
  "session": {
    "duration_secs": 754,
    "total_files": 47,
    "domain_breakdown": {
      "ir_compression": {
        "total_raw_tokens": 245000,
        "total_compressed_tokens": 67000,
        "savings_pct": 72.7
      },
      "cbm_filter": {
        "tokens_removed": 4200
      },
      "prompt_cache": {
        "tokens_saved": 3200,
        "hits": 15,
        "misses": 3
      },
      "tool_filter": {
        "total_raw_tokens": 98000,
        "total_compressed_tokens": 14000,
        "savings_pct": 85.7
      }
    },
    "total_raw_tokens": 343000,
    "total_compressed_tokens": 84200,
    "total_savings_pct": 75.5
  },
  "files": [...]
}
```

---

## Testing Strategy

### Unit Tests (Phase 1)

| Test | File | What It Verifies |
|------|------|------------------|
| `test_build_skip_set_low` | `intelligence/fidelity.rs` | Symbols with score < 0.4 → included in skip set |
| `test_build_skip_set_medium` | `intelligence/fidelity.rs` | Score 0.4-0.8 → NOT in skip set |
| `test_build_skip_set_high` | `intelligence/fidelity.rs` | Score > 0.8 → NOT in skip set |
| `test_build_skip_set_empty` | `intelligence/fidelity.rs` | Empty importance map → empty skip set |
| `test_build_skip_set_unrelated` | `intelligence/fidelity.rs` | Symbols in different file → not in skip set |
| `test_skip_capture_class` | `compression/pipeline.rs` | Class in skip set → dropped from output |
| `test_skip_capture_method` | `compression/pipeline.rs` | Method in skip set → dropped from output |
| `test_skip_capture_ir_class` | `ir/compiler.rs` | Class in skip set → no `DefClass` in IR |
| `test_skip_capture_ir_method` | `ir/compiler.rs` | Method in skip set → no `DefMethod` in IR |
| `test_domain_tagging` | `mcp/session_stats.rs` | `record_compression` with different domains → correctly grouped |
| `test_domain_summary` | `mcp/session_stats.rs` | `summary()` includes per-domain breakdown |
| `test_no_domain_overlap` | `mcp/session_stats.rs` | File cannot have entries in multiple domains |
| `test_dashboard_text_domains` | `mcp/session_stats.rs` | Text renderer includes all 4 domains |
| `test_dashboard_json_domains` | `mcp/session_stats.rs` | JSON renderer includes `"domain_breakdown"` |

### Integration Tests (Phase 1 + 2)

| Test | What It Verifies |
|------|------------------|
| `test_cbm_filter_in_pipeline` | Full `provide_code_context` with mock CBM skip set → low-importance captures dropped |
| `test_cbm_filter_token_savings_recorded` | Skip set → `session_stats` has matching `CbmFilter` domain entry |
| `test_no_enrichment_after_removal` | `enrich_with_cbm` no longer called (response has no `cbm_enrichment` field) |
| `test_proxy_stats_endpoint` | `GET /stats` returns valid JSON with `FilterStats` and `CacheStats` |
| `test_dashboard_includes_proxy` | `handle_context_stats()` shows tool filtering section when proxy available |

### E2E Tests (Phase 2)

| Test | What It Verifies |
|------|------------------|
| `cbm_filter_e2e` | CBM server running → skip set populated → compressed output has fewer tokens than without CBM |
| `dashboard_e2e` | Full session (compress + CBM filter + proxy filter + cache hits) → dashboard shows all 4 domains with correct totals |

### Tests to Remove

| Test | Reason |
|------|--------|
| `test_cbm_enrichment_*` in `mcp/tool_handlers.rs` | `enrich_with_cbm()` is being removed |
| Tests for `compress_cbm_response` (if `json_compress.rs` is removed) | No longer needed |
| Workspace enrichment tests in `cbm/integration.rs` | `enrich_workspace_with_cbm()` is being removed |

---

## File Change Summary

| File | Action | Est. Δ Lines | Phase |
|------|--------|-------------|-------|
| `src/mcp/state.rs` | Add `CbmFilterState` struct + field | +30 | 1 |
| `src/intelligence/fidelity.rs` | Add `build_cbm_skip_set()` | +40 | 1 |
| `src/mcp/tool_handlers.rs` | Replace CBM enrich with skip-set build; add proxy stats fetch | -120 / +80 | 1, 2 |
| `src/compression/pipeline.rs` | Wire `should_skip_capture()` | +30 | 1 |
| `src/ir/compiler.rs` | Wire skip set in `compile()` | +25 | 1 |
| `src/mcp/tool_helpers.rs` | Thread `CbmFilterState` through | +10 | 1 |
| `src/mcp/session_stats.rs` | Add `SavingsDomain` enum, domain field, per-domain renderers | +120 | 1, 4 |
| `proxy/src/server.rs` | Add `GET /stats` endpoint | +60 | 2 |
| `proxy/src/filter_stats.rs` | Export `FilterStats` serialization | +5 | 2 |
| `src/tests/mcp/tool_handlers.rs` | Update/remove enrichment tests | -50 / +40 | 3 |
| `docs/CBM_INTEGRATION_PLAN.md` | Update pipeline description | -20 / +30 | 3 |
| `docs/ARCHITECTURE_OVERVIEW.md` | Update pipeline diagram | -10 / +15 | 3 |

**Total:** ~340 new lines, ~200 removed, 12 files modified

---

## Execution Order

```
Phase 1 (Core Filter)     → Phase 2 (Proxy Stats)     → Phase 3 (Cleanup)      → Phase 4 (Dashboard)
   1.1 state.rs               2.1 proxy/server.rs          3.1 remove enrich        4.1 domain summary
   1.2 fidelity.rs            2.2 handle_context_stats     3.2 remove json_compress 4.2 text renderer
   1.3 tool_handlers.rs       2.3 session_stats/proxy      3.3 update tests         4.3 JSON renderer
   1.4 pipeline.rs                                       3.4 update docs          4.4 proxy wiring
   1.5 ir/compiler.rs                                    3.5 update ARCH docs     4.5 cache wiring
   1.6 tool_helpers.rs
   1.7 session_stats/domain
```

Each phase produces compilable, tested code before moving to the next.