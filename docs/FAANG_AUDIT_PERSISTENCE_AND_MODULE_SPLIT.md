# FAANG Audit: Buffered Persistence + Module Split Changes

**Date:** 2026-06-12  
**Scope:** `src/mcp/buffered_store.rs`, `src/mcp/sqlite_store.rs`, `src/mcp/state.rs`, `src/mcp/tool_handlers.rs`, `src/mcp/server.rs`, `src/mcp/workspace.rs`, `src/mcp/workspace_util.rs`, `src/mcp/tools.rs`, `src/mcp/tool_helpers.rs`, `src/mcp/context_store.rs`, `src/mcp/mod.rs`, `src/ir/compiler.rs`, `src/ir/compiler_methods.rs`, `src/ir/mod.rs`  
**Test Status:** 940 passing, 0 warnings  
**Build Status:** Clean build

---

## Executive Summary

The implementation is functionally sound for a Phase 1 persistence layer and module split. The codebase compiles cleanly with 940 tests passing. However, there are several implementation gaps, dead code paths, and correctness issues that range from critical (replay handler is a no-op) to minor (hardcoded stats on restart). This document catalogues all findings.

---

## Critical Findings (P0 — Broken Functionality)

### CRIT-01: `handle_replay_history` is a no-op
**File:** `src/mcp/tool_handlers.rs:1229-1231`  
**Severity:** Critical  
**Impact:** The `replay_history` tool claims to replay state but never updates `state.ir_context`.

```rust
let mut state_ir = state.ir_context.clone();
state_ir.load_ir(ir.clone());
// Note: Can't assign directly due to borrow, but we can respond with the data
```

The handler clones `ir_context`, loads the replayed IR into the clone, then drops the clone. The response claims success, but the in-memory state is unchanged. Any subsequent `delta_code_context` call will use the old baseline, not the replayed one.

**Fix:** Remove the clone — assign directly:
```rust
state.ir_context.load_ir(ir.clone());
```
This works because `state` is `&mut McpState`, so `state.ir_context` is the real target.

---

### CRIT-02: `handle_save_context` hash is version-dependent — overwrites history
**File:** `src/mcp/tool_handlers.rs:1112,1137`  
**Severity:** Critical  
**Impact:** The context ID is derived from `sha256("{fp}:{version}")`, which changes with every version. Combined with `INSERT OR REPLACE` in `sqlite_store.rs:282`, this means each save overwrites the previous one, destroying version history.

```rust
let hash = format!("{:x}", sha2Sha256::digest(format!("{}:{}", fp, version).as_bytes()));
```

When version=1 is saved, the ID is `ctx-sha256("file.ts:1")`. When version=2 is saved, the ID is `ctx-sha256("file.ts:2")` — a completely different row. The old row is NOT overwritten; instead, a new row is created but the `file_path` column has the same value, so `load_latest` picks the newest by `updated_at`. However, the old deltas (linked to the old context ID) are now orphaned.

**Fix:** Use a content-hash-based ID (like the automatic persistence does) rather than version-dependent:
```rust
let hash = format!("{:x}", sha2::Sha256::digest(fp.as_bytes()));
```

---

## High Priority Findings (P1 — Data Integrity / Correctness)

### HIGH-01: `load_latest` returns hardcoded `version: 0`
**File:** `src/mcp/sqlite_store.rs:319`  
**Severity:** High  
**Impact:** `StoredContextMeta.version` is always 0, making it useless for consumers. The `context_history` handler displays this value.

```rust
Ok(Some(StoredContextMeta {
    file_path: fp,
    fidelity,
    version: 0, // ← should query COUNT(deltas) + 1
    is_angular: false,
    source_hash: hash,
    created_at: std::time::SystemTime::now(),
}))
```

**Fix:** Compute version as `delta_count(context_id) + 1` or add a `version` column to the schema.

---

### HIGH-02: `load_latest` returns current time for `created_at`
**File:** `src/mcp/sqlite_store.rs:322`  
**Severity:** High  
**Impact:** The `created_at` field always reflects the time of the `load_latest` call, not when the context was actually created. Consumers that compare timestamps will get incorrect results.

```rust
created_at: std::time::SystemTime::now(), // ← should read from DB
```

**Fix:** Read the `created_at` column from the SQL query (it's already in the schema as `created_at TEXT`).

---

### HIGH-03: `rebuild_stats` hardcodes token counts
**File:** `src/mcp/sqlite_store.rs:213`  
**Severity:** High  
**Impact:** After a server restart, the dashboard shows fake data (100 raw / 30 compressed tokens for every file).

```rust
stats.record_compression(
    &path, 100, 30, "low", false, strategy
);
```

**Fix:** Store `raw_tokens` and `compressed_tokens` in the `contexts` table and read them back during rebuild. Alternatively, accept degraded stats as a known limitation and document it.

---

### HIGH-04: `context_store::clear_file` orphan leak
**File:** `src/mcp/context_store.rs:214-231`  
**Severity:** High  
**Impact:** The `clear_file` implementation only removes entries where `id_to_path[id] == file_path`. But `generate_id` creates IDs like `ctx-/path/to/file-0000000000000001` (timestamp-based), while `save_context` uses the ID returned by `generate_id` and maps it via `id_to_path`. The reverse lookup works for the ID → path mapping, but deltas are keyed by the same ID, so they ARE cleaned up. However, the `let _ = meta;` on line 229 is dead code that suggests the implementer gave up on proper cleanup.

**Actual impact:** The cleanup works for the current usage pattern because `id_to_path` correctly maps back. But the code is confusing and the `let _ = meta` is misleading.

---

## Medium Priority Findings (P2 — Design Gaps / Missing Functionality)

### MED-01: `queue_clear_file` doesn't auto-flush
**File:** `src/mcp/buffered_store.rs:370-376`  
**Severity:** Medium  
**Impact:** `queue_save_context` and `queue_append_delta` auto-flush at `BATCH_THRESHOLD`, but `queue_clear_file` does not. If a user calls `restore_context` (which calls `clear_file`), the clear op sits in the buffer until the next auto-flush. This means a subsequent `load_latest` may not see the clear.

---

### MED-02: `diff_code_context` handler bypasses `source_cache`
**File:** `src/mcp/tool_helpers.rs:186`  
**Severity:** Medium  
**Impact:** The diff handler reads from disk via `std::fs::read_to_string(&file)` instead of using `state.read_source()`. This means files read by other handlers are cached, but diff reads are not. In a multi-tool workflow, this causes redundant I/O.

---

### MED-03: `save_context` handler doesn't persist `compressed_output`
**File:** `src/mcp/tool_handlers.rs:1114`  
**Severity:** Medium  
**Impact:** The explicit `save_context` handler passes `""` for `compressed_output`:
```rust
store.save_context(fp, Fidelity::Low, "", Some(&ir_binary), &hash)
```
This means the `pretty_text` column is always empty for manually-saved contexts. The automatic persistence in `handle_compress` does save the compressed text, so this is only a gap for explicit saves.

---

### MED-04: `compress_pass_with_global_symbols` doesn't store delta baselines
**File:** `src/mcp/workspace.rs:222-338`  
**Severity:** Medium  
**Impact:** When `compress_workspace_dir` uses the global symbols path (Low fidelity), text delta baselines are never stored. Subsequent `delta_text_context` calls on workspace-compressed files will always return full output instead of deltas.

This is likely intentional (workspace compression is a batch operation), but it's undocumented.

---

### MED-05: `workspace_util::format_manifest_footer` dead loop
**File:** `src/mcp/workspace_util.rs:56-68`  
**Severity:** Medium (code quality)  
**Impact:** The loop iterates over Angular-adjacent files but does nothing because `state` is immutably borrowed. The comment acknowledges this but the dead code is confusing.

---

## Low Priority Findings (P3 — Code Quality / Minor Issues)

### LOW-01: Variable shadowing in persistence hook
**File:** `src/mcp/tool_handlers.rs:148`  
**Severity:** Low  
**Impact:** `Ok(id)` shadows the function parameter `id` (JSON-RPC request ID). In this context it doesn't cause incorrect behavior, but it's confusing and lint-unfriendly.

---

### LOW-02: `InMemoryContextStore` not cleared on `restore_context`
**File:** `src/mcp/tool_handlers.rs:894`  
**Severity:** Low  
**Impact:** `restore_context` clears the SQLite-backed `persistence_store` and the text delta baselines, but does NOT clear `state.context_store` (the in-memory store). This means stale metadata may persist.

---

### LOW-03: `ContextStore` trait has `#[allow(dead_code)]` on active methods
**File:** `src/mcp/context_store.rs:24,68`  
**Severity:** Low (code quality)  
**Impact:** The `#[allow(dead_code)]` annotations suppress warnings for methods that ARE used by `SqliteStore` but not by `InMemoryContextStore`. These should be removed now that the SQLite implementation exists.

---

### LOW-04: `format_manifest_header` takes `&McpState` but only uses `config`
**File:** `src/mcp/workspace_util.rs:29-32`  
**Severity:** Low (API design)  
**Impact:** The function signature is broader than necessary. Taking `&CleanCtxConfig` directly would be more precise.

---

### LOW-05: `SqliteStore` comment claims single-threaded but is wrapped in `Arc<Mutex<>>`
**File:** `src/mcp/sqlite_store.rs:15`  
**Severity:** Low (documentation)  
**Impact:** The comment "No Mutex needed — MCP server is single-threaded (stdin/stdout loop)" is outdated. The `BufferedStore` now wraps `SqliteStore` in `Arc<Mutex<>>`, which is correct for the retry/fallback pattern but contradicts the comment.

---

### LOW-06: `BufferedStore::flush` silently returns 0 on lock poison
**File:** `src/mcp/buffered_store.rs:91-93`  
**Severity:** Low  
**Impact:** If the `pending` mutex is poisoned (a thread panicked while holding it), `flush` silently returns 0. The ops are lost. In practice this shouldn't happen, but a log message would aid debugging.

---

## Summary Table

| ID | Severity | File | Description |
|---|---|---|---|
| CRIT-01 | Critical | tool_handlers.rs:1229 | `replay_history` is a no-op (clone never written back) |
| CRIT-02 | Critical | tool_handlers.rs:1112 | Version-dependent hash causes orphaned context rows |
| HIGH-01 | High | sqlite_store.rs:319 | `load_latest` returns hardcoded `version: 0` |
| HIGH-02 | High | sqlite_store.rs:322 | `load_latest` returns current time for `created_at` |
| HIGH-03 | High | sqlite_store.rs:213 | `rebuild_stats` hardcodes fake token counts |
| HIGH-04 | High | context_store.rs:214 | `clear_file` has confusing dead code (`let _ = meta`) |
| MED-01 | Medium | buffered_store.rs:370 | `queue_clear_file` missing auto-flush |
| MED-02 | Medium | tool_helpers.rs:186 | `diff_code_context` bypasses source cache |
| MED-03 | Medium | tool_handlers.rs:1114 | Explicit `save_context` doesn't persist compressed_output |
| MED-04 | Medium | workspace.rs:222 | Global symbols path doesn't store delta baselines |
| MED-05 | Medium | workspace_util.rs:56 | Dead loop in manifest footer |
| LOW-01 | Low | tool_handlers.rs:148 | Variable `id` shadowing in persistence hook |
| LOW-02 | Low | tool_handlers.rs:894 | InMemoryContextStore not cleared on restore |
| LOW-03 | Low | context_store.rs:24 | Unnecessary `#[allow(dead_code)]` on used methods |
| LOW-04 | Low | workspace_util.rs:29 | Function takes broader type than needed |
| LOW-05 | Low | sqlite_store.rs:15 | Outdated single-threaded comment |
| LOW-06 | Low | buffered_store.rs:91 | Silent loss of ops on lock poison |

---

## Recommendations

### Immediate (P0) — Fix before next release
1. **Fix CRIT-01:** Remove the clone in `handle_replay_history`, assign directly to `state.ir_context`.
2. **Fix CRIT-02:** Change the hash in `handle_save_context` to be content-based, not version-based.

### Short-term (P1) — Fix in next sprint
3. **Fix HIGH-01:** Read actual version from DB in `load_latest`.
4. **Fix HIGH-02:** Read actual `created_at` from DB in `load_latest`.
5. **Fix HIGH-03:** Store token counts in DB schema or document degraded stats.

### Medium-term (P2)
6. Add auto-flush to `queue_clear_file`.
7. Route `diff_code_context` through `source_cache`.
8. Persist `compressed_output` in explicit `save_context`.

### Documentation
9. Update `sqlite_store.rs:15` comment to reflect `Arc<Mutex<>>` usage.
10. Document that global-symbols workspace compression doesn't support delta baselines.