# CBM Integration — FAANG Audit v1.0

**Status:** ✅ Complete (Post-Remediation + Circuit Breaker Audit)  
**Date:** 2026-06-18  
**Scope:** Full CBM integration codebase + circuit breaker/resilience layer  
**Audit Type:** Code review — correctness, completeness, and reliability  
**CBM Version:** v0.8.1  
**Referenced Docs:** `CBM_API_AUDIT_AND_PHASE2_PLAN.md`, `CBM_INTEGRATION_PLAN.md`  

---

## Audit Summary

| Severity | Count | Status |
|----------|------:|--------|
| CRITICAL | 0 | — |
| HIGH | 1 | 🟡 Needs fix |
| MEDIUM | 4 | 🟡 Needs fix |
| LOW | 3 | 🟢 Acceptable |
| INFO | 4 | ℹ️ Noted |

**Overall verdict:** The CBM integration is structurally sound. The client communication layer is correct, the proxy pipeline works, and the intelligence layer blending (pagerank + fidelity) is properly designed. **1 HIGH finding** requires attention: `bridge.search()` and `get_blast_radius()` pass semantically wrong queries to CBM and will silently return empty results. **4 MEDIUM findings** relate to handler/bridge API mismatches and the `query_graph` bridge method losing column data. No data loss, no crashes, no security issues.

---

## HIGH Severity Findings

### H-01: `bridge.search()` and `get_blast_radius()` pass incorrect name_patterns to CBM

**File:** `src/cbm/bridge.rs`, lines 341-372, 199-223  
**Impact:** All `search()` and `get_blast_radius()` calls silently return empty results.

**Root Cause:** The bridge's `search()` method passes the raw query string as `name_pattern` to CBM's `search_graph`. But CBM's `name_pattern` expects a regex pattern matching function names (e.g., `".*compress.*"`). When a user calls `handle_graph_search` with query `"depends_on:UserService"`, this string is treated as a literal regex — it won't match any function name and will return empty results. Similarly, `get_blast_radius` constructs `"depends_on:{sym}"` which is also passed as a `name_pattern` regex — semantically meaningless.

**Reproduction:**
```rust
// bridge.search("depends_on:UserService")
// → calls client.search_graph("depends_on:UserService", project, None)
// → CBM treats name_pattern="depends_on:UserService" as regex
// → Matches zero functions (no function is literally named "depends_on:UserService")
// → Returns empty Vec
```

**Evidence from live CBM testing:** `search_graph` with `name_pattern: ".*compress.*"` correctly returns 78 results. Using `name_pattern: "depends_on:UserService"` would match 0 results.

**Fix:** 
```rust
// Option A: Use query_graph with Cypher for semantic queries
pub fn search(&mut self, query: &str) -> Vec<GraphNode> {
    let cypher = format!("MATCH (f:Function) WHERE f.name =~ '.*{}.*' RETURN f.name, f.file_path LIMIT 50", query);
    // Parse rows into GraphNodes...
}

// Option B: Pass query as a regex name_pattern directly (no "depends_on:" prefix)
pub fn search(&mut self, query: &str) -> Vec<GraphNode> {
    // Sanitize query — if user provides bare "UserService", treat as regex .*UserService.*
    let name_pattern = if query.contains(':') { query } else { &format!(".*{query}.*") };
    // ...
}
```

**Also affected:** `get_blast_radius()` — same issue. The `depends_on:` prefix is wrong for name_pattern.

---

## MEDIUM Severity Findings

### M-01: `handle_graph_trace` ignores `to` parameter — only traces from source

**File:** `src/cbm/handlers.rs` line 106, `src/cbm/bridge.rs` lines 375-409  
**Impact:** Graph trace handler accepts `from` and `to` but only traces `from → all` (direction "both"), discarding the target. Users requesting `trace from: "login", to: "authenticate"` will get ALL paths from `login`, not just those reaching `authenticate`.

**Root Cause:** `bridge.trace_path(from, to)` passes `from` as CBM's `function_name` and hardcodes `direction: "both"`. The `to` parameter is stored as `_t` (intentionally unused, per comment). CBM's `trace_path` doesn't support from→to filtering — it only traces from a single function.

**Fix Options:**
1. Post-filter results in the bridge: call `trace_path(from, "outbound", ...)` then filter edges where `to == target`
2. Use `query_graph` with Cypher: `MATCH path = (a:Function)-[*1..5]->(b:Function) WHERE a.name = 'from_fn' AND b.name = 'to_fn' RETURN path`
3. Update handler to accept `function_name` + `direction` instead of `from`/`to`

### M-02: `handle_graph_search` accepts `query` parameter but handler passes it as `name_pattern`

**File:** `src/cbm/handlers.rs` line 39  
**Impact:** The handler's MCP tool schema advertises a `query` parameter, but internally calls `bridge.search(query)` which passes it as a `name_pattern` regex to CBM. Users providing semantic queries like "what depends on UserService" will get empty results (see H-01).

**Fix:** Update the MCP tool schema to accept `name_pattern` and `label` parameters matching CBM's actual API, or update `bridge.search()` to translate queries to Cypher (see H-01).

### M-03: `bridge.query_graph()` loses all column data — only uses first column

**File:** `src/cbm/bridge.rs` lines 295-339  
**Impact:** When the bridge calls CBM's `query_graph`, the client returns `Vec<Vec<Value>>` with all columns. But the bridge's `query_graph()` method creates one `GraphNode` per row using only the first column as both `id` and `name`. Columns 2+ are discarded. The `edges` vec is always empty.

**Example:** A Cypher query returning `[name, file_path, in_degree, out_degree]` would produce nodes with `id=name`, `label=""`, `file=""`, `properties={}`, losing `file_path`, `in_degree`, and `out_degree`.

**Note:** This is acceptable for the handler's current use case (displaying node/edge counts), but breaks any caller that needs actual column data. The `get_symbol_importance`, `get_dead_code`, and `enrich_with_cbm` functions bypass `bridge.query_graph()` and use `client.query_graph()` directly, so they are unaffected.

**Fix:** Add a `query_graph_rows()` method to the bridge that returns `Vec<Vec<Value>>` directly, or add column parsing to the existing method.

### M-04: `get_blast_radius` uses semantically invalid `name_pattern`

**File:** `src/cbm/bridge.rs` lines 199-223  
**Impact:** `get_blast_radius(symbol)` constructs `name_pattern: "depends_on:{symbol}"` and passes it to `search_graph`. This is not a valid regex and will match 0 symbols. The function will always return `vec![]`.

**Root Cause:** Same as H-01 — `depends_on:` is not a valid CBM name_pattern prefix.

**Fix:** Use `query_graph` with Cypher:
```cypher
MATCH (f:Function) WHERE f.name = '{symbol}' 
OPTIONAL MATCH (f)<-[:CALLS]-(caller:Function)
RETURN caller.name, caller.file_path
```

---

## LOW Severity Findings

### L-01: Duplicate `MAX_RESPONSE_BYTES` constant

**File:** `src/cbm/client.rs` lines 40 and 265  
**Impact:** None — same value (4 MB) defined twice. Maintenance risk if one is changed without the other.

**Fix:** Remove the inner constant and use the module-level one.

### L-02: `Drop` implementation may hang on unresponsive child process

**File:** `src/cbm/client.rs` lines 419-423  
**Impact:** `child.kill()` followed by `child.wait()` — if the CBM process ignores the kill signal, this call hangs indefinitely. Low risk in practice (CBM is well-behaved).

**Fix:** Use `child.wait_timeout(Duration::from_secs(5))` after kill.

### L-03: `project_str()` clones on every call

**File:** `src/cbm/bridge.rs` line 161-163  
**Impact:** Minor allocation overhead on every bridge query. Trivial for the expected call frequency (<100 calls per session).

**Fix:** Accept and move on. Not worth optimizing.

---

## INFO: Items Verified as Correct

### I-01: Client subprocess communication ✅

`CbmClient::try_launch()` correctly:
- Spawns CBM as a subprocess with piped stdin/stdout/stderr
- Drains stderr in a background thread (H-1 regression guard)
- Uses JSON-RPC 2.0 over stdin/stdout with proper `tools/call` method
- Returns `Ok(None)` when binary not found (never panics)

### I-02: All typed wrapper parameter names match CBM v0.8.1 ✅

Verified against live testing:
- `search_graph`: `name_pattern`, `label`, `project` — ✅ correct
- `trace_path`: `function_name`, `direction`, `depth`, `project` — ✅ correct
- `query_graph`: `query` (Cypher string), `project` — ✅ correct
- `get_architecture`: `project` — ✅ correct
- `get_symbol_importance`: uses `query_graph` with valid Cypher — ✅ correct
- `get_dead_code`: uses `query_graph` with valid Cypher — ✅ correct

### I-03: `parse_cbm_response` correctly handles MCP content wrapper ✅

CBM returns `result.content[0].text` as a JSON-encoded string. The helper correctly:
1. Extracts the first content array element
2. Reads the `text` field
3. Parses it as JSON
4. Returns structured `Value`

### I-04: Intelligence Layer CBM blending ✅

`pagerank.rs` correctly:
- Normalizes IR scores to 0.0-1.0 range
- Normalizes CBM scores to 0.0-1.0 range
- Blends at 60% IR / 40% CBM
- Includes CBM-only symbols, IR-only symbols, and overlapping symbols

`fidelity.rs` correctly:
- Forces `High` for symbols with importance > 0.8
- Forces `Low` for symbols with importance < 0.4
- Returns `NoRecommendation` for medium-range or no matches

### I-05: Proxy pipeline (RC-1, RC-2) ✅

`handle_cbm_proxy` correctly:
- Forwards to CBM and intercepts at pipe level
- Uses JSON-aware compressor (not tree-sitter) — RC-1
- NEVER returns raw output — RC-2 minimum compression fallback
- Uses pluggable tokenizer for accurate token counts
- Records compression stats

### I-06: Graceful degradation ✅

All bridge methods return empty results when CBM is unavailable:
- `get_symbol_importance_mut()` → empty HashMap
- `get_dead_code()` → empty Vec
- `get_architecture()` → None
- `get_blast_radius()` → empty Vec
- `search()` → empty Vec
- `trace_path()` → empty Vec
- `detect_changes()` → Ok(None)

### I-07: Test coverage ✅

52 tests across 4 test files:
- 12 unit tests (json_compress)
- 23 regression tests (error handling, cache, compression)
- 5 integration tests (enrichment pipeline, envelope stripping)
- 7 E2E tests (3 live + 4 mock)
- 5 setup tests (binary detection, config)

### I-08: CBM response compression ✅

`compress_cbm_response` correctly:
- Strips JSON-RPC envelope
- Shortens 44 common JSON keys to 1-4 character codes
- Strips null values
- Achieves ~70-80% compression ratio as verified by live testing

---

## Remediation Plan

| ID | Priority | Effort | Description |
|----|----------|--------|-------------|
| H-01 | 🔴 High | 30 min | Fix `bridge.search()` to treat query as regex pattern, not prefix |
| M-01 | 🟡 Medium | 20 min | Add post-filtering to `trace_path` for from→to tracing |
| M-02 | 🟡 Medium | 15 min | Update `handle_graph_search` to accept `name_pattern` parameter |
| M-03 | 🟡 Medium | 20 min | Add `query_graph_rows()` method returning raw column data |
| M-04 | 🟡 Medium | 20 min | Rewrite `get_blast_radius` using Cypher `query_graph` |
| L-01 | 🟢 Low | 5 min | Remove duplicate `MAX_RESPONSE_BYTES` constant |
| L-02 | 🟢 Low | 5 min | Add timeout to `Drop` child wait |
| L-03 | 🟢 Low | — | No action needed |

**Total remediation effort:** ~2 hours

---

## References

- `src/cbm/client.rs` — JSON-RPC subprocess client
- `src/cbm/bridge.rs` — GraphBridge with TTL caching
- `src/cbm/handlers.rs` — MCP tool handlers
- `src/cbm/proxy.rs` — Pipe-level interception proxy
- `src/cbm/json_compress.rs` — JSON-aware compression
- `src/intelligence/pagerank.rs` — IR + CBM PageRank blending
- `src/intelligence/fidelity.rs` — CBM-informed fidelity selection
- `docs/CBM_API_AUDIT_AND_PHASE2_PLAN.md` — Phase 2 integration plan