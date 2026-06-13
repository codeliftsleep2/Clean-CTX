# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.7] — Unreleased

### Added
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

### Changed
- `handle_tools_list`, `handle_prompts_list`, `handle_prompts_get` now take `&mut McpState` for cache hint injection
- `inject_cache_breakpoints` signature extended with 6th parameter: `tokenizer: Option<&dyn Tokenizer>`
- All persistence `save_context` calls in `handle_provide_code_context` (both FullCompress and DeltaTransport paths) now use the pluggable tokenizer for accurate token counts instead of the `estimate_tokens` chars/4 heuristic
- `compute_workspace_breaker` no longer has `#[allow(dead_code)]` — it's wired into the `compress_workspace` handler (manifests hashed directly)
- `handle_context_history` now emits per-file cache breakpoint status and session-level cache hit rate
- Tokenizer parsed earlier in `handle_provide_code_context` so both persistence and cache breakpoints use real token counts

### Fixed
- All remaining `estimate_tokens()` call sites in `handle_provide_code_context` persistence blocks replaced with pluggable tokenizer counts

### Test count
- 1006/1006 tests pass
- 0 new clippy warnings (1 pre-existing `too_many_arguments` on `queue_save_context`, 1 pre-existing `compute_workspace_breaker` dead-code suppression)

## [0.1.6] — 2026-06-10

### Added

#### Zero-Touch Workflow
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

## [0.1.5] — 2026-06-08

### Changed

#### FAANG Audit — Compiler IR Phase E (F-30 through F-47)
- `src/ir/compiler.rs` — **F-30:** `compile()` now returns `Result<CompiledIR, CompileError>` (typed enum with `Capture`, `Layer`, `NoCaptures` variants) instead of `Box<dyn std::error::Error>`. **F-31:** `id_counter` changed from `u32` to `u64` to prevent arithmetic overflow.
- `src/ir/patterns.rs` — **F-33/F-34:** Documented that `PatternOp::consumed()` is a heuristic approximation; actual count is computed by `try_compress_pattern`.
- `src/ir/render.rs` — **F-35/F-36/F-37:** Documented ASYNC `$a` backward compatibility, unknown flag handling, and fidelity match arm rationale.
- `src/ir/positional.rs` — **F-39:** Documented `PositionalConfig` struct rationale. **F-40:** Documented `verify_round_trip` return semantics. **F-41:** Fixed misleading "tokens" docstring (function returns char counts). **F-42/F-43:** Documented encoding naming and `+ 12` envelope estimate.
- `src/ir/layers/angular.rs` — **F-44/F-45:** Documented text round-trip through `parse_phi_line` as known design debt.
- `src/ir/layers/mod.rs` — **F-46:** Documented `LayerContext` ownership semantics for `GlobalSymbolTable`.
- `src/ir/layers/typescript.rs` — **F-47:** Replaced byte-level parsing with char-level iteration in `extract_class_relationships` to prevent issues with multi-byte UTF-8 and non-ASCII whitespace.
- `src/ir/mod.rs` — Exported `CompileError` for downstream consumers.

### Test count
- 318/318 IR tests pass (0 failures, 0 regressions)
- 0 clippy warnings (`cargo clippy --all-targets -- -D warnings`)

---

## [0.1.4] — 2026-06-08

### Added

#### Track C — Phi Marker Grammar Centralisation (F-ANG-06)
- `src/angular_meta/markers.rs` — `PhiLineKind` enum (single source of truth for all 14 marker kinds), `PhiLine` trait with per-variant struct impls (`ComponentLine`, `ServiceLine`, `ModuleLine`, `DirectiveLine`, `PipeLine`, `InputLine`, `OutputLine`, `ModelLine`, `InjectsLine`), generic `expand_phi_in_line` / `expand_phi` loops replacing 3 scattered tables
- Adding a new marker is now a 1-step change (add `PhiLineKind` variant + impl) instead of 3-step (builder + two tables)
- 3 new structural tests: `phi_line_kind_uniqueness`, `phi_vocab_is_bijective`, `phi_line_round_trip`

#### Track D — God-function split + extract_class_blocks rewrite (F-ANG-15, F-ANG-03, F-ANG-20)
- `src/mcp/workspace.rs` — `compress_workspace_dir` decomposed into 5 focused helpers: `format_manifest_header`, `compress_pass`, `bundle_pass`, `graph_pass`, `format_manifest_footer`, with `PassContext` struct for shared state
- `src/mcp/workspace.rs` — `extract_class_blocks` rewritten from 137-line duplicate state machine to ~20-line driver delegating to Track A-promoted `decorators::find_class_body_open` + `decorators::find_matching_brace`, with new `find_decorator_start` handling `export`/`abstract`/`default`/`declare` modifier keywords
- 5 new tests: `extract_class_blocks_does_not_panic_on_unclosed_body`, `extract_class_blocks_handles_empty_input`, `compress_pass_emits_per_file_section`, `bundle_pass_emits_phi_bundle_and_footer`, `graph_pass_emits_phi_graph_section`

### Fixed
- `src/angular_meta/decorators.rs` — `consume_call_expression` termination bug: after the closing `)` brought depth to 0, the loop continued scanning past the decorator boundary because `depth == 0` was checked at loop top, not immediately after decrement. Fixed by returning immediately when depth reaches 0. This was a pre-existing bug that prevented decorator args from being extracted when the decorator call was followed by class body content.
- `src/mcp/workspace.rs` — `find_decorator_start` now scans backwards through decorator names (`Injectable`, `Component`, etc.) to find `@`, not just checking the character immediately before `(`

### Test count
- 301/301 tests pass (5 new from Tracks C+D)
- 0 clippy warnings (`cargo clippy --all-targets -- -D warnings`)

---

## [0.1.3] — 2026-06-08

### Added
- `src/angular_meta/graph.rs` — New `AngularGraphBuilder` type (F-ANG-05, Track B): the mutable builder holds `register_class` and `build(self)` consumes it, returning an immutable `AngularGraph` with no public `register_class`/`resolve_all` methods. `AngularGraph::new()` removed from public API; the only construction path is `AngularGraphBuilder::build()`.
- 2 new typestate tests: `builder_consumes_self` (documents the compile-time guarantee) and `resolved_flag_always_true_for_builder_output` (replaces the old `!is_resolved()` check that was only possible on the now-removed `AngularGraph::new()`)

### Changed
- `GraphCollector::build_graph` now drives `AngularGraphBuilder` internally instead of calling `AngularGraph::register_class` / `resolve_all` directly
- `AngularGraph::all_classes` now returns entries in insertion order (no longer sorts by `class_name`) — matches the doc-comment and reduces allocation

### Test count
- 293/293 tests pass (2 new from Track B)
- 0 clippy warnings (`cargo clippy --all-targets -- -D warnings`)

---

## [0.1.2] — 2026-06-07

### Added

#### Angular Meta-Layer Phase 3 (Cross-File Dependency Graph)
- `src/angular_meta/graph.rs` — `AngularGraph` struct with `ClassKind` (Component/Service/Directive/Pipe/Module), `ClassEntry` metadata, DI injection resolution (`resolve_inject_type` → `"UserService@α12"`), selector linkage (`resolve_selector` → `"UserCardComponent@α9"`), `Φgraph:` marker lines with injects/injected-by edges, `§ΦGRAPH` footer formatter
- `src/angular_meta/graph_state.rs` — `AngularGraphHandle` (Arc<Mutex<Option<AngularGraph>>>) for thread-safe McpState integration
- `src/mcp/state.rs` — New `angular_graph: AngularGraphHandle` field for cross-file graph lifecycle
- `src/mcp/workspace.rs` — Phase 3 post-compression graph building pass: text-based class block extraction (`extract_class_blocks`), `GraphCollector` batch collection, angular graph build + store + manifest emission
- `src/angular_meta/decorators.rs` — New `extract_graph_entries()` public function returning `(class_name, kind, selector, injects, pipe_name)` for graph registration
- Test fixtures: `src/test_files/angular/graph/` directory with 4 files (`user-card.component.ts`, `user.service.ts`, `logger.service.ts`, `user-page.component.ts`) for cross-file DI + selector testing
- 36 new unit tests: graph construction/selectors/DI/footers (14), DI resolution including transitive + reverse edges + resolution failure (11), selector linkage including multi-class + directive + pipe (11)
- Total test count: 279 (up from 244)
- `cargo clippy --all-targets -- -D warnings`: 0 warnings (clean)
- Documentation updated: `docs/ARCHITECTURE_OVERVIEW.md` (new Phase 3 section + module tree), `docs/ROADMAP.md` (R-22 Phase 3 ✅)

### Fixed
- Collapsible `if` in `src/angular_meta/template.rs` (pre-existing clippy warning suppressed by collapsing nested check into single condition)

---

## [0.1.1] — 2026-06-07

### Added

#### Angular Meta-Layer Phase 2.5 (Modern Angular 17–21 Syntax)
- `src/angular_meta/template.rs` — Text-node scanning for `@if`/`@for`/`@switch`/`@defer`/`@let` control-flow syntax; `self_closing_tag` node handler for `<app-avatar />` tags
- `src/angular_meta/decorators.rs` — `collect_signal_fields()` for `input()`, `output()`, `model()`, `inject()` signal function calls
- `src/angular_meta/markers.rs` — `Φmodel:` marker builder and `build_model_line()` for two-way binding signals
- Test fixtures: `user-card-modern.component.ts`, `user-card-modern.component.html`, `user-card-mixed.component.html`
- 19 new tests: 17 template tests (modern syntax, mixed legacy/modern, false-positive prevention, comprehensive integration), 2 markers tests (`Φmodel:` builder + expand)
- Total test count: 244 (up from 229)

### Fixed
- `self_closing_tag` not handled by tree-sitter-html walker — added explicit arm with `process_self_closing_tag_node`
- `@let` deduplication — multiple `@let` in same text node collapse to 1 entry after dedup
- Regex dependency avoided — implemented `contains_at_keyword` with manual word-boundary heuristics instead of `regex`/`lazy_static`

---

## [0.1.0] — 2026-06-07

### Added

#### Angular Meta-Layer Phase 1 + 2 (Decorators + Bundling)
- `src/angular_meta/bundler.rs` — File-triplet resolver: `*.component.ts` → `*.{html,scss,css,sass,less}` siblings
- `src/angular_meta/template.rs` — tree-sitter-html Angular-syntax template extractor: tags, `[prop]`, `(event)`, `[(banana)]`, `*ngIf/*ngFor/*ngSwitch`, `{{ }}`, custom-elements
- `src/angular_meta/style.rs` — CSS/SCSS class selector + variable + at-rule extractor (byte-level scanner, no regex)
- `src/angular_meta/footer.rs` — `§ΦMAP` workspace footer with `FooterBuilder` for incremental bundle registration
- `src/dictionary/path.rs` — Bundle alias (Φ1, Φ2, …) support alongside α/β/γ path aliases
- `src/angular_meta/detect.rs` — `is_angular_sibling()` for Angular-adjacent file detection
- `src/mcp/workspace.rs` — Extended `collect_source_files` to include `.html`/`.scss`/`.css`/`.sass`/`.less`; post-compression bundling pass emits `ΦBUNDLE` groups with template/style shape summaries and `§ΦMAP` footer
- Test fixtures: `user-card.component.html`, `user-card.component.scss`, `user-page.component.ts`, `user-page.component.html`, `user-page.component.scss`, `non_triplet_file.ts`
- 56 new unit tests: bundler resolution (14), template extraction (18), style extraction (16), footer formatting (8)
- `Cargo.toml` — Added `tree-sitter-html = "=0.20.0"` (pinned to tree-sitter 0.20.x ABI)

#### Core Compression
- Three-fidelity compression engine (Low/Medium/High) with configurable behavior
- Tree-sitter-based AST parsing for TypeScript/JavaScript (`.ts`, `.js`) and C# (`.cs`)
- Fidelity-aware filtering: Low strips all modifiers, Medium preserves async/exports/markers, High preserves full keywords
- Symbol opcode dictionary: 34 built-in primitives (`$c`, `$s`, `$b`, `$P`, etc.) + auto-assigned custom opcodes for repeated tokens
- Automatic path alias mapping (`α1`, `α2`, …) with `§MAP` footer
- Behavioral marker system (`⊕guard`, `⊕loop`, `⊕⇒`, `⊕!throw`, `⊕export`)

#### MCP Server
- `compress_code_context` — Single file compression tool
- `decompress_code_context` — Compressed output expansion tool
- `compress_workspace` — Directory-tree compression with shared path aliases
- `diff_code_context` — AST-level structural diff with baseline snapshots
- `cleanctx-notation` system prompt for LLM instruction

#### Caching & Performance
- `LocalStateCache` with SHA-256 content-hash registry (F-14)
- Baseline snapshot registry for `diff_code_context` (F-21)
- Raw-token count cache to skip BPE re-encode on cache hits (F-23)
- Precomputed sorted opcode list in decompressor (F-15)

#### Security & Hardening
- `OnceLock`-cached BPE engine with explicit startup init (F-01)
- 16 MB JSON-RPC line size cap with drain recovery (F-02)
- `Fidelity::parse` returns `Result` — typo rejection with `-32602` errors (F-03)
- 10 MB `MAX_FILE_BYTES` guard on `compress_file` (F-18)
- Symlink-loop protection via canonical-path tracking + `MAX_WALK_DEPTH=32` (F-17)
- Config `is_excluded` uses segment-based glob matching (F-12)
- No `unsafe` blocks in the entire codebase

#### Configuration
- `.clean-ctx.json` project-level configuration (F-05)
- `exclude_patterns` with glob/dot-pattern matching
- `fidelity_overrides` per file extension
- `type_aliases` for custom type name resolution
- `OnceLock`-cached config lookup (F-11)

#### Documentation
- FAANG audit report with 41 findings across 5 phases
- SOLID refactoring plan and execution history
- Architecture overview with module structure and pipeline stages
- Developer documentation with language/tool/opcode extension guides
- IDE configuration examples for Cline, Cursor, Claude Code, Continue.dev, Zed

#### CI & Tooling
- `.github/workflows/ci.yml` — cargo check + clippy + test + audit on push/PR
- `deny.toml` — license allow-list, vulnerability deny, unsafe code ban
- Pre-commit checklist and code quality gates

### Fixed

#### Phase 1 — Crash Safety
- BPE engine no longer panics the server on load failure (was `cl100k_base().unwrap()`) → cached in `OnceLock`
- JSON-RPC reader no longer OOMs on unbounded lines → 16 MB cap
- `Fidelity::parse` no longer silently defaults typos to Low → returns `Result`

#### Phase 2 — Correctness
- `format_final_output` now reports real class/method/import counts (was always 0)
- `CleanCtxConfig` is now plumbed through `McpState` (was loaded to `_config` and discarded)
- `word_boundary_replace` now handles non-ASCII characters correctly (was ASCII-only)
- `extract_class_name` modifier strip now loops until stable (was single-pass)
- Fidelity is now threaded through capture-pipeline closure (was hard-coded Low)

#### Phase 3 — Session Coherence
- `compress_workspace` now shares path dictionary and cache with per-file tool (was fresh per call)
- `Fidelity` now implements `Hash` + `Eq` with stable `as u8` cache key
- `find_config` result is cached in `OnceLock` (was uncached)
- `is_excluded` now uses segment-based glob matching (was substring match)
- Workspace errors are surfaced as structured `WorkspaceResult` (were inline comments)
- Cache-hit path no longer re-tokenizes the entire source (raw-token count side-table)

#### Phase 4 — Performance & Hardening
- Decompressor sorts opcode list once in `parse()`, not per line in `decompress()`
- `strip_modifiers` unified into `modifiers.rs` (was duplicated and quadratic)
- Symlink loops detected and skipped in workspace walk
- Files larger than 10 MB return a clean error instead of OOM
- `diff_code_context` fast-path for unchanged files (hash-based skip)
- BPE data path is embedded in binary via `tiktoken-rs` 0.11 `include_bytes!`

#### Phase 5 — Hygiene
- 19 hygiene findings fixed: dead code removed, shim layers deleted, bandaid scripts removed
- O(1) path alias lookup (was O(n) linear scan)
- `BTreeMap` → `HashMap` in all caches (no sorted iteration needed)
- `write!`/`writeln!` replaces `format!` allocations in hot path
- `DiffKind`/`DiffTarget` derives `Serialize`/`Deserialize`
- Cargo.toml: `license`, `rust-version`, `[lib]`, `[[bin]]` added
- tree-sitter crates pinned to exact versions with `// SAFETY:` comments
- `#![allow(dead_code)]` and shim modules removed from `lib.rs`

### Deferred
- **F-19** — Streaming workspace walk (replaced collect-then-sort with `walkdir`)
- **F-20** — Rayon parallelization for `compress_workspace` (blocked by tree-sitter `!Send` constraint)

---

## [0.0.0] — Initial audit baseline

The codebase was audited at 28 production source files, 13 test files, ~3,300 LoC with 58/58 tests passing. The audit found 41 distinct issues ranging from a server-crashing panic to substring-match globs. All issues resolved across 5 phases.

---

## Versioning

This project follows [Semantic Versioning](https://semver.org/). Major version zero (0.y.z) is for initial development. Breaking changes may occur at minor versions before 1.0.0.

### Version history

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.7 | Unreleased | Prompt cache optimization — 1006 tests, 0 clippy warnings |
| 0.1.6 | 2026-06-10 | Zero-touch workflow + SQLite persistence + XHTML fix + inline template — 798 tests, 0 clippy warnings |
| 0.1.5 | 2026-06-08 | FAANG Audit Compiler IR Phase E (F-30 through F-47) — 318 tests, 0 clippy warnings |
| 0.1.4 | 2026-06-08 | Tracks C+D: Phi marker centralisation + god-function split — 301 tests, 0 clippy warnings |
| 0.1.3 | 2026-06-08 | Track B: `AngularGraphBuilder` typestate split — 293 tests, 0 clippy warnings |
| 0.1.2 | 2026-06-07 | Angular Meta-Layer Phase 3 (cross-file DI + selector graph) — 279 tests, 0 clippy warnings |
| 0.1.1 | 2026-06-07 | Angular Meta-Layer Phase 2.5 (modern Angular 17–21 syntax) — 244 tests, 0 clippy warnings |
| 0.1.0 | 2026-06-07 | Initial release — all 5 FAANG audit phases complete, 121 tests, 0 clippy warnings |
| 0.0.0 | 2026-06-06 | Audit baseline — 58 tests, 41 findings |