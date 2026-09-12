---

## [0.1.6] — 2026-06-10

### Added

#### Zero-Touch Workflow
- **Prompt Cache Optimization**: Anthropic API cache breakpoint injection via `_meta.cache_hints` in MCP responses.
  - `CacheConfig` struct with 7 fields (`enabled`, `system_prompt_ttl`, `tools_ttl`, `baseline_ttl`, `tail_ttl`, `vocab_version`, `tool_defs_version`) in `.clean-ctx.json`
  - Cache hints module (`src/mcp/cache_hints.rs`) with `CacheMetrics`, `CacheHints`, `CacheBreakpoint` types and `inject_cache_breakpoints()` function
  - Deduplication via `state.emitted_breakpoints` to avoid paying the 2.0× write multiplier
  - Four breakpoint regions: `system_prompt` (vocabulary), `tools` (tool definitions), `baseline` (persisted baselines), `tail` (dynamic content)
  - `clean-ctx-vocabulary` MCP prompt resource with the full opcode/marker vocabulary
  - Cache hints injected into `tools/list`, `prompts/list`, `prompts/get`, `provide_code_context`, `restore_context` responses
  - Cache status section in `context_stats` dashboard (both text and JSON)
  - Default config includes `"cache": { "enabled": true, ... }`
- **Workspace cache breakpoint**: `compress_workspace` now injects a baseline cache hint keyed on a SHA-256 hash of the manifest, so the entire workspace scan result is cacheable across sessions.
- **Cache metrics in context_history**: Per-file cache breakpoint status and session hit rate now shown in `context_history` output (both single-file and all-files modes).
- **Pluggable tokenizer for cache savings**: `inject_cache_breakpoints` now accepts an optional `&dyn Tokenizer` for accurate token savings estimates on cache hits. When a tokenizer is available, the full response JSON is tokenized; otherwise falls back to the rough `chars/4` heuristic.
- `src/mcp/heuristics.rs` — New heuristics engine that automatically selects optimal fidelity and compression strategy based on file characteristics, explicit intent, and existing baselines
- `src/mcp/session_stats.rs` — Session statistics tracking with per-file metrics (raw/compressed tokens, savings %, delta count, strategy, Angular detection) and dashboard rendering (text + JSON formats)
- `src/mcp/tools.rs` — Four new zero-touch workflow tools:
  - `provide_code_context` — Single entry point that orchestrates heuristics, compression/delta, Angular detection, and stats recording
  - `restore_context` — Force full re-compression, clearing all baselines and DB entries
  - `context_history` — View compression history and delta savings for tracked files
  - `context_stats` — Dashboard showing token savings, compression stats, and session metrics
- `src/mcp/prompts.rs` — New `dashboard` MCP prompt for system-level dashboard instructions

#### SQLite Persistence Layer
- `src/mcp/sqlite_store.rs` — Full `ContextStore` trait implementation backed by SQLite with WAL mode, schema versioning, and content-hash deterministic IDs
- `src/mcp/context_store.rs` — `ContextStore` trait abstraction with `InMemoryContextStore` (session-scoped) and `SqliteStore` (cross-session) implementations
- `src/mcp/mod.rs` — Lazy DB initialization from `CLEANCTX_PERSISTENCE_DB` environment variable
- `src/mcp/state.rs` — `persistence_store: Option<SqliteStore>` field on `McpState`
- `src/mcp/tools.rs` — Four new persistence tools:
  - `save_context` — Explicit manual checkpoint to DB
  - `list_sessions` — Show tracked sessions/files
  - `replay_history` — Replay deltas from DB up to target sequence (crash recovery)
  - `purge_old_deltas` — Trim old delta history by age
- Schema v1: `contexts` (baselines + IR BLOBs), `deltas` (sequential payloads), `symbols` (symbol table entries), `sessions` (workspace tracking), `_schema_version` (migration tracking)
- Persistence hooks in `provide_code_context` (FullCompress + DeltaTransport paths) and `restore_context` (DB clear)
- Non-fatal persistence: all DB writes are fire-and-forget with `eprintln!` warnings — compression never fails due to DB issues

#### FAANG Audit — Zero-Touch Workflow
- `docs/FAANG_AUDIT_ZERO_TOUCH.md` — Complete audit of zero-touch workflow and persistence layer with 4 issues found and fixed

### Changed
- `handle_tools_list`, `handle_prompts_list`, `handle_prompts_get` now take `&mut McpState` for cache hint injection
- `inject_cache_breakpoints` signature extended with 6th parameter: `tokenizer: Option<&dyn Tokenizer>`
- All persistence `save_context` calls in `handle_provide_code_context` (both FullCompress and DeltaTransport paths) now use the pluggable tokenizer for accurate token counts instead of the `estimate_tokens` chars/4 heuristic
- `compute_workspace_breaker` no longer has `#[allow(dead_code)]` — it's wired into the `compress_workspace` handler (manifests hashed directly)
- `handle_context_history` now emits per-file cache breakpoint status and session-level cache hit rate
- Tokenizer parsed earlier in `handle_provide_code_context` so both persistence and cache breakpoints use real token counts

### Fixed

#### XHTML Self-Closing Tag Parsing (F-FULL-XX)
- `src/angular_meta/template.rs` — Fixed `process_element_node` silently losing tag names, custom element detection, and all attribute bindings (property, event, two-way, structural directives) for XHTML-style self-closing Angular components. tree-sitter-html 0.20.x wraps `<app-avatar />` in an `element` node containing a `self_closing_tag` child rather than a `start_tag`. The previous `extract_tag_name_from_element` call looked only for `start_tag` children, silently returning `None` for self-closing elements. Added a `find_child(node, "self_closing_tag")` check at the top of `process_element_node` that delegates to the existing `process_self_closing_tag_node` handler.
- This bug affected all modern Angular templates using XHTML self-closing syntax inside `@if`, `@for`, `@switch`, `@defer` blocks, and standalone self-closing components like `<app-avatar [user]="user" />`.

#### Inline Template Shape Extraction (F-ANG-XX)
- `src/angular_meta/decorators.rs` — New `DecoratorsResult` struct carries both `lines` (Φ markers) and `inline_template` (raw template content from `template: '...'`). `extract_decorators` now returns `Option<DecoratorsResult>` instead of `Option<Vec<String>>`, enabling downstream consumers to access the inline template without re-parsing.
- `src/angular_meta/mod.rs` — `run_meta_layer` now runs `extract_template_shape` (tree-sitter-html) on inline template content from `@Component({template: '...'})` decorators, emitting `Φtpl:` marker lines with structural shape analysis (tags, bindings, directives, control-flow blocks) for components using inline templates.

### Added
- 7 new tests: 6 XHTML self-closing template tests (`extracts_self_closing_xhtml_component`, `extracts_self_closing_xhtml_in_container`, `extracts_self_closing_in_control_flow`, `extracts_void_element_with_bindings`, `extracts_multiple_self_closing_at_root`, `marker_line_includes_self_closing_components`) + 1 inline template integration test (`meta_layer_extracts_inline_template_shape`)

### Test count
- 798/798 tests pass (13 new from SQLite persistence tests + XHTML fix + inline template extraction)
- 0 clippy warnings

---

