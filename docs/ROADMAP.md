# Clean-CTX — Future Roadmap

**Last updated:** 2026-07-01 (A-07 complete)

> **Living document.** Items are reviewed and pruned every release. Status legend: 📋 proposed · 🚧 in-progress · ✅ done · ⏸️ deferred

---

## At a Glance

| Horizon | Target Release | Theme | Items |
|---------|----------------|-------|------:|
| **Now** | v0.2.0 | Real-world ready | 4 |
| **Next** | v0.3.0 | Advanced capabilities | 10 |
| **Later** | v1.0.0+ | Ecosystem & integrations | 6 |
| **Architectural** | Continuous | Code health & tooling | 13 |
| **Community** | Continuous | Docs & marketing | 5 |

---

## Completed (v0.1.x – v0.2.0-rc1) — shipped ✅

These items are complete and documented. Listed for historical context.

| ID | Title | Shipped in | Notes |
|----|-------|-----------|-------|
| **R-22** | Angular Meta-Layer | v0.1.x | Phase 1 (decorators) ✅ · Phase 2 (triplet bundling) ✅ · Phase 2.5 (Angular 17–21 syntax) ✅ · Phase 3 (cross-file DI + selector graph) ✅ |
| **R-30** | SQLite Persistence Layer | v0.1.x | `SqliteStore` with WAL mode, contexts/deltas/symbols/sessions tables, content-hash deterministic IDs, non-fatal fire-and-forget writes |
| **R-31** | Zero-Touch Workflow + Heuristics Engine | v0.1.x | `provide_code_context` single entry point, `heuristics.rs` auto-selects fidelity + strategy, session stats dashboard |
| **R-32** | ULTRA_COMPACT_PLAN — All Phases | v0.1.x | Phase I (string table + field-level delta diffing + contextual delta) ✅ · Phase II (hierarchical IR + binary wire) ✅ · Phase III (header elision + structural dedup + cross-file symbols + Huffman + VarInt micro-opcodes) ✅ · Phase IV (text delta transport) ✅ · 759 tests passing |
| **R-11** | Cross-File Symbol Resolution | v0.1.x | Angular Phase 3 DI graph + selector linkages + transitive dependencies + global cross-language TS/C# symbol table — supersedes original scope |
| **R-20** | Persisted Baseline Cache | v0.1.x | Full SQLite persistence via `SqliteStore` with cross-session delta replay — supersedes original in-memory-only scope |
| **R-03** | Compression Ratio Statistics | v0.1.x | `context_stats` tool + `session_stats.rs` dashboard with per-file/session token savings metrics |
| **R-13** | Compress-to-Compress Diff | v0.1.x | Text delta + IR field-level delta transport + `§GSYM` global dictionary + path alias framework — supersedes original scope |
| **R-35** | CBM Integration (Phase 0 + 1) | v0.1.8 | `CbmClient` (JSON-RPC 2.0 subprocess with retry + exponential backoff) ✅ · `GraphBridge` (DashMap TTL caching + `detect_changes` cache invalidation) ✅ · `CbmProxy` (pipe-level response interception + pluggable tokenizer) ✅ · `json_compress.rs` (key shortening, envelope stripping, ~78% compression of CBM responses) ✅ · 36 regression/integration/e2e tests ✅ |
| **R-19** | Pluggable tokenizers | v0.1.7 | ✅ `Tokenizer` trait with `cl100k` (GPT-4), `o200k` (GPT-4o), `claude`, and `llama3` implementations. Selectable via `tokenizer` tool argument or `.clean-ctx.json` config. Process-global BPE caches, ratio-adjusted approximations for Claude/Llama-3. |
| **R-01d** | Java language layer | v0.1.7 | ✅ `JavaLayer` in `ir/layers/java.rs` — extends/implements with generics stripping, constructor detection, abstract/static/private/protected flags, Jakarta/Spring annotation patterns. 25 tests. |
| **R-01c** | Rust language layer | v0.1.7 | ✅ `RustLayer` in `ir/layers/rust.rs` — derives, generics, cfg attributes, impl relationships, self kind, unsafe. 30 tests + 30 rust_integration tests. |
| **R-38** | Spring Boot Meta-Layer | v0.1.8 | ✅ `SpringMetaLayer` in `ir/layers/spring.rs` — Φ markers for @RestController, @Controller, @Service, @Repository, @Configuration, @RequestMapping, @Autowired, @Value, @Bean, @ConfigurationProperties. `SpringGraph` for cross-file DI resolution. 45+ tests. |
| **R-39** | Secret Scrubbing | v0.1.7 | ✅ Regex-based scrubbing for AWS keys, GitHub tokens, JWTs, PEM keys, Bearer tokens. `ScrubFailClosed` semantics. Runs in proxy pipeline. |
| **R-40** | Shell Output Filtering | v0.1.7 | ✅ 26 built-in TOML filters (cargo, npm, git, pytest, tsc, dotnet, ng, eslint, docker, go, kubectl, and more). Filter pipeline: strip_ansi → replace → match_output collapse → grouping → §FILTERED marker. Community filter support via `.clean-ctx/filters/`. |
| **CBM Audit** | CBM integration audit fixes | v0.1.8 | 6 findings resolved: CRITICAL (CBM enrichment data compression), HIGH (call_tool_raw retry, enrich_with_cbm timeout guard), MEDIUM (pluggable tokenizer in proxy), LOW (cache eviction pattern, visibility). 1,331 tests passing. |
| **R-35 (P2)** | CBM Phase 2 — Intelligence Layer seeding | v0.1.9 | Fixed CBM client API mismatches (`search_graph`/`trace_path` param names, replaced non-existent `get_symbol_importance`/`get_dead_code` with Cypher queries). Intelligence Layer (pagerank.rs, fidelity.rs) now correctly blends CBM cross-file `in_degree` scores (60% IR + 40% CBM) for adaptive per-symbol fidelity. `enrich_with_cbm` in tool_handlers injects compressed CBM metadata into responses. See `docs/CBM_API_AUDIT_AND_PHASE2_PLAN.md`. |
| **Compiler-IR Audit** | Compiler-IR audit + clippy cleanup | v0.1.8 | Verified all 8 phases (A–H). Resolved 29 clippy warnings across entire build. Rewrote COMPILER_IR.md from spec to implementation docs. All 1,277 tests pass with 0 clippy warnings. |
| **A-09** | Multi-threaded MCP server dispatch | v0.2.0-rc1 | ✅ Production-grade `Dispatcher` with `crossbeam_channel` bounded queue + backpressure, `RwLock`-protected state for parallel reads, `catch_unwind` panic recovery, dedicated stdout writer thread, request tracing/observability, configurable worker count (auto-detect CPU count). Includes 6+ unit tests covering spawn lifecycle, concurrent mutations, panic recovery, and tracing. **Formerly P0 Critical — #1 adoption blocker.** See `src/mcp/dispatcher.rs`. |
| **A-10** | Proxy hardening: auth + rate limiting | v0.2.0-rc1 | ✅ Optional `X-Api-Key` header authentication via `PROXY_API_KEY` env var. Per-client-IP token bucket rate limiter (configurable `RATE_LIMIT_RPS`/`RATE_LIMIT_BURST`, default 60/10). Returns `401 Unauthorized` for bad/missing keys, `429 Too Many Requests` when rate limited. Rate limiter uses GC-enabled sliding window to prevent unbounded map growth. Stats endpoint exposes rate limiter status. Nginx sidecar pattern documented in `docs/PROXY.md`. 141 proxy tests passing (6 new rate limiter tests + 5 new server tests). |
| **A-12** | Tree-sitter version migration | v0.2.0-rc1 | ✅ Migrated from `=0.20.x` pinned versions to `^0.26.x` semver ranges. All 6 tree-sitter crates updated (`tree-sitter` 0.26.10, `c-sharp` 0.23.5, `typescript` 0.23.2, `html` 0.23.2, `rust` 0.24.2, `java` 0.23.5). Parser API migrated (LanguageFn → Language, `&Language` borrows, `StreamingIterator` for QueryMatches). CI guard (`scripts/check-tree-sitter-versions.ps1`) ensures all grammars share the same `tree-sitter-language` ABI. 1,360 tests passing, 0 clippy warnings. **Unblocks F-20 (Parser is now `Send`).** See `docs/TREE_SITTER_MIGRATION_PLAN.md`. |
| **A-11** | Meta-layer detection hardening | v0.2.0 | ✅ Migrated from `source.contains()` string scanning to proper AST-node matching using tree-sitter queries for Angular/Spring Boot detection. Eliminates false positives from comments and string literals. |
| **A-13** | Resource limits and memory guardrails | v0.2.0 | ✅ Added `ResourceLimits` config with max file size (10 MB default), max workspace file count (10,000 default), max memory usage. Graceful error messages instead of OOM crashes. Wired into compression entry points (`compress_code_context`, `compress_workspace_dir`, `delta_code_context`) and proxy body buffers. |
| **A-14** | CI/CD awareness | v0.2.0 | ✅ Auto-disables persistence when `CI=true` or `TF_BUILD=true` env vars are detected. Documented in default `.clean-ctx.json`. Prevents stale `persistence.db` from leaking between CI builds. |
| **A-15** | Configuration precedence documentation | v0.2.0 | ✅ Documented explicit precedence rules (tool arg > env var > config file > default) in `docs/CONFIGURATION.md`. Includes complete `.clean-ctx.json` example, env var reference, resource limits docs, CI/CD behavior, debug instructions. |
| **F-19** | Streaming workspace walk | v0.2.0 | ✅ Replaced recursive `collect_source_files_inner` with `walkdir` streaming visitor. Pre-allocates path aliases during single-threaded file-collection step for deterministic `αN` numbering. Required before F-20. |
| **F-20** | Rayon parallelization for `compress_workspace` | v0.2.0 | ✅ Applied `par_iter()` to the per-file compression loop with Mutex-wrapped manifest/errors collectors. Pre-assigns aliases deterministically (F-21) before parallel work. **Prerequisites:** F-19 (walkdir) ✅, A-12 (Parser `Send`) ✅. |
| **F-21** | Deterministic alias assignment | v0.2.0 | ✅ Pre-assigns α1, α2…αN aliases sequentially before the parallel Rayon loop. Once assigned, `get_or_create_alias` is a read-only HashMap lookup safe for concurrent access. |
| **F-22** | Workspace compression result caching | v0.2.0 | ✅ Caches the complete `WorkspaceResult` keyed by a content hash of file paths + mtimes/sizes. Subsequent calls with no file changes return the cached result instantly. Saves 5-15s per redundant call for a 100-file workspace. |

---

## Foundation (v0.2.0) — "Architectural hardening (BLOCKING)"

**✅ ALL FOUNDATION ITEMS COMPLETE.** v0.2.0 is no longer blocked by architectural hardening.

These items were prerequisites for enterprise adoption, identified by a FAANG-level architectural review. All have been resolved:

| ID | Title | Status |
|----|-------|--------|
| **A-09** | Multi-threaded MCP server dispatch | ✅ v0.2.0-rc1 |
| **A-10** | Proxy hardening: auth + rate limiting | ✅ v0.2.0-rc1 |
| **A-11** | Meta-layer detection hardening | ✅ v0.2.0 |
| **A-12** | Tree-sitter version migration | ✅ v0.2.0-rc1 |
| **A-13** | Resource limits and memory guardrails | ✅ v0.2.0 |
| **A-14** | CI/CD awareness | ✅ v0.2.0 |
| **A-15** | Configuration precedence documentation | ✅ v0.2.0 |

---

## Now (v0.2.0) — "Real-world ready"

These items address the most common user requests and unlock adoption on larger codebases.
**NOTE:** Foundation items are now **all complete** — v0.2.0 is no longer blocked.

### ✅ Completed Now items

| ID | Title | Description |
|----|-------|-------------|
| **F-19** | Streaming workspace walk | ✅ Replaced recursive `collect_source_files_inner` with `walkdir` streaming visitor. Pre-allocates path aliases during single-threaded file-collection step for deterministic `αN` numbering. |
| **F-20** | Rayon parallelization for `compress_workspace` | ✅ Applied `par_iter()` to the per-file compression loop with Mutex-wrapped manifest/errors collectors. Pre-assigns aliases deterministically (F-21) before parallel work. |
| **F-21** | Deterministic alias assignment | ✅ Pre-assigns α1, α2…αN aliases sequentially before the parallel Rayon loop. Once assigned, `get_or_create_alias` is a read-only HashMap lookup safe for concurrent access. |
| **F-22** | Workspace compression result caching | ✅ Caches the complete `WorkspaceResult` keyed by a content hash of file paths + mtimes/sizes. Subsequent calls with no file changes return the cached result instantly. Saves 5-15s per redundant call for a 100-file workspace. |

### Remaining Now items

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **A-07** | Property-based tests with `proptest` | Fuzz-style input tests for the decompressor, the config glob matcher, and the modifier stripper. Would have caught F-06 (Unicode) and F-12 (substring match) regressions. | 1-2 days | 🔴 High |
| **R-29** | Intelligence Layer | Ranked context delivery on top of the existing compression stack. **Phase 1 (complete):** PageRank symbol scoring + CBM-informed adaptive per-symbol fidelity (60% IR + 40% CBM blend). **Phase 2 (complete):** Blast radius integrated into `handle_provide_code_context` and `handle_delta_code_context` — depth-1 affected files compressed at Low fidelity and appended with `§IMPACT` markers. Regression tests added. **Phase 3 (not started):** Token budget knapsack packing. All phases opt-in via `.clean-ctx.json`, zero overhead when disabled. See `docs/INTELLIGENCE_LAYER_PLAN.md`. | 5.5 days | 🔴 High |
| **A-08** | TOKEN_EFFICIENCY_AUDIT findings | 4 open findings: underutilized `source_cache` (High), double IR compile in delta path (Medium), path resolution inconsistency (Medium), fragile source ownership (Low). | 1-2 days | 🟡 Medium |
| **A-04** | Observability upgrade (was Low) | **Upgraded from Low to Medium priority.** Add OpenTelemetry-compatible structured logging (`tracing` + `metrics`) with OTLP export. Export key metrics: compression latency, delta hit rate, cache efficiency, CBM query latency, error rates by category. This is required before any production deployment. | 2-3 days | 🟡 Medium |

---

## Next (v0.3.0) — "Advanced capabilities"

**GATE:** All remaining Foundation items (A-10 through A-15) must be complete before v0.3.0 work begins. Meta-layer items (R-36, R-37, R-23, R-24) additionally depend on A-11 (detection hardening). Note: R-12's dependency on A-09 is now **unblocked** since A-09 shipped.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-01** | Python language layer | Most-requested language. Follows the 4-step guide in `DEVELOPER_DOCUMENTATION.md`. | 1-2 days | 🔴 High |
| **R-02** | Type-aware compression | Inline `type_aliases` from config: `UserId` → `$uid`, `JsonObject` → `$jo`. Currently the type table is loaded but not injected into the capture pipeline. | 2-3 days | 🔴 High |
| **R-12** | Multi-file / git-commit diff | Diff an entire workspace between two git commits; emit per-file deltas in one tool call. Powers "what changed in this PR?" workflows. **A-09 ✅ (complete)** — now unblocked. | 3-5 days | 🔴 High |
| **R-36** | React Meta-Layer | Additive meta-layer on TS/JS. Component/hook/context bundling, prop type compression, React-specific lifecycle markers. **BLOCKED by A-11.** | 3-4 days | 🔴 High |
| **R-37** | Redux Meta-Layer | Additive meta-layer on TS/JS. Action/reducer/selector compression, thunk/saga patterns, store shape compression. **BLOCKED by A-11.** | 2-3 days | 🔴 High |
| **R-23** | NgRx Meta-Layer | Framework-annotation layer for NgRx state management (sits on top of TS + Angular layers). **BLOCKED by A-11.** | 3-4 days | 🔴 High |
| **R-01b** | Go language layer | Second-most requested. | 1-2 days | 🟡 Medium |
| **R-07** | MCP `resources` support | Expose compressed snapshots as MCP resources in addition to tools, enabling LLM clients to read prior state without re-invoking tools. | 1-2 days | 🟡 Medium |
| **R-08** | Improved diff: rename detection | Detect class/method renames (same signature, different name) and emit as `~` with a `renamed from X` hint instead of a delete+add pair. | 1 day | 🟡 Medium |
| **R-41** | Sliding Context Window (Tier 1) | Age-based tool-result truncation in the proxy pipeline, with force-preserve rules (floor + path cross-reference). Opt-in via `SLIDING_WINDOW=1`. See `docs/SLIDING_CONTEXT_WINDOW_PLAN.md`. | 3-5 days | 🟡 Medium |
| **R-42** | Sliding Context Window (Tier 2 — scored pruning) | Relevance-scored pruning beyond simple age. Dry-run mode required before default-on. | 4-6 days | 🟢 Low |

---

## Later (v1.0.0+) — "Ecosystem & integrations"

Items that add value but require demand signal before investing. **YAGNI applies** — these should not be built speculatively.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-26** | Vue Meta-Layer | Additive meta-layer on TS/JS. Single-file component bundling (`.vue` = script + template + style), Composition API markers. **BLOCKED by A-11.** | 3-4 days | 🟡 Medium |
| **R-27** | ASP.NET Meta-Layer | Additive meta-layer on C# layer. Controller/service/repository bundling, DI registration graph (`services.AddScoped<IFoo, Foo>()`), route markers. **BLOCKED by A-11.** | 3-4 days | 🟡 Medium |
| **R-28** | Entity Framework Meta-Layer | Additive meta-layer on C# (sits on top of ASP.NET layer). Entity model compression, DbContext graph, migration markers. **BLOCKED by A-11.** | 2-3 days | 🟢 Low |
| **R-06** | Config hot-reload | File-watcher on `.clean-ctx.json`; debounce + atomic reload of `McpState.config` without restart. | 1 day | 🟡 Medium |
| **R-10** | Tool input validation via `schemars` | Replace hand-rolled `args.get("foo")` chains with `schemars`-derived JSON Schema; tool schemas in `tools.rs` auto-generate from Rust types. | 1-2 days | 🟡 Medium |
| **R-18** | Markdown / JSON / YAML compression | Extend compression to data formats using the same opcode framework with format-specific markers. | 1-2 days | 🟡 Medium |

---

## Architectural improvements (continuous)

Code health work that has no user-facing feature but improves long-term maintainability.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **A-01** | Remove `src/helpers.rs` shim | The 18-line re-export shim should be removed once all internal callers are confirmed to use `crate::compaction::*`. | 1 hour | 🟢 Low |
| **A-02** | Migrate to `walkdir` (precondition for F-20) | Required before parallelism can be added — streaming input is a prerequisite for `par_iter`. Now absorbed into F-19 scope. | 1 day | 🟡 Medium |
| **A-03** | `schemars` for tool schemas | Replaces hand-written JSON Schema in `src/mcp/tools.rs`; less drift between Rust types and advertised schema. | 1-2 days | 🟡 Medium |
| **A-04** | `tracing` + `metrics` | Structured logging + OpenTelemetry-compatible spans; useful for diagnosing slow compressions in production. **Upgraded to Medium priority — see Now list.** | 2-3 days | 🟡 Medium |
| **A-05** | Workspace-aware language detection | When `compress_workspace` runs, cache language detection results across files. TS file next to another TS file is, predictably, TS. | 1 day | 🟢 Low |
| **A-07** | Property-based tests with `proptest` | See Now list. Fuzz-style input tests for decompressor, config glob matcher, modifier stripper. | 1-2 days | 🔴 High |
| **A-08** | TOKEN_EFFICIENCY_AUDIT findings | 4 open findings: underutilized `source_cache` (High), double IR compile in delta path (Medium), path resolution inconsistency (Medium), fragile source ownership (Low). See `docs/TOKEN_EFFICIENCY_AUDIT.md`. | 1-2 days | 🟡 Medium |
| **A-11** | Meta-layer detection hardening | See Foundation list. **P0 Critical — blocks all new meta-layers.** | 1 day | 🔴 **P0 Critical** |
| **A-13** | Resource limits and memory guardrails | See Foundation list. Max file size, workspace count, memory limits with graceful errors. | 1 day | 🔴 High |
| **A-14** | CI/CD awareness | See Foundation list. Auto-disable persistence in CI environments. | 0.5 day | 🔴 High |
| **A-15** | Configuration precedence documentation | See Foundation list. Precedence rules + `--config-dump` flag. | 0.5 day | 🟡 Medium |

---

## Documentation & community (continuous)

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **D-01** | README screenshot / GIF | Top of README showing side-by-side raw vs compressed output for a real class. Increases GitHub star rate. Especially important now with LinkedIn post driving traffic. | 1 hour | 🔴 High |
| **D-02** | Blog post: "Why we wrote a tree-sitter-based LLM compressor" | Marketing + technical deep-dive; good for HackerNews launch and SEO. | 1 day | 🟡 Medium |
| **D-03** | Cross-fidelity benchmark table in README | 50-edit simulation results (96.6% Low / 92.0% Medium / 89.9% High delta savings) already exist in `docs/PERFORMANCE.md` — surface the headline table in README where it's visible on first load. | 2 hours | 🔴 High |
| **D-04** | Competitor comparison table in README | ForgeIndex vs LeanCTX vs Clean-CTX capability matrix. Draft exists in `docs/INTELLIGENCE_LAYER_PLAN.md`. | 1 hour | 🟡 Medium |
| **D-05** | Architecture decision records (ADRs) | Track WHY each major decision was made (tree-sitter, stdio, no-network, Φ marker namespace, additive-only guarantee, etc.). | 2-3 hours per ADR | 🟢 Low |

---

## Deferred

Items explicitly deferred — not forgotten, not prioritized.

| ID | Title | Deferred reason |
|----|-------|----------------|
| **R-05** | Semantic opcode suggestions | Low value vs rest of stack; the Intelligence Layer adaptive fidelity covers the spirit of this more systematically |
| **R-04** | Incremental decompression | Superseded by delta transport (text + IR) which is a better architectural solution |
| **R-14** | LLM-aware prompt optimization | Covered more systematically by R-29 Intelligence Layer (PageRank + adaptive fidelity) |
| **Idea #5** | RLE Delta Batching (ULTRA_COMPACT_PLAN) | Lower priority; deferred in original plan; revisit if delta envelope size becomes a bottleneck |

---

## Carry-over from FAANG Audit

| ID | Title | Description | Status |
|----|-------|-------------|--------|
| **F-19** | Streaming workspace walk | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |
| **F-20** | Rayon parallelization | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |
| **F-21** | Deterministic alias assignment | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |
| **F-22** | Workspace compression result caching | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |
| **A-09** | Multi-threaded MCP server dispatch | ✅ **Completed in v0.2.0-rc1** — moved to Completed section. | ✅ Done |
| **A-10** | Proxy hardening: auth + rate limiting | ✅ **Completed in v0.2.0-rc1** — moved to Completed section. | ✅ Done |
| **A-11** | Meta-layer detection hardening | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |
| **A-12** | Tree-sitter version migration | ✅ **Completed in v0.2.0-rc1** — moved to Completed section. | ✅ Done |
| **A-13** | Resource limits and memory guardrails | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |
| **A-14** | CI/CD awareness | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |
| **A-15** | Configuration precedence documentation | ✅ **Completed in v0.2.0** — moved to Completed section. | ✅ Done |

---

## Prioritization rationale

### Foundation-first ordering (v0.2.0)

**✅ ALL FOUNDATION ITEMS COMPLETE.** v0.2.0 is no longer blocked by architectural hardening.

The Foundation items were prerequisites for enterprise adoption, identified by a FAANG-level architectural review. All 7 items are now resolved:

| ID | Title | Shipped in | Summary |
|----|-------|-----------|---------|
| **A-09** | Multi-threaded MCP server dispatch | v0.2.0-rc1 | Production-grade `Dispatcher` with `crossbeam_channel` bounded queue + backpressure, `RwLock`-protected state, `catch_unwind` panic recovery, dedicated stdout writer thread, request tracing. |
| **A-10** | Proxy hardening: auth + rate limiting | v0.2.0-rc1 | Optional `X-Api-Key` header auth via `PROXY_API_KEY`. Per-client-IP token bucket rate limiter (configurable RPS/burst). 141 proxy tests. |
| **A-11** | Meta-layer detection hardening | v0.2.0 | Migrated from `source.contains()` to AST-node matching using tree-sitter queries. Eliminates false positives from comments/string literals. |
| **A-12** | Tree-sitter version migration | v0.2.0-rc1 | Migrated from `=0.20.x` pins to `^0.26.x`. All 6 crates updated. `Parser` is now `Send`. CI guard in place. |
| **A-13** | Resource limits and memory guardrails | v0.2.0 | `ResourceLimits` config with max file size (10 MB), max workspace file count (10,000), max memory usage. Graceful errors. |
| **A-14** | CI/CD awareness | v0.2.0 | Auto-disables persistence when `CI=true` or `TF_BUILD=true`. Documented in default config. |
| **A-15** | Configuration precedence documentation | v0.2.0 | Precedence rules documented in `docs/CONFIGURATION.md`. Tool arg > env var > config file > default. |

### Now list priorities (v0.2.0)

Foundation is complete. Remaining Now items:

1. **A-07 (proptest)** — Regression insurance against input-validation bugs. Cheap to implement, prevents regressions.
2. **R-29 (Intelligence Layer)** — Phase 1 (PageRank) + Phase 2 (blast radius) complete. Phase 3 (token budget packing) not started.
3. **A-08 (Token Efficiency Audit)** — 4 open findings to resolve before claiming production-readiness.
4. **A-04 (Observability)** — **Upgraded from Low to Medium priority.** OpenTelemetry-compatible tracing/metrics required before any production deployment.

### Next list priorities (v0.3.0)

GATED by remaining Foundation completion (A-10 through A-15). Meta-layer items additionally gated by A-11. Note: R-12's dependency on A-09 is now **unblocked** since A-09 shipped.

| Priority | Item | Dependency |
|----------|------|------------|
| 🔴 High | R-01 Python language layer | None (A-12 unblocks newer grammar crate) |
| 🔴 High | R-02 Type-aware compression | None |
| 🔴 High | R-12 Multi-file git-commit diff | **A-09 ✅ (complete)** — now unblocked |
| 🔴 High | R-36 React Meta-Layer | **A-11 (detection hardening)** |
| 🔴 High | R-37 Redux Meta-Layer | **A-11 (detection hardening)** |
| 🔴 High | R-23 NgRx Meta-Layer | **A-11 (detection hardening)** |
| 🟡 Medium | R-01b Go language layer | None (A-12 unblocks newer grammar crate) |
| 🟡 Medium | R-07 MCP resources support | None |
| 🟡 Medium | R-08 Improved diff: rename detection | None |
| 🟡 Medium | R-41 Sliding Context Window (Tier 1) | A-04 (observability, for pruning audit trail) |
| 🟢 Low | R-42 Sliding Context Window (Tier 2) | R-41, A-07 (proptest for force-preserve invariants) |

### Items explicitly deferred from Next list

The following items that were previously in the Next list have been deferred to Later (v1.0.0+) to focus on Foundation + critical features:

- **R-24 (RxJS Meta-Layer)** — Deferred. No demand signal, blocked by A-11.
- **R-07 (MCP resources)** — Moved to Next. Moderate demand signal.
- **R-08 (Rename detection)** — Moved to Next. Low effort, moderate value.

### Completed pilot stack

The Java + Spring + CBM pilot stack (R-01d Java language layer, R-38 Spring Boot Meta-Layer, R-35 CBM Integration) is complete and shipped in v0.1.7–v0.1.9.

### CBM integration architecture decisions (locked)

- Communication: MCP JSON-RPC between Clean-CTX and CBM (loose coupling, no direct DB access)
- Runtime model: Both run as separate MCP servers; optional `--with-cbm` flag for auto-start
- Graph data flow: CBM provides graph intelligence → Clean-CTX consumes via `search_graph`, `trace_path`, `detect_changes`, `get_architecture`
- Intelligence Layer synergy: CBM symbol importance seeds → Clean-CTX PageRank → per-symbol adaptive fidelity
- **FAANG review note:** CBM integration quality is high (circuit breaker, retry, stderr drain). The `in_degree / 100.0` importance heuristic needs validation before claiming 30-50% token savings.

---

## How to add/remove items

1. **Adding:** Open a GitHub issue using the `roadmap` label, then add the item here with status 📋 proposed
2. **Promoting:** When a proposed item is scheduled for a release, change status to 🚧 in-progress and link the issue
3. **Completing:** When shipped, change status to ✅ and move to the Completed section
4. **Removing:** If an item is no longer relevant, move to Deferred with a reason
5. **Pruning:** Every release (every minor version bump), review this document and remove anything that has been 📋 for >2 releases

---

## Tracking

Each item should eventually link to a GitHub issue. Until issues exist, the IDs (R-01, F-19, A-07, etc.) are stable references for discussion in PRs and code comments.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.