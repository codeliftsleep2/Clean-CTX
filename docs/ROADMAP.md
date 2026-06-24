# Clean-CTX — Future Roadmap

**Last updated:** 2026-06-24

> **Living document.** Items are reviewed and pruned every release. Status legend: 📋 proposed · 🚧 in-progress · ✅ done · ⏸️ deferred

---

## At a Glance

| Horizon | Target Release | Theme | Items |
|---------|----------------|-------|------:|
| **Now** | v0.2.0 | Real-world ready | 6 |
| **Foundation** | v0.2.0 | Architectural hardening (blocking) | 5 |
| **Next** | v0.3.0 | Advanced capabilities | 8 |
| **Later** | v1.0.0+ | Ecosystem & integrations | 6 |
| **Architectural** | Continuous | Code health & tooling | 15 |
| **Community** | Continuous | Docs & marketing | 5 |

---

## Completed (v0.1.x – v0.1.8) — shipped ✅

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

---

## Foundation (v0.2.0) — "Architectural hardening (BLOCKING)"

These items are **prerequisites** for enterprise adoption. They address findings from a FAANG-level architectural review. All Foundation items must be complete before v0.2.0 ships. Do NOT begin new feature work (Next list) until these are resolved.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **A-09** | Multi-threaded MCP server dispatch | Convert the single-threaded stdin/stdout loop to a thread-pool model. Each tool call (compress, delta, CBM query) runs on its own thread. A slow CBM query must not block other tools. Use `std::thread::pool` or `rayon` for the dispatch layer. **This is the #1 adoption blocker.** | 2-3 days | 🔴 **P0 Critical** |
| **A-10** | Proxy hardening: auth + rate limiting | Add optional API key header check (`X-Api-Key`) to the proxy. Add per-client rate limiting with configurable RPS and burst windows. Document nginx sidecar pattern for corporate TLS termination. Open HTTP proxy on localhost with no auth is a security finding. | 1 day | 🔴 **P0 Critical** |
| **A-11** | Meta-layer detection hardening | Migrate from `source.contains()` string scanning to proper AST-node matching for Angular/Spring Boot detection. False positives from comments (`// TODO: remove @Component`) or string literals trigger the full meta-layer pipeline incorrectly. This pattern must be fixed BEFORE adding React, Vue, or ASP.NET meta-layers. | 1 day | 🔴 **P0 Critical** |
| **A-12** | Tree-sitter version strategy | Document and implement a migration path for tree-sitter version bumps. Currently all tree-sitter crates are pinned with `=0.20.x` exact versions. This creates a dependency island incompatible with other Rust tools. Add a CI check that alerts on new tree-sitter releases. | 1 day | 🔴 High |
| **A-13** | Resource limits and memory guardrails | Add explicit configuration for: max file size (default: 10 MB), max workspace file count (default: 10,000), max memory usage. Return graceful error messages instead of OOM crashes. Implement in the compression entry points and proxy body buffers. | 1 day | 🔴 High |
| **A-14** | CI/CD awareness | Auto-disable persistence when `CI=true` or `TF_BUILD=true` env vars are detected. Default `.clean-ctx.json` should document this. Prevent stale `persistence.db` from leaking between CI builds. | 0.5 day | 🔴 High |
| **A-15** | Configuration precedence documentation | Two parallel config systems (`.clean-ctx.json` + 12+ env vars for proxy). Document explicit precedence rules: tool arg > env var > config file > default. Add a `--config-dump` flag that prints the resolved configuration. | 0.5 day | 🟡 Medium |

---

## Now (v0.2.0) — "Real-world ready"

These items address the most common user requests and unlock adoption on larger codebases.
**NOTE:** These items may proceed in parallel with Foundation items, but v0.2.0 must NOT ship until all Foundation items are complete.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **F-19** | Streaming workspace walk | Replace `collect_source_files` collect-then-sort pattern with a `walkdir` streaming visitor. Required before F-20. | 1 day | 🟡 Medium |
| **F-20** | Rayon parallelization for `compress_workspace` | Per-thread tree-sitter `Parser` pool, shared `DashMap` for the path dictionary, `par_iter().try_for_each`. Expected ~4× speedup on 16-core boxes. Requires F-19. | 3-5 days | 🔴 High |
| **A-07** | Property-based tests with `proptest` | Fuzz-style input tests for the decompressor, the config glob matcher, and the modifier stripper. Would have caught F-06 (Unicode) and F-12 (substring match) regressions. | 1-2 days | 🔴 High |
| **R-29** | Intelligence Layer | Ranked context delivery on top of the existing compression stack. **Phase 1 (complete):** PageRank symbol scoring + CBM-informed adaptive per-symbol fidelity (60% IR + 40% CBM blend). **Phase 2 (complete):** Blast radius integrated into `handle_provide_code_context` and `handle_delta_code_context` — depth-1 affected files compressed at Low fidelity and appended with `§IMPACT` markers. Regression tests added. **Phase 3 (not started):** Token budget knapsack packing. All phases opt-in via `.clean-ctx.json`, zero overhead when disabled. See `docs/INTELLIGENCE_LAYER_PLAN.md`. | 5.5 days | 🔴 High |
| **A-08** | TOKEN_EFFICIENCY_AUDIT findings | 4 open findings: underutilized `source_cache` (High), double IR compile in delta path (Medium), path resolution inconsistency (Medium), fragile source ownership (Low). | 1-2 days | 🟡 Medium |
| **A-04** | Observability upgrade (was Low) | **Upgraded from Low to Medium priority.** Add OpenTelemetry-compatible structured logging (`tracing` + `metrics`) with OTLP export. Export key metrics: compression latency, delta hit rate, cache efficiency, CBM query latency, error rates by category. This is required before any production deployment. | 2-3 days | 🟡 Medium |

---

## Next (v0.3.0) — "Advanced capabilities"

**GATE:** All Foundation items (A-09 through A-14) must be complete before v0.3.0 work begins. Meta-layer items (R-36, R-37, R-23, R-24) additionally depend on A-11 (detection hardening).

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-01** | Python language layer | Most-requested language. Follows the 4-step guide in `DEVELOPER_DOCUMENTATION.md`. | 1-2 days | 🔴 High |
| **R-02** | Type-aware compression | Inline `type_aliases` from config: `UserId` → `$uid`, `JsonObject` → `$jo`. Currently the type table is loaded but not injected into the capture pipeline. | 2-3 days | 🔴 High |
| **R-12** | Multi-file / git-commit diff | Diff an entire workspace between two git commits; emit per-file deltas in one tool call. Powers "what changed in this PR?" workflows. | 3-5 days | 🔴 High |
| **R-36** | React Meta-Layer | Additive meta-layer on TS/JS. Component/hook/context bundling, prop type compression, React-specific lifecycle markers. **BLOCKED by A-11.** | 3-4 days | 🔴 High |
| **R-37** | Redux Meta-Layer | Additive meta-layer on TS/JS. Action/reducer/selector compression, thunk/saga patterns, store shape compression. **BLOCKED by A-11.** | 2-3 days | 🔴 High |
| **R-23** | NgRx Meta-Layer | Framework-annotation layer for NgRx state management (sits on top of TS + Angular layers). **BLOCKED by A-11.** | 3-4 days | 🔴 High |
| **R-01b** | Go language layer | Second-most requested. | 1-2 days | 🟡 Medium |
| **R-07** | MCP `resources` support | Expose compressed snapshots as MCP resources in addition to tools, enabling LLM clients to read prior state without re-invoking tools. | 1-2 days | 🟡 Medium |
| **R-08** | Improved diff: rename detection | Detect class/method renames (same signature, different name) and emit as `~` with a `renamed from X` hint instead of a delete+add pair. | 1 day | 🟡 Medium |

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
| **A-02** | Migrate to `walkdir` (precondition for F-20) | Required before parallelism can be added — streaming input is a prerequisite for `par_iter`. | 1 day | 🟡 Medium |
| **A-03** | `schemars` for tool schemas | Replaces hand-written JSON Schema in `src/mcp/tools.rs`; less drift between Rust types and advertised schema. | 1-2 days | 🟡 Medium |
| **A-04** | `tracing` + `metrics` | Structured logging + OpenTelemetry-compatible spans; useful for diagnosing slow compressions in production. **Upgraded to Medium priority — see Now list.** | 2-3 days | 🟡 Medium |
| **A-05** | Workspace-aware language detection | When `compress_workspace` runs, cache language detection results across files. TS file next to another TS file is, predictably, TS. | 1 day | 🟢 Low |
| **A-07** | Property-based tests with `proptest` | See Now list. Fuzz-style input tests for decompressor, config glob matcher, modifier stripper. | 1-2 days | 🔴 High |
| **A-08** | TOKEN_EFFICIENCY_AUDIT findings | 4 open findings: underutilized `source_cache` (High), double IR compile in delta path (Medium), path resolution inconsistency (Medium), fragile source ownership (Low). See `docs/TOKEN_EFFICIENCY_AUDIT.md`. | 1-2 days | 🟡 Medium |
| **A-09** | Multi-threaded MCP server dispatch | See Foundation list. **P0 Critical — #1 adoption blocker.** | 2-3 days | 🔴 **P0 Critical** |
| **A-10** | Proxy hardening: auth + rate limiting | See Foundation list. **P0 Critical — security finding.** | 1 day | 🔴 **P0 Critical** |
| **A-11** | Meta-layer detection hardening | See Foundation list. **P0 Critical — blocks all new meta-layers.** | 1 day | 🔴 **P0 Critical** |
| **A-12** | Tree-sitter version strategy | See Foundation list. Document migration path, add CI alert on new releases. | 1 day | 🔴 High |
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
| **F-19** | Streaming workspace walk | See Now list. | 📋 Proposed |
| **F-20** | Rayon parallelization | See Now list. | 📋 Proposed |

---

## Prioritization rationale

### Foundation-first ordering (v0.2.0)

The Foundation items must ship FIRST because they are **blocking prerequisites** for enterprise adoption. They were identified by a FAANG-level architectural review and directly address:

1. **A-09 (Multi-threaded MCP server)** — #1 adoption blocker. A single slow CBM query currently blocks ALL tools. Until this is fixed, the server cannot be used in production with CBM enabled, and even without CBM, a large file compression blocks all other concurrent requests.

2. **A-10 (Proxy auth + rate limiting)** — #2 adoption blocker. Security teams will flag an open HTTP proxy on localhost that forwards to Anthropic/OpenAI. Any process on the machine can use it. Adding `X-Api-Key` auth and rate limiting closes this finding.

3. **A-11 (Meta-layer detection hardening)** — #3 adoption blocker. The current `source.contains()` pattern produces false positives (comments, string literals). This must be hardened BEFORE adding React, Redux, NgRx, Vue, or ASP.NET meta-layers — otherwise every new meta-layer inherits the same fragile detection.

4. **A-12 (Tree-sitter version strategy)** — Supply-chain risk. Pinned `=0.20.x` versions create a dependency island. Must document a migration path before v1.0.0.

5. **A-13 (Resource limits)** — Production safety. Without documented max file size or memory limits, a single large file can OOM the server.

6. **A-14 (CI/CD awareness)** — CI adoption blocker. Default persistence writes `.clean-ctx/persistence.db` to project root, which leaks stale state between CI builds.

7. **A-15 (Configuration precedence)** — Operational sanity. Two parallel config systems with no documented precedence creates user confusion and debugging difficulty.

### Now list priorities (v0.2.0)

These items may proceed in parallel with Foundation items, but v0.2.0 must NOT ship until Foundation is complete:

1. **F-19 → F-20 (walkdir + Rayon)** — Streamlined workspace handling and parallel compilation. Builds on the multi-threaded dispatch from A-09.
2. **A-07 (proptest)** — Regression insurance against input-validation bugs. Cheap to implement, prevents regressions.
3. **R-29 (Intelligence Layer)** — Phase 1 (PageRank) + Phase 2 (blast radius) complete. Phase 3 (token budget packing) not started.
4. **A-08 (Token Efficiency Audit)** — 4 open findings to resolve before claiming production-readiness.
5. **A-04 (Observability)** — **Upgraded from Low to Medium priority.** OpenTelemetry-compatible tracing/metrics required before any production deployment.

### Next list priorities (v0.3.0)

GATED by Foundation completion. Meta-layer items additionally gated by A-11:

| Priority | Item | Dependency |
|----------|------|------------|
| 🔴 High | R-01 Python language layer | None |
| 🔴 High | R-02 Type-aware compression | None |
| 🔴 High | R-12 Multi-file git-commit diff | A-09 (multi-threaded server) |
| 🔴 High | R-36 React Meta-Layer | **A-11 (detection hardening)** |
| 🔴 High | R-37 Redux Meta-Layer | **A-11 (detection hardening)** |
| 🔴 High | R-23 NgRx Meta-Layer | **A-11 (detection hardening)** |
| 🟡 Medium | R-01b Go language layer | None |
| 🟡 Medium | R-07 MCP resources support | None |
| 🟡 Medium | R-08 Improved diff: rename detection | None |

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