# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] — 2026-06-07

### Added

#### Angular Meta-Layer Phase 2 (File-Triplet Bundling)
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
| 0.1.1 | 2026-06-07 | Angular Meta-Layer Phase 2 (file-triplet bundling) — 229 tests, 0 clippy warnings |
| 0.1.0 | 2026-06-07 | Initial release — all 5 FAANG audit phases complete, 121 tests, 0 clippy warnings |
| 0.0.0 | 2026-06-06 | Audit baseline — 58 tests, 41 findings |
