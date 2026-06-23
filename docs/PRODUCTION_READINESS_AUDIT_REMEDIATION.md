# Production Readiness Audit — Remediation Plan

**Created:** 2026-06-18
**Status:** Plan — not yet started
**Audit:** FAANG Principal Engineer full-system architectural audit
**Source:** See audit report above for complete findings list (3 Blockers, 6 High, 6 Medium, 4 Low)

---

## Phase 1 — Blockers (Must fix before any release)

### B-03: `debug_log` writes to disk, contradicting SECURITY.md
**Effort:** 15 min | **Risk:** LOW

**Problem:** `debug_log()` in `src/mcp/tool_handlers.rs:17-31` opens `.clean-ctx/debug.log` and appends timestamps + messages. The SECURITY.md (line 190) states "No logging to disk". This is a security compliance issue for air-gapped/DLP environments.

**Fix:**
1. In `src/mcp/tool_handlers.rs`, remove the `debug_log()` function entirely (lines 17-31).
2. Remove the `use std::fs::OpenOptions` and `use std::io::Write` imports.
3. Replace all `debug_log(...)` calls with `eprintln!("[clean-ctx] ...")` calls.
4. Call sites to change:
   - Line 156: `debug_log(format!("handle_compress: persist_store={}", ...))` → `eprintln!`
   - Line 161: `debug_log(format!("handle_compress: calling save_context for {}", ...))` → `eprintln!`
   - Line 171: `debug_log(format!("handle_compress: save_context OK id={}", ...))` → `eprintln!`
   - Line 172: `debug_log(format!("handle_compress: save_context FAILED: {e}"))` → `eprintln!`
   - Line 175: `debug_log("handle_compress: persist_store is None, skipping")` → `eprintln!`
5. Also check `handle_provide_code_context` area for any `debug_log` calls and convert those.
6. Update SECURITY.md line 190: Change "No logging to disk" to document that stderr may contain operational messages, and no persistent log files are written.

**Regression Test — B-03: No disk logging**
- **File:** CI pipeline / grep check
- **Test name:** `no_debug_log_function_compiles`
- **Type:** Compile-time assertion (build test)
- **Assertion:** `grep -r "debug_log" src/` returns zero matches after fix
- **Also:** CI step that searches for `OpenOptions::new().create(true).append(true)` patterns in non-test, non-persistence source files — none should exist in handlers
- **E2E test (optional):** `security_guide_has_accurate_logging_statement` — parse SECURITY.md, verify line ~190 does NOT claim "No logging to disk" without qualification, and that `debug.log` is not referenced

---

### B-01: Intelligence Layer not wired into hot path
**Effort:** 2-3 hours | **Risk:** MEDIUM (core feature gap)

**Problem:** The `src/intelligence/` module (pagerank.rs, blast_radius.rs, fidelity.rs) exists and compiles, but `provide_code_context` handler never calls `cbm_informed_fidelity()`. The 60% IR + 40% CBM PageRank blending is dead code. The roadmap shows R-29 (Intelligence Layer) as in-progress, but zero user-visible effect exists.

**Fix:**
1. **Add config field** in `src/config.rs` `CleanCtxConfig`:
   ```rust
   /// Intelligence Layer configuration (CBM-informed fidelity, PageRank, blast radius).
   #[serde(default)]
   pub intelligence: IntelligenceConfig,
   ```
   With `IntelligenceConfig` struct:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct IntelligenceConfig {
       #[serde(default = "default_true")]
       pub enabled: bool,
   }
   impl Default for IntelligenceConfig { ... }
   ```

2. **Wire into heuristics** in `src/mcp/heuristics.rs`:
   - In `compute_strategy()`, after the existing classification, add:
   ```rust
   // CBM Intelligence Layer: if bridge is available and intel enabled,
   // consult cbm_informed_fidelity for per-symbol fidelity adjustment
   if state.config.intelligence.enabled {
       if let Some(ref mut bridge) = state.graph_bridge {
           if bridge.is_available() {
               let recommendation = crate::intelligence::fidelity::cbm_informed_fidelity(
                   file_path, bridge, &decision,
               );
               decision = crate::intelligence::fidelity::apply_recommendation(decision, recommendation);
           }
       }
   }
   ```

3. **Update default config** in `src/main.rs` `generate_default_config()`:
   ```json
   "intelligence": { "enabled": true }
   ```

4. **Add session_stats recording** for CBM-informed fidelity changes so the dashboard shows when intel influenced decisions.

**Integration Test — B-01: Intelligence Layer wired**
- **File:** `src/tests/cbm/integration.rs`
- **Test 1:** `intelligence_layer_influences_fidelity_when_cbm_available`
  - Setup: Create McpState with mocked GraphBridge returning high symbol importance for target file
  - Call: `handle_provide_code_context` with `intelligence.enabled = true`
  - Assert: `_meta.decision_summary` contains `cbm_informed: true` (or equivalent marker)
  - Assert: `_meta.fidelity` reflects CBM influence (e.g., was Medium, now High because ForceHigh recommendation)
- **Test 2:** `intelligence_layer_does_nothing_when_disabled`
  - Setup: McpState with `intelligence.enabled = false`, mocked bridge available
  - Call: `handle_provide_code_context`
  - Assert: `_meta.decision_summary` does NOT contain `cbm_informed`
  - Assert: fidelity matches default heuristic (no CBM override)
- **Test 3:** `intelligence_layer_does_nothing_when_bridge_unavailable`
  - Setup: McpState with `graph_bridge = None`, `intelligence.enabled = true`
  - Call: `handle_provide_code_context`
  - Assert: handler succeeds normally (no panic, no error)
  - Assert: `_meta.decision_summary` does not crash on missing bridge
- **Test 4:** `intelligence_config_defaults_to_enabled`
  - Setup: Default `CleanCtxConfig::default()`
  - Assert: `config.intelligence.enabled == true`

---

### B-02: `compress_workspace` receives zero CBM enrichment
**Effort:** 1 hour | **Risk:** LOW

**Problem:** `enrich_with_cbm()` is called from `handle_provide_code_context` but NOT from the `compress_workspace` handler in `tools.rs:358-423`. Workspace users get no CBM metadata.

**Fix:**
1. Create a workspace-aware enrichment function in `tool_handlers.rs` (or extend `enrich_with_cbm` to accept an optional file path for workspace mode):
   ```rust
   pub(crate) fn enrich_workspace_with_cbm(
       response: &mut serde_json::Value,
       state: &mut McpState,
   ) {
       // Similar to enrich_with_cbm but operates on workspace-level data
       // - Architecture overview (modules + dependencies)
       // - Top-10 most important symbols across the workspace
       // - Dead code count
   }
   ```

2. Call it from `tools.rs` in the `compress_workspace` dispatch, after the response is built but before `send_response()`:
   ```rust
   // After line 413 (after inject_cache_breakpoints block):
   enrich_workspace_with_cbm(&mut response, state);
   ```

**Integration/E2E Test — B-02: Workspace CBM enrichment**
- **File:** `src/tests/cbm/integration.rs`
- **Test 1:** `compress_workspace_receives_cbm_enrichment_when_available`
  - Setup: McpState with mocked GraphBridge, temp workspace dir with 2+ .ts files
  - Call: `compress_workspace` with the temp dir
  - Assert: Response `_meta.cbm_status` exists and equals `"available"`
  - Assert: Response `_meta.cbm_architecture_summary` exists with `modules` and `dependencies` keys
  - Assert: Response `_meta.cbm_enrichment` exists (workspace-level symbol summary)
- **Test 2:** `compress_workspace_gracefully_degrades_when_cbm_unavailable`
  - Setup: McpState with `graph_bridge = None`, temp workspace dir
  - Call: `compress_workspace`
  - Assert: Response succeeds (no error)
  - Assert: `_meta.cbm_status` equals `"unavailable"`
  - Assert: No `_meta.cbm_enrichment` field
- **E2E Test** (in `src/tests/cbm/e2e.rs`, requires live CBM):
  - `compress_workspace_enrichment_e2e` — live CBM binary on PATH, actual workspace dir, verify both per-file and workspace-level enrichment fields present

---

## Phase 2 — High Findings (Fix before v0.2.0 release)

### H-01: Bump Cargo.toml version to 0.1.9 or 0.2.0-rc1
**Effort:** 5 min | **Risk:** NONE

**Fix:** Change `version = "0.1.6"` in `Cargo.toml` line 7 to `version = "0.2.0-rc1"`.
Also update `docs/ARCHITECTURE_OVERVIEW.md` line 3 from `0.1.6` to `0.2.0-rc1`.

**Regression Test — H-01: Version consistency**
- **File:** `src/tests/mcp/regression.rs`
- **Test name:** `cargo_toml_version_matches_architecture_doc`
- **Type:** Unit test using `include_str!`
- **Assertion:** Parse `Cargo.toml` from project root, extract `version = "..."`, compare with `docs/ARCHITECTURE_OVERVIEW.md` first-line version string

---

### H-02: Remove `rayon` dependency or implement F-20
**Effort:** 5 min (removal) or 3-5 days (implementation) | **Risk:** LOW

**Decision needed:** If F-20 (Rayon parallelization) won't be done before launch, remove `rayon = "1.10"` from `Cargo.toml` line 53. It adds compile time, binary size, and supply chain surface with zero benefit.

**Fix (removal option):**
1. Remove `rayon = "1.10"` from `Cargo.toml`
2. Run `cargo update` to refresh lockfile
3. Verify build with `cargo build --release`

**Fix (implementation option):**
If you choose to implement F-20 now, see `docs/ROADMAP.md` F-20 for the plan.

**Build Test — H-02: No dead dependencies**
- **Type:** CI pipeline check
- **Assertion:** `cargo build --release` succeeds without `rayon` in `Cargo.lock` (if removal option chosen)
- **Ongoing:** CI step that runs `cargo udeps` (or manual grep) to ensure no unused dependencies in `Cargo.toml`

---

### H-03: Add integration test for `provide_code_context → CBM enrichment`
**Effort:** 1-2 hours | **Risk:** LOW

**Problem:** No integration test verifies the `_meta.cbm_enrichment` field is populated when CBM is available.

**Fix location:** `src/tests/cbm/integration.rs`

**New test cases:**
1. `provide_code_context_enriches_with_cbm_when_available` — set up McpState with a mock/stub GraphBridge, call `handle_provide_code_context`, verify `_meta.cbm_enrichment` exists and contains `text` field.
2. `provide_code_context_skips_cbm_when_degraded` — set cbm_status to "degraded", verify enrichment is skipped, response still succeeds.
3. `provide_code_context_skips_cbm_when_disabled` — set `intelligence.enabled = false`, verify no enrichment.

**Mock strategy:** Create a test-only `GraphBridge::new_mock()` or use existing test infrastructure to inject a bridge that returns canned data without needing a real CBM binary.

**Regression Test — H-03: Enrichment injection path**
- **File:** `src/tests/cbm/integration.rs`
- **Test 1:** `enrich_with_cbm_populates_meta_when_bridge_available`
  - Setup: McpState with mocked GraphBridge, `cbm_status = available`
  - Call: `enrich_with_cbm(&mut response, file_path, state)` with test response JSON
  - Assert: `response["result"]["_meta"]["cbm_status"]` == `"available"`
  - Assert: `response["result"]["_meta"]["cbm_enrichment"]["text"]` is a non-empty string
- **Test 2:** `enrich_with_cbm_skips_when_cbm_degraded`
  - Setup: McpState with `cbm_status.summary() = "degraded"`, bridge present but status degraded
  - Call: `enrich_with_cbm(&mut response, file_path, state)`
  - Assert: No `cbm_enrichment` field injected
  - Assert: `cbm_status` field injected (always injected, even when degraded)
- **Test 3:** `enrich_with_cbm_skips_when_no_bridge`
  - Setup: McpState with `graph_bridge = None`
  - Call: `enrich_with_cbm(&mut response, file_path, state)`
  - Assert: Response unchanged (no enrichment fields)
- **E2E Test:** `provide_code_context_enrichment_e2e` (in `src/tests/cbm/e2e.rs`, requires live CBM)
  - Live call to `provide_code_context`, verify `_meta.cbm_enrichment` present and compressed

---

### H-04: CBM tool handlers bypass circuit breaker health check
**Effort:** 30 min | **Risk:** LOW

**Problem:** CBM tool handlers (`handle_graph_search`, `handle_graph_query`, etc.) call bridge methods which hit the client. If circuit is open, each call still attempts I/O before failing.

**Fix:** At the top of each handler in `src/cbm/handlers.rs`, add:
```rust
pub fn handle_graph_search(id: &Value, params: &Value, state: &mut McpState) {
    // Circuit breaker guard
    if state.cbm_status.summary() != "available" {
        send_response(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32603, "message": format!("CBM unavailable: {}", state.cbm_status.summary()) }
        }));
        return;
    }
    // ... existing logic
}
```
Apply identical guard to: `handle_graph_query`, `handle_graph_trace`, `handle_get_architecture`, `handle_get_cbm_status` (status is always available though), `handle_cbm_proxy`.

**Regression Test — H-04: Circuit breaker guard in handlers**
- **File:** `src/tests/cbm/regression.rs`
- **Test 1:** `cbm_handler_returns_error_when_degraded` — set `state.cbm_status = CbmStatus::Degraded(...)`, call `handle_graph_search`, assert JSON-RPC error with `-32603` and message containing "unavailable"
- **Test 2:** `cbm_handler_returns_error_when_unavailable` — set `state.cbm_status = CbmStatus::Unavailable`, call each CBM handler, assert all return error
- **Test 3:** `cbm_handler_proceeds_when_available` — set `state.cbm_status = CbmStatus::Available`, mocked bridge, call handler, assert handler logic executes (no early return)
- **Test 4:** `handle_get_cbm_status_always_returns_status` — even when degraded, `handle_get_cbm_status` should return the status string (not the circuit breaker guard error, since this handler's job IS to report status)

---

### H-05: Add structured error types for cross-system failures
**Effort:** 2-3 hours | **Risk:** MEDIUM (touches many files)

**Problem:** Errors propagate as `Box<dyn Error>` or `String`. No way to distinguish transient (retryable) from permanent failures.

**Fix:**
1. Create `src/error.rs`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum CleanCtxError {
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
       #[error("CBM error: {0}")]
       Cbm(String),
       #[error("Compression error: {0}")]
       Compression(String),
       #[error("IR error: {0}")]
       Ir(String),
       #[error("Persistence error: {0}")]
       Persistence(String),
       #[error("Config error: {0}")]
       Config(String),
   }
   impl CleanCtxError {
       pub fn is_retryable(&self) -> bool { ... }
       pub fn status_code(&self) -> i64 { ... }
   }
   ```

2. Thread `CleanCtxError` through the handler chain — change all `Box<dyn Error>` return types to `CleanCtxError`.

3. In `send_response`, use `status_code()` to map to JSON-RPC error codes.

**Note:** This is a medium-effort refactor. Deferrable to Phase 3 if schedule is tight.

**Unit Test — H-05: Error type classification**
- **File:** `src/tests/mcp/error.rs` (new)
- **Test 1:** `io_errors_are_retryable` — `CleanCtxError::Io(std::io::Error::from(std::io::ErrorKind::ConnectionReset)).is_retryable()` == true
- **Test 2:** `cbm_errors_are_retryable` — `CleanCtxError::Cbm("ConnectionLost".into()).is_retryable()` == true
- **Test 3:** `config_errors_are_not_retryable` — `CleanCtxError::Config("missing field".into()).is_retryable()` == false
- **Test 4:** `compression_errors_are_not_retryable` — `CleanCtxError::Compression("parse error".into()).is_retryable()` == false
- **Test 5:** `status_code_mapping` — each variant maps to correct JSON-RPC error code: Io→-32603, Cbm→-32603, Compression→-32603, Config→-32602, Persistence→-32603, Ir→-32603

---

### H-06: Document MCP vs Proxy cache separation
**Effort:** 30 min | **Risk:** NONE

**Fix:** 
1. In `docs/PROXY.md`, add a note under "Configuration" clarifying that the proxy's cache system is separate from the MCP server's `CacheConfig`.
2. In `docs/ARCHITECTURE_OVERVIEW.md`, add a note about the two cache systems.

**No automated test — documentation-only fix.**
**Manual verification:** Review both docs for the added clarification text.

---

## Phase 3 — Medium Findings (Fix in v0.2.0 or v0.3.0)

### M-01: Remove `helpers.rs` shim (A-01)
**Effort:** 1 hour | **Risk:** LOW

Verify no callers still import from `crate::helpers::`. If all have migrated to `crate::compaction::`, delete `src/helpers.rs` and its `pub` declaration in `lib.rs`. If some remain, migrate them first.

**Build Test — M-01: No stale shim imports**
- **Type:** Compile-time assertion
- **Step 1:** Run `grep -r "crate::helpers::" src/` — should return zero matches
- **Step 2:** Verify `src/helpers.rs` is deleted
- **Step 3:** Verify `pub mod helpers;` removed from `src/lib.rs`
- **Step 4:** `cargo build --all-targets` succeeds

---

### M-02: Convert `eprintln!` to structured logging
**Effort:** 2-3 hours | **Risk:** LOW

Add the `tracing` crate (already in roadmap A-04) and convert all `eprintln!("[clean-ctx] ...")` calls to `tracing::info!` / `tracing::warn!` / `tracing::error!`. Add a `--quiet` flag to suppress non-error output for air-gapped environments.

**Regression Test — M-02: Structured logging**
- **File:** `src/tests/mcp/regression.rs`
- **Test 1:** `no_clean_ctx_eprintln_in_src` — grep `src/` for `eprintln!("[clean-ctx]"`, verify 0 hits (all migrated to `tracing`)
- **Test 2:** `tracing_crate_is_in_dependencies` — verify `tracing` exists in `Cargo.toml` dependencies
- **Test 3 (integration):** `quiet_flag_suppresses_info_output` — run with `--quiet`, verify only error-level messages appear on stderr

---

### M-03: Fix stale comment in config.rs
**Effort:** 1 min | **Risk:** NONE

In `src/config.rs` line 200, change:
```rust
/// Persistence configuration (placeholder for future SQLite layer).
```
To:
```rust
/// Persistence configuration for SQLite-backed cross-session storage.
```

**Unit Test — M-03: No stale doc comments**
- **File:** `src/tests/config.rs`
- **Test name:** `persistence_config_doc_comment_is_accurate`
- **Type:** Compile-time doc test or string content check
- **Assertion:** Source of `PersistenceConfig` does NOT contain the word "placeholder"

---

### M-04: Document path alias sharing between workspace and single-file modes
**Effort:** 15 min | **Risk:** NONE

Add a note in `docs/ARCHITECTURE_OVERVIEW.md` under "Why a shared cache?" explaining that path aliases are global across the session and that `compress_workspace` populates aliases visible to subsequent `provide_code_context` calls.

**Integration Test — M-04: Path alias sharing**
- **File:** `src/tests/mcp/regression.rs`
- **Test name:** `workspace_aliases_visible_in_single_file_mode`
- **Setup:** Create McpState, call `compress_workspace` on temp dir with `src/services/user.ts`
- **Then:** Call `provide_code_context` for `src/services/user.ts`
- **Assert:** The `§PATHMAP` footer in response contains the same alias (`α1`) that workspace mode assigned

---

### M-05: Log warning when CBM enrichment is silently dropped
**Effort:** 15 min | **Risk:** NONE

In `enrich_with_cbm()` (tool_handlers.rs:1720), add:
```rust
} else {
    eprintln!("[clean-ctx] WARNING: CBM enrichment compression failed for {}", file_path);
}
```

**Regression Test — M-05: Enrichment drop warning**
- **File:** `src/tests/cbm/regression.rs`
- **Test name:** `enrichment_drop_logs_warning`
- **Setup:** McpState with mocked bridge that returns uncompilable data (force `compress_cbm_response` to return None)
- **Call:** `enrich_with_cbm()`
- **Assert:** stderr captured output contains "WARNING: CBM enrichment compression failed"
- **Note:** Requires stderr capture in test harness

---

### M-06: Add `criterion` benchmarks
**Effort:** 1 day | **Risk:** LOW

Add a `benches/` directory with criterion benchmarks for:
- Compression throughput (lines/sec) at each fidelity
- Delta computation time vs full recompression
- Workspace compression at various directory sizes
This provides regression protection for compression performance.

**Benchmark Regression Test — M-06: Performance guard**
- **File:** `benches/compression_benchmarks.rs` (new)
- **Bench 1:** `low_fidelity_throughput` — compress `LargeService.ts` (438 lines) at Low fidelity, record lines/sec
- **Bench 2:** `medium_fidelity_throughput` — same file at Medium fidelity
- **Bench 3:** `high_fidelity_throughput` — same file at High fidelity
- **Bench 4:** `delta_vs_full_recompression` — 50-edit simulation, compare total time
- **CI integration:** Run `cargo bench --no-run` to ensure benches compile

---

## Phase 4 — Low Findings (Fix when convenient)

### L-01: Clamp `days` to >= 1 in `handle_purge_old_deltas`
**Effort:** 2 min | **Risk:** NONE

```rust
let days = params["arguments"]["days"].as_i64().unwrap_or(30).max(1);
```

**Unit Test — L-01: Negative days clamped**
- **File:** `src/tests/mcp/tool_handlers.rs`
- **Test name:** `purge_old_deltas_clamps_negative_days`
- **Setup:** Call handler with `days = -5`
- **Assert:** Internal `days` value is clamped to `1`
- **Setup:** Call handler with `days = 0`
- **Assert:** Internal `days` value is clamped to `1`

### L-02: Log bytes drained in oversize request recovery
**Effort:** 5 min | **Risk:** NONE

In `drain_line()` in `src/mcp/server.rs`, add a counter and log:
```rust
eprintln!("[clean-ctx] WARNING: Drained {} oversize bytes from stdin", drained);
```

**Unit Test — L-02: Drain line logs bytes**
- **File:** `src/tests/mcp/server.rs`
- **Test name:** `drain_line_logs_byte_count_for_oversize_request`
- **Setup:** Mock stdin with 20MB of data followed by `\n`
- **Call:** `read_request_line()` → expect `OversizeRequest`
- **Assert:** `drain_line()` was called and stderr contains byte count

### L-03: Document test file convention
**Effort:** 10 min | **Risk:** NONE

Either update `.clinerules` to acknowledge the Rust `#[path]` convention as compliant, or add a note to `CONTRIBUTING.md` explaining the test file layout.

**CI Test — L-03: Test file convention compliance**
- **Type:** CI lint step
- **Check:** Run `find src/ -name "*.rs" | xargs grep -l '#\[cfg(test)\]'` — verify all test modules referenced via `#[path]` exist at the referenced path
- **Check:** `.clinerules` updated to document the convention

### L-04: Document canonical path behavior in read_source
**Effort:** 10 min | **Risk:** NONE

Add a doc comment on `McpState::read_source()` explaining that canonicalization means symlink-equivalent paths share cache entries.

**Unit Test — L-04: Symlink-equivalent paths share cache**
- **File:** `src/tests/mcp/state.rs`
- **Test name:** `symlink_equivalent_paths_share_source_cache` (skip on Windows if not supported)
- **Setup:** Create temp file with known content, create symlink pointing to it
- **Call:** `read_source(temp_file_path)` then `read_source(symlink_path)`
- **Assert:** Both calls return `Arc` pointers to the same allocation (`Arc::ptr_eq`)
- **Assert:** `source_cache.len()` == 1 (not 2)

---

## Test Coverage Summary

| Finding | Test Type | File | Test Count |
|---------|-----------|------|:---:|
| B-03 | Build/CI | grep + CI check | 2 |
| B-01 | Integration | `src/tests/cbm/integration.rs` | 4 |
| B-02 | Integration + E2E | `src/tests/cbm/integration.rs` + `e2e.rs` | 3 |
| H-01 | Unit | `src/tests/mcp/regression.rs` | 1 |
| H-02 | Build/CI | CI pipeline | 1 |
| H-03 | Integration + E2E | `src/tests/cbm/integration.rs` + `e2e.rs` | 4 |
| H-04 | Regression | `src/tests/cbm/regression.rs` | 4 |
| H-05 | Unit | `src/tests/mcp/error.rs` (new) | 5 |
| H-06 | Manual | Documentation review | — |
| M-01 | Build | Compile + grep | 1 |
| M-02 | Regression | `src/tests/mcp/regression.rs` | 3 |
| M-03 | Unit | `src/tests/config.rs` | 1 |
| M-04 | Integration | `src/tests/mcp/regression.rs` | 1 |
| M-05 | Regression | `src/tests/cbm/regression.rs` | 1 |
| M-06 | Benchmark | `benches/` (new) | 4 |
| L-01 | Unit | `src/tests/mcp/tool_handlers.rs` | 1 |
| L-02 | Unit | `src/tests/mcp/server.rs` | 1 |
| L-03 | CI Lint | CI pipeline | 1 |
| L-04 | Unit | `src/tests/mcp/state.rs` | 1 |
| **Total** | | | **~34 new tests** |

---

## Verification Checklist (Post-Fix)

After all phases are complete, run:

```bash
# Build
cargo build --release
cargo build --all-targets

# Lint
cargo clippy --all-targets -- -D warnings

# Test
cargo test
cargo test -p clean-ctx-proxy

# Audit
cargo audit
cargo deny check

# Verify zero issues
```

## Sign-off Criteria

- [ ] 3 Blockers resolved with regression tests
- [ ] 6 High findings resolved with regression/integration tests
- [ ] 6 Medium findings resolved or deferred with dates
- [ ] 4 Low findings resolved or deferred with dates
- [ ] All tests pass (1,035+ existing + ~34 new)
- [ ] 0 clippy warnings
- [ ] `cargo audit` clean
- [ ] SECURITY.md updated
- [ ] Version bumped to 0.2.0-rc1