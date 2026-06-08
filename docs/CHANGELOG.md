# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
| 0.1.4 | 2026-06-08 | Tracks C+D: Phi marker centralisation + god-function split — 301 tests, 0 clippy warnings |
| 0.1.3 | 2026-06-08 | Track B: `AngularGraphBuilder` typestate split — 293 tests, 0 clippy warnings |
| 0.1.2 | 2026-06-07 | Angular Meta-Layer Phase 3 (cross-file DI + selector graph) — 279 tests, 0 clippy warnings |
| 0.1.1 | 2026-06-07 | Angular Meta-Layer Phase 2.5 (modern Angular 17–21 syntax) — 244 tests, 0 clippy warnings |
| 0.1.0 | 2026-06-07 | Initial release — all 5 FAANG audit phases complete, 121 tests, 0 clippy warnings |
| 0.0.0 | 2026-06-06 | Audit baseline — 58 tests, 41 findings |
