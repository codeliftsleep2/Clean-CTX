# Phase 1: Module Split Refactoring Plan

## Overview

This plan targets the largest source file (`src/mcp/tools.rs`, 1,891 lines) and secondary candidates for splitting to improve maintainability, reduce cognitive load, and enforce single-responsibility boundaries.

## Current State (File Size Heatmap)

| File | Lines | Priority |
|------|-------|----------|
| `src/mcp/tools.rs` | 1,891 | **P0 — Critical** |
| `src/mcp/workspace.rs` | 803 | P1 — High |
| `src/ir/compiler.rs` | 735 | P2 — Medium |
| `src/angular_meta/decorators.rs` | 1,040 | P2 — Medium (cohesive domain) |
| `src/compression/pipeline.rs` | 536 | P3 — Low |

---

## Phase 0: Fix `context_stats` Dashboard (Always Shows 0)

### Root Cause Analysis

I traced the full data flow from `provide_code_context` → `state.session_stats.record_compression()` → `context_stats` and found **two** independent causes:

#### Cause 1: `handle_compress_code_context` doesn't record stats

The `compress_code_context` handler (lines 447–552) is called by all Clean-CTX MCP tools (`compress_code_context`, `delta_code_context`, `diff_code_context`). None of them call `state.session_stats.record_compression()`. Only `handle_provide_code_context` records stats (in its `FullCompress` and `DeltaTransport` branches).

**Fix**: Add `session_stats.record_compression()` to:
- `handle_compress_code_context` (line 542)
- `handle_delta_code_context` (line 725)
- `handle_delta_text_context` (line 822)

#### Cause 2: Stats are purely in-memory, lost on server restart

`SessionStats` lives in `McpState.session_stats` — an in-memory `HashMap`. Every time the MCP server binary is rebuilt and the IDE reconnects, a fresh `McpState` is created with `SessionStats::new()`. The `context_stats` tool then shows 0 because the accumulator was wiped.

The DB already exists (`SqliteStore` with `rebuild_stats()` at `src/mcp/sqlite_store.rs:192`) but:
- `PersistenceConfig::default()` has `enabled: false` — it's opt-in
- `McpState::new()` only opens the DB — it doesn't call `rebuild_stats()`

**Fix 2a**: Change `PersistenceConfig::default()` to `enabled: true` so persistence is always-on. Then fix `main.rs` `generate_default_config()` to reflect the same.

**Fix 2b**: Wire `rebuild_stats()` into `McpState::new()` so persisted stats load on server start.

### Implementation Plan

| # | Change | File | Lines |
|---|--------|------|-------|
| 1 | Add `session_stats.record_compression()` to `handle_compress_code_context` | `src/mcp/tools.rs` | ~8 |
| 2 | Add `session_stats.record_compression()` to `handle_delta_code_context` | `src/mcp/tools.rs` | ~8 |
| 3 | Add `session_stats.record_compression()` to `handle_delta_text_context` | `src/mcp/tools.rs` | ~8 |
| 4 | Change `PersistenceConfig::default()` `enabled` to `true` | `src/config.rs` | 1 |
| 5 | Update `generate_default_config()` same | `src/main.rs` | 1 |
| 6 | Call `rebuild_stats()` in `McpState::new()` when DB is available | `src/mcp/state.rs` | ~10 |

---

## Phase 1: Split `src/mcp/tools.rs`

### Problem

`tools.rs` bundles 4 distinct concerns:
1. **Tool definitions** (`tool_list()` — ~240 lines of JSON schemas)
2. **Dispatch** (`dispatch_tools_call()` — the router)
3. **All tool handlers** (14 `handle_*` functions: ~1,100 lines)
4. **Internal helpers** (`compress_text_body`, `compile_file_ir`, `resolve_file_path`, `estimate_tokens`, `diff_code_context_handler` — ~350 lines)

### Proposed Module Structure

```
src/mcp/
  ├── mod.rs                  ← unchanged (adds 2 new modules)
  ├── tools.rs                ← KEPT: tool_list(), parse_fidelity_arg(), resolve_fidelity(), dispatch_tools_call()
  ├── tool_handlers.rs        ← NEW: all 14 handle_* functions
  └── tool_helpers.rs         ← NEW: compress_text_body, compile_file_ir, resolve_file_path, estimate_tokens, diff_code_context_handler
```

### Detailed Breakdown

#### `tools.rs` (remaining: ~280 lines)
- `tool_list()` — returns tool definitions
- `parse_fidelity_arg(id, params)` — fidelity arg parser
- `resolve_fidelity(explicit, ext, config)` — fidelity resolver
- `dispatch_tools_call(id, tool_name, params, state)` — the router/match
- `#[path = "../tests/mcp/tools.rs"] mod tests;`

#### `tool_handlers.rs` (new: ~1,100 lines)
All 14 handler functions (each receives `id`, `params`, `state`):
| Handler | Lines | Category |
|---------|-------|----------|
| `handle_compress_code_context` | ~100 | Compress |
| `handle_diff_code_context` | ~50 | Compress |
| `handle_delta_code_context` | ~120 | Compress |
| `handle_delta_text_context` | ~95 | Compress |
| `handle_apply_delta` | ~80 | Compress |
| `handle_provide_code_context` | ~240 | Context |
| `handle_restore_context` | ~100 | Context |
| `handle_context_history` | ~75 | Context |
| `handle_context_stats` | ~80 | Context |
| `handle_save_context` | ~85 | Persistence |
| `handle_list_sessions` | ~35 | Persistence |
| `handle_replay_history` | ~60 | Persistence |
| `handle_purge_old_deltas` | ~40 | Persistence |

#### `tool_helpers.rs` (new: ~350 lines)
| Helper | Lines |
|--------|-------|
| `compress_text_body(file_path, fidelity, state)` | ~75 |
| `compile_file_ir(file_path, fidelity, state)` | ~85 |
| `resolve_file_path(path, workspace_root)` | ~20 |
| `estimate_tokens(text)` | ~5 |
| `diff_code_context_handler(file, cache, fidelity)` | ~70 |

### Migration Steps

1. **Create `tool_handlers.rs`** with all 14 `handle_*` functions. These are `fn` (not `pub(crate)`) — they're only called from `dispatch_tools_call`.
2. **Create `tool_helpers.rs`** with `compress_text_body`, `compile_file_ir`, `resolve_file_path`, `estimate_tokens`, `diff_code_context_handler`. These need `pub(crate)` visibility.
3. **Add modules** to `mcp/mod.rs`:
   ```rust
   mod tool_handlers;
   mod tool_helpers;
   ```
4. **Update `tools.rs`**:
   - Remove all handler and helper function bodies
   - Add `use super::tool_handlers::*;` and `use super::tool_helpers::*;`
   - Keep `tool_list()`, `parse_fidelity_arg()`, `resolve_fidelity()`, `dispatch_tools_call()`

### Risk Assessment

- **Low risk** — pure mechanical extraction with no behavioral changes
- **No dependency cycle risk** — handlers call helpers, not vice versa
- **No test changes needed** — the `#[path]` test module stays in `tools.rs`

---

## Phase 2: Extract `compress_text_body` to Compression Module

### Problem
`compress_text_body` lives in `tool_helpers.rs` (or currently `tools.rs`) but implements a compression pipeline. It imports heavily from `compression::pipeline`, `compression::capture_pipeline`, and `compaction::*`. This creates an unnatural dependency: the MCP layer directly orchestrates compression internals.

### Solution
Move `compress_text_body` to `src/compression/pipeline.rs` as a `pub(crate)` function.

### Migration Steps
1. Add `compress_body_with_delta` to `src/compression/pipeline.rs`
2. Call it from the handler instead of the local `compress_text_body`
3. Remove the local copy from MCP helpers

---

## Phase 3: Split `src/mcp/workspace.rs` (803 lines)

### Problem
`workspace.rs` bundles:
- Workspace compression (`compress_workspace_dir`, `compress_pass`, `compress_pass_with_global_symbols`)
- Source file collection (`collect_source_files`, `collect_source_files_inner`)
- Manifest formatting (`format_manifest_header`, `format_manifest_footer`, `triplet_name`)
- Class block extraction (`extract_class_blocks`, `find_next_class_keyword`, `find_decorator_start`)
- Graph/bundle passes (`graph_pass`, `bundle_pass`)

### Proposed Split
```
src/mcp/workspace/
  ├── mod.rs                  ← re-exports; compress_workspace_dir, WorkspaceResult, PassContext
  ├── compress.rs             ← compress_pass, compress_pass_with_global_symbols
  ├── files.rs                ← collect_source_files, collect_source_files_inner
  ├── manifest.rs             ← format_manifest_header, format_manifest_footer, triplet_name
  └── classes.rs              ← extract_class_blocks, find_next_class_keyword, find_decorator_start
```

Alternatively, a lighter split into 2 files:
```
src/mcp/
  ├── workspace.rs            ← compress_workspace_dir, compress_pass, compress_pass_with_global_symbols, bundle_pass, graph_pass
  └── workspace_util.rs       ← collect_source_files, extract_class_blocks, manifest formatting helpers
```

---

## Phase 4: Empty Test Stub Cleanup

### Problem
Many `src/tests/*.rs` files exist but are empty (0 bytes / 0 lines). This is test scaffolding dead weight.

### Files to Delete or Populate
```
src/tests/compaction/class.rs, field.rs, method.rs, modifiers.rs (4 empty)
src/tests/compression/pipeline.rs, opcodes.rs, micro_opcodes.rs, fidelity.rs,
           capture_pipeline.rs, markers.rs, language.rs, scope_defaults.rs,
           streaming.rs, symbol_compression.rs, workspace_symbols.rs,
           text_delta.rs, report.rs (13 empty)
src/tests/angular_meta/*.rs (8 empty files)
src/tests/decompression/decompressor.rs (empty)
src/tests/dictionary/*.rs (4 empty files)
```

### Action
1. Audit each: either populate with real test content, or delete the file
2. Update `mod.rs` in each test subdirectory to remove deleted modules

---

## Phase 5: `src/ir/compiler.rs` Split (735 lines)

### Problem
`IRCompiler` is the heart of the IR pipeline. While it's well-structured, at 735 lines and growing, the `emit_*` methods and `MethodSig` parsing could be extracted.

### Proposed Split (lower priority)
```
src/ir/
  ├── compiler.rs             ← KEPT: IRCompiler struct, compile(), flush_method_flags()
  └── compiler_methods.rs     ← NEW: MethodSig, parse_method_sig(), emit_method_ir(), emit_import_ir()
```

---

## Execution Order

| Phase | Change | Effort | Risk | Files Changed |
|-------|--------|--------|------|---------------|
| **0** | Fix context_stats (stats recording + persistence) | Small | Low | 4 |
| **1** | Split tools.rs | Medium | Low | 4 (mod.rs, tools.rs, +2 new) |
| **2** | Extract compress_text_body | Small | Low | 3 (pipeline.rs, mod.rs, tool_helpers.rs) |
| **3** | Split workspace.rs | Medium | Low | 2-5 |
| **4** | Clean empty test stubs | Small | Low | 20-30 |
| **5** | Split ir/compiler.rs | Medium | Low | 3 |

---

## Success Criteria

1. No file in `src/mcp/` exceeds 400 lines
2. No behavioral changes — all existing tests pass
3. `context_stats` shows real data after one compression call
4. Stats survive server restart (loaded from SQLite)
5. `compress_text_body` lives in `compression/` module
6. Empty test stubs are either populated or removed
7. `cargo clippy` produces no new warnings
8. `cargo test` passes with same count as before (891)