# Plan: Buffered Persistence Layer with Retry and Fallback

## Status: COMPLETE

## Problems Found

### 1. Server CWD != Project Root
The MCP server binary lives at `target/release/clean-ctx.exe` but Cline launches
it with a CWD that is NOT the project root. This causes:
- `CleanCtxConfig::load(current_dir())` fails to find `.clean-ctx.json`
- Falls back to `PersistenceConfig::default()` which has `enabled: false`
- `persistence_store` is always `None` -> no DB writes ever happen
- `debug_log` writes `.clean-ctx/debug.log` relative to wrong CWD

### 2. No Batch Writes
`flush()` calls `conn.save_context()` per-op, each as its own SQLite transaction.
Defeats the purpose of batching.

### 3. No Retry/Resilience
If a DB write fails, data is lost. No retries, no fallback.

## Architecture

### Tier 1: Batched Writes with Auto-Flush
- Buffer `SaveContext`, `AppendDelta`, `ClearFile` ops in memory
- Auto-flush when buffer reaches `BATCH_THRESHOLD` (5 items)
- Explicit flush on `context_stats` call and server shutdown
- Wrap all ops in `BEGIN TRANSACTION ... COMMIT`
- `PRAGMA wal_checkpoint(TRUNCATE)` after commit

### Tier 2: Retry with Exponential Backoff
- On flush failure: retry up to `MAX_RETRIES` (3) times
- Backoff: 0ms, 50ms, 200ms between retries
- Failed ops re-queued to `pending` for next flush attempt

### Tier 3: JSON File Fallback
- If all retries exhausted, write failed ops to `.clean-ctx/fallback/` as JSON files
- On next successful flush, check for fallback files, re-import into SQLite, delete files

### CWD Fix
- Add `find_project_root()` in `server.rs`: walk up from executable dir
  looking for `.clean-ctx.json` or `Cargo.toml`
- Use project root for: config loading, DB path resolution, debug log path

## Files to Modify

1. `src/mcp/server.rs` — add `find_project_root()`, use for config loading
2. `src/mcp/buffered_store.rs` — transaction batching, retry, auto-flush, fallback
3. `src/mcp/state.rs` — anchor DB path to project root
4. `src/mcp/tool_handlers.rs` — fix debug_log path, error surfacing via `_warnings`

## Testing
- All 920 existing tests must pass
- New tests for: retry logic, fallback file creation, auto-flush threshold,
  project root detection

## Three-Tier Defense Summary

| Tier | Mechanism | Handles |
|------|-----------|---------|
| 1 | Batched writes + SQLite transactions | Normal operation, performance |
| 2 | Retry with exponential backoff | Transient DB lock/failure |
| 3 | JSON file fallback | Total DB failure (corrupt, permissions) |