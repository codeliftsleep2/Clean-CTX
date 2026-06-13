# Post-Audit Fix Plan: context_stats / BufferedStore / DB Persistence

## Overview

This document tracks the fixes identified during the FAANG-level audit of the
context_stats, BufferedStore, and DB persistence subsystems. Each issue is
prioritized and assigned a phase for implementation.

---

## Phase 1: Critical Fixes (CRIT-1, CRIT-2, CRIT-3)

### CRIT-1: `SqliteStore::load_latest` doesn't read token counts from DB

**File:** `src/mcp/sqlite_store.rs`  
**Problem:** The v2 schema migration added `raw_tokens` and `compressed_tokens`
columns, and `rebuild_stats()` reads them correctly. But `load_latest()` hardcodes
them to `0`. Any code path using `load_latest` (e.g., `handle_context_history`)
gets wrong token data.

**Fix:** Add `raw_tokens` and `compressed_tokens` to the SELECT query in
`load_latest()` and populate the `StoredContextMeta` fields.

**Test:** Add a test that saves with token counts, loads via `load_latest`, and
verifies the counts are non-zero.

---

### CRIT-2: `handle_save_context` passes `0, 0` for token counts

**File:** `src/mcp/tool_handlers.rs`  
**Problem:** The explicit `save_context` tool handler always writes zero token
counts to DB. When `rebuild_stats()` reads these rows, it gets 0 tokens →
dashboard shows incomplete data for explicitly-saved files.

**Fix:** Look up token counts from `session_stats` before persisting. If the
file has been tracked in `session_stats`, use those counts. Otherwise, pass 0.

**Test:** Add a test that compresses a file, then calls `save_context`, and
verifies the DB has non-zero token counts.

---

### CRIT-3: `handle_delta_code_context` computes compressed tokens on raw source

**File:** `src/mcp/tool_handlers.rs`  
**Problem:** The handler counts tokens on the raw source text as "compressed"
tokens, making raw ≈ compressed → 0% savings shown. The compressed tokens
should count the delta wire output.

**Fix:** Count tokens on the delta wire output (or the full IR output) instead
of the raw source text.

**Test:** Add a test that verifies the delta handler records correct token counts.

---

## Phase 2: High Priority Fixes (HIGH-1, HIGH-2, HIGH-3)

### HIGH-1: Inconsistent tokenizer usage across handlers

**Files:** `src/mcp/tool_handlers.rs`  
**Problem:** Only `handle_compress_code_context` uses the pluggable tokenizer
(`count_tokens_with_tokenizer`). All other handlers use `estimate_tokens`
(chars/4). If the user configures a tokenizer, only `compress_code_context`
stats will be accurate.

**Fix:** Thread the pluggable tokenizer through all handlers. At minimum:
- `handle_provide_code_context`
- `handle_restore_context`
- `handle_diff_code_context`
- `handle_delta_text_context`
- `handle_delta_code_context`

**Test:** Add a test that verifies using a non-default tokenizer produces
different token counts.

---

### HIGH-2: `BufferedStore::queue_save_context` hardcodes tokens to 0

**File:** `src/mcp/buffered_store.rs`  
**Problem:** The public convenience method doesn't accept token params. While
not called by production handlers (they use the `ContextStore::save_context`
trait impl), it's a footgun for anyone using the queue API directly.

**Fix:** Add `raw_tokens` and `compressed_tokens` params to `queue_save_context`,
or deprecate it in favor of the trait method.

**Test:** Add a test that verifies `queue_save_context` preserves token counts.

---

### HIGH-3: `merge()` session-level counter over-counting

**File:** `src/mcp/session_stats.rs`  
**Problem:** When a file exists in both in-memory and DB, the file is skipped
during merge (in-memory wins), but the DB's session-level operation counts are
still added unconditionally. This means the dashboard's "Full Compressions" and
"Delta Operations" counts can be inflated.

**Fix:** Recalculate `full_compress_count` and `delta_count` from file entries
after merge, or subtract the skipped file's contribution from the DB counts.

**Test:** Add a test that verifies merge doesn't inflate operation counts.

---

## Phase 3: Medium Priority Fixes (MED-1, MED-2, MED-3)

### MED-1: `handle_save_context` uses file-path-only hash for context ID

**File:** `src/mcp/tool_handlers.rs`  
**Problem:** Re-saving the same file always overwrites the previous context
(INSERT OR REPLACE). Previous IR binary and compressed output are lost.

**Fix:** Document this as intentional (CRIT-02 fix) and ensure version history
is preserved via delta rows.

---

### MED-2: `BufferedStore::load_latest` flushes before reading (double lock)

**File:** `src/mcp/buffered_store.rs`  
**Problem:** Two sequential lock acquisitions on the same `Arc<Mutex<>>`.

**Fix:** Batch into a single lock scope.

---

### MED-3: `SessionStats::merge()` doesn't recalculate operation counts from files

**File:** `src/mcp/session_stats.rs`  
**Problem:** After merging file entries, totals for `total_raw_tokens` and
`total_compressed_tokens` are recalculated from file data, but
`full_compress_count` and `delta_count` are not.

**Fix:** Recalculate operation counts from file entries after merge.

---

## Phase 4: Low Priority Fixes (LOW-1, LOW-2, LOW-3) ✅ COMPLETE

### LOW-1: `record_file_stats` helper is unused ✅

**File:** `src/mcp/tool_helpers.rs`  
**Fix:** Removed the unused `record_file_stats` helper (was marked `#[allow(dead_code)]`). All handlers already call `state.session_stats.record_compression()` directly with resolved paths.

---

### LOW-2: `test_sqlite_rebuild_stats` doesn't verify exact token values ✅

**File:** `src/tests/mcp/sqlite_store.rs`  
**Fix:** Added exact value assertions for both files (raw_tokens=500/1000, compressed_tokens=100/200).

---

### LOW-3: Fallback file timestamp collision ✅

**File:** `src/mcp/buffered_store.rs`  
**Fix:** Documented as non-issue — the index prefix (`op_{i}_{ts}.json`) uniquely identifies each operation even if multiple ops share the same nanosecond timestamp.

---

## Phase 5: Test Coverage Gaps ✅ COMPLETE

All gaps have been addressed with 16 regression tests in `src/tests/mcp/regression.rs`:

| Gap | Status | Test |
|-----|--------|------|
| Handler-level stats tests | ✅ | `regression_high1_*`, `regression_crit3_*` |
| `load_latest` token count test | ✅ | `regression_crit2_*`, `regression_med2_*` |
| Merge over-counting test | ✅ | `regression_high3_*` |
| Pluggable tokenizer stats test | ✅ | `regression_high1_*` |
| `queue_save_context` token loss | ✅ | `regression_high2_*` |
| MED-1 overwrite behavior | ✅ | `regression_med1_*` |
| MED-2 single lock scope | ✅ | `regression_med2_*` |
| LOW-2 exact token values | ✅ | `regression_low2_*` |
| E2E compress→flush→DB | ✅ | `regression_e2e_*` |

---

## Implementation Order

1. **Phase 1** (Critical) — Fix CRIT-1, CRIT-2, CRIT-3 ✅
2. **Phase 2** (High) — Fix HIGH-1, HIGH-2, HIGH-3 ✅
3. **Phase 3** (Medium) — Fix MED-1, MED-2, MED-3 ✅
4. **Phase 4** (Low) — Fix LOW-1, LOW-2, LOW-3 ✅
5. **Phase 5** (Tests) — Add missing test coverage ✅

**Final status: 991 tests passing, 0 failures, 0 compiler warnings.**
