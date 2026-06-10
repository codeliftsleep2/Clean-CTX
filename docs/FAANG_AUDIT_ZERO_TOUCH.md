# FAANG Audit — Zero-Touch Workflow

**Date**: 2026-06-10
**Scope**: New zero-touch workflow implementation (provide_code_context, restore_context, context_history, context_stats, heuristics engine, ContextStore, SessionStats, dashboard prompt, clean-ctx init CLI)

## ✅ PASS: Full Test Suite
- 785/785 tests pass
- `cargo clippy --all-targets` — 0 warnings

## ✅ PASS: Backward Compatibility
- All 7 existing tool definitions unchanged
- All dispatch arms still match correctly
- Config deserialization backward compatible (all new fields `#[serde(default)]`)

## ✅ PASS: Prompt Registration
- `prompts::prompt_list()` returns both `"cleanctx-notation"` and `"dashboard"`
- `router.rs` properly routes `"prompts/get"` with params extraction
- Both prompts return valid JSON-RPC responses

## ✅ PASS: CLI Argument Handling
- `clean-ctx` → MCP server
- `clean-ctx init` → creates `.clean-ctx/` dir + `.clean-ctx.json`
- No panics from any argument pattern
- `clean-ctx <unknown>` falls through to MCP server (acceptable)

## 🟡 ISSUES FOUND AND FIXED

### Issue 1: `force_high_fidelity` Extension Matching Bug (CRITICAL)

**File**: `src/mcp/heuristics.rs`, `resolve_fidelity()` function (lines 98-107)

**Problem**: `std::path::Path::extension()` returns only the final segment after the last dot. For `foo.service.ts`, it returns `"ts"`, not `"service.ts"`. The `force_high_fidelity` config defaults to `["*.service.ts", "*.component.ts", "*.guard.ts"]`. When the code constructs `format!("*.{}", extension)`, it produces `"*.ts"` and then checks `glob_match_simple("*.service.ts", "*.ts")` which returns `false`. The pattern `*.service.ts` never matches any file through this code path.

**Fix**: Changed to match against the full file name (via `file_stem` + full name) rather than just the extension.

### Issue 2: `restore_context` — Inconsistent Exclusion Check (MEDIUM)

**File**: `src/mcp/tools.rs`, `handle_restore_context()` (line 1111)

**Problem**: Checks `state.config.is_excluded(file_path_str)` against the raw (possibly relative) path, while `provide_code_context` first resolves the path via `resolve_file_path()` before checking exclusion. Relative paths could bypass the exclusion check or produce false matches.

**Fix**: Use `resolve_file_path()` before the exclusion check (though `restore_context` doesn't accept `workspaceRoot`, so relative resolution is simple CWD-join).

### Issue 3: SessionStats Strategy Counter Drift (LOW)

**File**: `src/mcp/session_stats.rs`, `record_compression()` (lines 116-119)

**Problem**: When the same file is recorded with a different strategy on a subsequent call, the old strategy counter is not decremented. E.g., first call with "full" → `full_compress_count=1, delta_count=0`; second call with "delta" → `full_compress_count=1, delta_count=1`. The total ops counter inflates.

**Fix**: Decrement the old strategy counter when the strategy changes.

### Issue 4: Dead Conditional in `resolve_fidelity` (LOW)

**File**: `src/mcp/heuristics.rs` (lines 87-94)

**Problem**: The `if f != Fidelity::Low || explicit_fidelity.is_some()` check is always true at this point (priority 1 would have caught an explicit fidelity), making the second `return f` dead code.

**Fix**: Simplify to `return f` directly.

---

## ✅ PASS: SQLite Persistence Layer (Phase 5 Addition)

**Date**: 2026-06-10
**Scope**: Cross-session persistence for compression contexts via SQLite

### Components Added

| File | Description |
|------|-------------|
| `src/mcp/sqlite_store.rs` | `SqliteStore` — full `ContextStore` trait impl backed by SQLite (WAL mode) |
| `src/tests/mcp/sqlite_store.rs` | 13 integration tests (all passing) |
| `src/mcp/mod.rs` | Lazy DB init from `CLEANCTX_PERSISTENCE_DB` env var |
| `src/mcp/state.rs` | `persistence_store: Option<SqliteStore>` on `McpState` |

### Schema (v1)

- **`contexts`** — baselines (content-hash PK, IR BLOB, fidelity, pretty text)
- **`deltas`** — sequential delta payloads (FK → contexts, auto-increment edit_sequence)
- **`symbols`** — symbol table entries (FK → contexts, phi markers)
- **`sessions`** — workspace session tracking
- **`_schema_version`** — migration version tracking

### New MCP Tools

| Tool | Description |
|------|-------------|
| `save_context` | Explicit manual checkpoint to DB |
| `list_sessions` | Show tracked sessions/files |
| `replay_history` | Replay deltas from DB up to target sequence |
| `purge_old_deltas` | Trim old delta history by age |

### Hot-Path Hooks

Persistence hooks fire automatically in:
- `provide_code_context` → `FullCompress` path (baseline save)
- `provide_code_context` → `DeltaTransport` path (baseline + delta save)
- `restore_context` → DB clear on file reset

### Test Coverage

```
test_sqlite_store_open_and_migrate ... ok
test_sqlite_save_and_load_round_trip ... ok
test_sqlite_save_with_ir_blob ... ok
test_sqlite_has_context ... ok
test_sqlite_clear_file ... ok
test_sqlite_delta_append_and_count ... ok
test_sqlite_deterministic_id_from_hash ... ok
test_sqlite_load_context_with_deltas ... ok
test_sqlite_load_nonexistent_returns_none ... ok
test_sqlite_purge_old_deltas ... ok
test_sqlite_delta_count_for_file ... ok
test_sqlite_rebuild_stats ... ok
test_sqlite_multiple_files_independent ... ok

test result: ok. 13 passed; 0 failed; 0 ignored
```

### Design Decisions

- **Non-fatal persistence**: All DB writes are fire-and-forget with `eprintln!` warnings — compression never fails due to DB issues.
- **Content-hash deterministic IDs**: `ctx-{sha256_hex}` ensures idempotent saves (same content → same ID → UPSERT).
- **No Mutex**: MCP server is single-threaded (stdin/stdout loop), so no concurrent access protection needed.
- **Lazy initialization**: DB only opens if `CLEANCTX_PERSISTENCE_DB` env var is set — zero overhead for users who don't need persistence.
- **`binary_wire::encode/decode`**: IR is serialized/deserialized as BLOBs; `file_id` and `version` are restored from DB columns on load (Gap 2 from plan).
