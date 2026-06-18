# CBM API Audit & Phase 2 Integration Plan

**Status:** ✅ Done  
**Created:** 2026-06-18  
**Last updated:** 2026-06-18 (Phases 2a, 2b, 2c complete)  
**Based on:** Live testing with CBM v0.8.1 against Clean-CTX repository  
**Roadmap ID:** R-35 (Phase 2)  
**Target Release:** v0.2.0  

---

## Executive Summary

This document captures the findings from a live integration test between Clean-CTX and codebase-memory-mcp (CBM) v0.8.1. The original CBM integration plan (`CBM_INTEGRATION_PLAN.md`) was written before CBM's actual API was verified. This audit reveals **significant mismatches** between what our `CbmClient` expects and what CBM actually exposes, and provides a corrected plan for Phase 2.

**Key finding:** CBM has no `get_symbol_importance` or `get_dead_code` tools. However, CBM's `query_graph` (Cypher) and `search_graph` tools expose the same data through different interfaces. The integration is still viable — we just need to use the correct API.

**All three phases (2a, 2b, 2c) are now complete.** 45/45 CBM tests pass with 0 warnings.

---

## 1. CBM v0.8.1 — Actual API Surface

### Tools That Exist (verified via live test)

| Tool | Parameters | Returns | Works? |
|------|-----------|---------|--------|
| `index_repository` | `repo_path` (string, required) | `{project, status, nodes, edges}` | ✅ |
| `list_projects` | none | `{projects: [...]}` | ✅ |
| `search_graph` | `label`, `name_pattern`, `file_pattern`, `project`, `limit`, `offset`, `min_degree`, `max_degree` | `{total, results: [{name, qualified_name, label, file_path, in_degree, out_degree, ...}]}` | ✅ |
| `trace_path` | `function_name`, `direction` ("inbound"\|"outbound"\|"both"), `depth`, `project` | `{edges: [{from, to, label}]}` | ✅ |
| `query_graph` | `query` (Cypher string), `project` | `{columns: [...], rows: [[...], ...], total}` | ✅ |
| `get_architecture` | `project` | `{modules, dependencies, ...}` | ✅ |
| `detect_changes` | `project` | `{changes: [...], graph_version}` | ✅ |
| `get_graph_schema` | `project` | Schema metadata | ✅ |
| `get_code_snippet` | `qualified_name`, `project` | Source code snippet | ✅ |
| `search_code` | `pattern`, `project` | Grep-like results | ✅ |
| `manage_adr` | `mode`, `project` | ADR management | ✅ |
| `ingest_traces` | trace data | Status | ✅ |

### Tools That Do NOT Exist

| Tool We Call | CBM Response | Replacement |
|-------------|-------------|-------------|
| `get_symbol_importance` | `-32601 (Method not found)` | `query_graph` with Cypher: `MATCH (f:Function) RETURN f.name, f.in_degree, f.file_path ORDER BY f.in_degree DESC` |
| `get_dead_code` | `-32601 (Method not found)` | `query_graph` with Cypher: `MATCH (f:Function) WHERE f.in_degree = 0 RETURN f.name, f.file_path` |

### Parameter Mismatches in Our Code (FIXED)

| Our `CbmClient` wrapper | Was Sending | Now Sends |
|------------------------|-------------|-----------|
| `search_graph` | `{"query": "...", "project": "..."}` | `{"name_pattern": "...", "label": "...", "project": "..."}` |
| `trace_path` | `{"from": "...", "to": "...", "project": "..."}` | `{"function_name": "...", "direction": "...", "depth": N, "project": "..."}` |

---

## 2. CBM Response Format (Verified)

### `search_graph` Response

CBM returns results wrapped in MCP `content` array format. The actual data is a JSON string inside `result.content[0].text`:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"total\":78,\"results\":[{\"name\":\"apply_minimum_compression\",\"in_degree\":3,\"out_degree\":2,\"file_path\":\"src/cbm/proxy.rs\",...}]}"
    }]
  }
}
```

Key properties on each function node:
- `name` — function name
- `qualified_name` — fully qualified path
- `file_path` — relative file path
- `in_degree` — **number of callers** (our importance metric!)
- `out_degree` — number of calls made
- `complexity` — cyclomatic complexity
- `lines` — line count
- `is_exported`, `is_test`, `is_entry_point` — boolean flags
- `signature`, `return_type` — type info

### `query_graph` (Cypher) Response

Returns `{columns, rows}` format:
```json
{"columns":["f.name","f.file_path","f.in_degree","f.out_degree"],"rows":[["render_hierarchical_for_llm","src/ir/render_llm.rs","40","4"],...],"total":5}
```

---

## 3. What Was Changed (All Complete)

### 3.1 `CbmClient` Typed Wrappers (`src/cbm/client.rs`)

All 6 wrappers fixed:
1. **`search_graph`** — Parameter changed from `query` → `name_pattern`, added optional `label`
2. **`trace_path`** — Changed from `from`/`to` → `function_name`/`direction`, added optional `depth`
3. **`parse_cbm_response`** — New helper to extract JSON from CBM's MCP `content[0].text` wrapper
4. **`get_symbol_importance`** — Replaced non-existent tool call with Cypher: `MATCH (f:Function) WHERE f.in_degree >= N RETURN f.name, f.file_path, f.in_degree, f.out_degree ORDER BY f.in_degree DESC`
5. **`get_dead_code`** — Replaced non-existent tool call with Cypher: `MATCH (f:Function) WHERE f.in_degree = 0 AND f.is_entry_point = false RETURN f.name, f.file_path`
6. **`query_graph`** — Return type changed to `Vec<Vec<Value>>` to match CBM's `{columns, rows}` format

### 3.2 `GraphBridge` Updates (`src/cbm/bridge.rs`)

4 call sites updated:
- `get_symbol_importance_mut` — passes `min_degree: Some(1)` to corrected client
- `search`/`get_blast_radius` — passes `label: None` to corrected `search_graph`
- `trace_path` — passes `function_name`, `direction: "both"`, `depth: Some(3)` to corrected client
- `query_graph` — handles `Vec<Vec<Value>>` rows from corrected client

### 3.3 Intelligence Layer (No Changes Needed)

Already implemented correctly:
- `src/intelligence/pagerank.rs` — 60% IR + 40% CBM blend
- `src/intelligence/fidelity.rs` — `ForceHigh`/`ForceLow` based on CBM importance
- `src/mcp/tool_handlers.rs` — `enrich_with_cbm()` calls `bridge.get_symbol_importance_mut()`

### 3.4 Test Fix

- `enrich_with_cbm_degraded_status_skips_enrichment` — Fixed to disable CBM in config (`config.cbm.enabled = false`) so the test doesn't fail when CBM is installed locally. 45/45 tests pass.

---

## 4. Phase 2 Integration Architecture

### 4.1 Data Flow

```
provide_code_context(file.rs)
  |
  ├─ 1. Compress file via tree-sitter → IR → micro-opcodes (existing)
  |
  ├─ 2. Query CBM for cross-file importance (if available)
  |     └─ Cypher: MATCH (f:Function) WHERE f.in_degree >= 1
  |                RETURN f.name, f.file_path, f.in_degree, f.out_degree
  |
  ├─ 3. Blend CBM scores into PageRank
  |     └─ ir_scores (60%) + cbm_scores (40%) → combined_importance
  |
  ├─ 4. Apply adaptive fidelity per-symbol
  |     └─ high_importance → Fidelity::High (preserve full body)
  |     └─ medium_importance → Fidelity::Medium (collapse body)
  |     └─ low_importance → Fidelity::Low (keep signature only)
  |
  └─ 5. Return compressed output with ⊕important/⊕leaf markers
```

---

## 5. CBM Response Size & Compression Savings

### Measured from live test

| CBM Tool | Raw Response Size | Est. Compressed | Savings |
|----------|------------------|-----------------|---------|
| `search_graph` (78 results) | ~15,000 tokens | ~3,000 tokens | ~80% |
| `query_graph` (5 rows) | ~500 tokens | ~150 tokens | ~70% |
| `get_architecture` | ~2,000 tokens | ~500 tokens | ~75% |

The `cbm_proxy` tool (Phase 1) handles the large responses. Phase 2 uses `query_graph` with targeted Cypher queries, which are much smaller and don't need compression — they return structured data that Clean-CTX processes internally.

---

## 6. Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Symbol importance source** | CBM `in_degree` via Cypher | Already computed by CBM, no extra indexing cost |
| **Dead code detection** | CBM Cypher: `in_degree = 0` | More accurate than our IR alone (cross-file aware) |
| **Blast radius** | CBM `detect_changes` tool | Maps git diff → affected symbols with risk classification |
| **PageRank blend** | 60% IR + 40% CBM | IR is always available; CBM enriches when present |
| **CBM availability** | Optional, degrade gracefully | Clean-CTX works perfectly without CBM |
| **Cypher queries** | Hardcoded in client wrappers | Simple queries, no need for a full Cypher builder |

---

## 7. Open Questions

1. **CBM project name format** — CBM uses `C-Users-MNasty-Desktop-RustContextLayerAI` (path with special chars replaced). Our `GraphBridge` auto-detects from directory name. Does this always match what CBM expects?

2. **CBM indexing latency** — CBM took ~60 seconds to index our 319-file project. For Phase 2, we need CBM to have already indexed the project before we query it. Should we trigger indexing on session start?

3. **Cypher query performance** — `MATCH (f:Function) WHERE f.file_path = '...'` — does CBM have an index on `file_path`? For large projects, this could be slow without one.

4. **`query_graph` return format** — CBM returns `{columns, rows}` but our `QueryResult` expects `{nodes, edges}`. Should we add a separate `CypherResult` type?

---

## 8. References

- [CBM GitHub Repository](https://github.com/DeusData/codebase-memory-mcp)
- [CBM v0.8.1 README](https://github.com/DeusData/codebase-memory-mcp) (158 languages, 14 MCP tools)
- [Clean-CTX CBM Integration Plan](./CBM_INTEGRATION_PLAN.md) (original, pre-audit)
- [Clean-CTX Intelligence Layer Plan](./INTELLIGENCE_LAYER_PLAN.md)
- [Clean-CTX Roadmap](./ROADMAP.md) (R-35)