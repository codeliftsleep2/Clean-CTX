# Clean-CTX — Future Roadmap

**Last updated:** 2026-08-24 (CBM graph-intelligence audits ✅ — graph-intel live-probe pass + typed graph_trace wire-contract fix, both recorded in Completed below)

> **Living document.** Items are reviewed and pruned every release. Status legend: 📋 proposed · 🚧 in-progress · ✅ done · ⏸️ deferred

---

## At a Glance

| Horizon | Target Release | Theme | Items |
|---------|----------------|-------|------:|
| **Now** | v0.3.0 | Advanced capabilities | ✅ 0 (all complete) |
| **Next** | v0.4.0 | Advanced capabilities | 6 |
| **Later** | v1.0.0+ | Ecosystem & integrations | 6 |
| **Architectural** | Continuous | Code health & tooling | 3 (A-01, A-03, A-05) |
| **Community** | Continuous | Docs & marketing | 5 |

---

## Completed (v0.1.x – v0.3.0) — shipped ✅

These items are complete and documented. Listed for historical context.

| ID | Title | Shipped in | Notes |
|----|-------|-----------|-------|
| **R-02** | Type-aware compression | v0.3.0 | Inline `type_aliases` applied across both text + IR compression paths. Text path: configured type names replaced with `$alias` tokens + `§TA $uid→UserId` footer. IR path: `CoreOp::TypeAlias(alias, original)` ops appended. Token-boundary matching (`$` treated as identifier char) prevents false matches in `UserService`/`user_id`. 5-15% additional savings on type-heavy files at Medium/High fidelity. See `docs/plans/TYPE_AWARE_COMPRESSION_PLAN.md`. |
| **R-12** | Multi-file / Git-Commit Diff | v0.3.0 | New `diff_commits` MCP tool + `src/gitdiff/` module (refs, runner, workspace, engine). Per-file AST change-sets across an entire workspace between two git refs. Strict ref allowlist + `--end-of-options` + XPIA mitigation. Resource limits (file count + per-file size). 32 unit tests + black-box e2e dispatch test. See `docs/DIFF_COMMITS_GUIDE.md`. |
| **R-22** | Angular Meta-Layer | v0.1.x | Phase 1 (decorators) ✅ · Phase 2 (triplet bundling) ✅ · Phase 2.5 (Angular 17–21 syntax) ✅ · Phase 3 (cross-file DI + selector graph) ✅ · Phase 4 (fidelity-gated template compression) ✅ |
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
| **R-35 (P2)** | CBM Phase 2 — Intelligence Layer seeding | v0.1.9 | Fixed CBM client API mismatches (`search_graph`/`trace_path` param names, replaced non-existent `get_symbol_importance`/`get_dead_code` with Cypher queries). Intelligence Layer (pagerank.rs, fidelity.rs) now correctly blends CBM cross-file `in_degree` scores (60% IR + 40% CBM) for adaptive per-symbol fidelity. `enrich_with_cbm` in tool_handlers injects compressed CBM metadata into responses. |
| **Compiler-IR Audit** | Compiler-IR audit + clippy cleanup | v0.1.8 | Verified all 8 phases (A–H). Resolved 29 clippy warnings across entire build. Rewrote COMPILER_IR.md from spec to implementation docs. All 1,277 tests pass with 0 clippy warnings. |
| **CBM Graph-Intel Audit** | Graph-intelligence live audit fixes | Unreleased | 4 findings resolved via fresh-process/fresh-index live probes: HIGH blast-radius Cypher fail-open (`m.name` -> `f.name`), MED silent tool-failure conflation (`CbmError::ToolError` + `Result` propagation: `Ok(empty)` = zero results, `Err` = CBM failure), MED dead code now covers Function + Method labels, MED dead DATAFLOW path removed (no such edge type in CBM 0.8.1). New 9-probe audit suite `src/tests/cbm/graph_intel.rs`. See `docs/ARCHITECTURE_OVERVIEW.md` (CBM Compatibility & Verified Limitations). |
| **CBM Trace-Wire Audit** | Typed graph_trace wire-contract fixes | Unreleased | 2 defects + boundary normalization resolved via verbatim raw-capture fixtures and fresh-process live probes over a synthetic fixture repo: HIGH typed `graph_trace` parsed a phantom `inner["edges"]` key — CBM 0.8.1 answers with directional `callers`/`callees` arrays keyed by `name`/`qualified_name`/`hop`, so every typed trace silently collapsed to zero edges while the raw proxy path worked; DIRECTION determination hardcoded outbound whenever both endpoints were supplied, making inbound-only relationships undiscoverable (outbound-first behavior preserved byte-for-byte; single inbound fallback fires only on success-but-filtered-empty); BOUNDARY normalization — M-01 target predicate matches exact-qualified OR final-dot-segment against canonical endpoints, so a bare-to name retains edges whose wire endpoints are qualified. hop>1 entries are flat BFS discoveries without parent linkage and never convert into invented edges. New trace_wire.rs suite: 16 tests (verbatim capture pins + synthetic policy pins + 4 fresh-process `serial(cbm_live)` probes); workspace now 2,497 passed / 0 failed / 5 ignored. Normative invariant `CBM-WIRE-001` in `docs/ARCHITECTURAL_INVARIANTS.md`. See `docs/CHANGELOG.md` and commit `193f885`. |
| **A-09** | Multi-threaded MCP server dispatch | v0.2.0-rc1 | ✅ Production-grade `Dispatcher` with `crossbeam_channel` bounded queue + backpressure, `RwLock`-protected state for parallel reads, `catch_unwind` panic recovery, dedicated stdout writer thread, request tracing/observability, configurable worker count (auto-detect CPU count). Includes 6+ unit tests covering spawn lifecycle, concurrent mutations, panic recovery, and tracing. **Formerly P0 Critical — #1 adoption blocker.** See `src/mcp/dispatcher.rs`. |
| **A-10** | Proxy hardening: auth + rate limiting | v0.2.0-rc1 | ✅ Optional `X-Api-Key` header authentication via `PROXY_API_KEY` env var. Per-client-IP token bucket rate limiter (configurable `RATE_LIMIT_RPS`/`RATE_LIMIT_BURST`, default 60/10). Returns `401 Unauthorized` for bad/missing keys, `429 Too Many Requests` when rate limited. Rate limiter uses GC-enabled sliding window to prevent unbounded map growth. Stats endpoint exposes rate limiter status. Nginx sidecar pattern documented in `docs/PROXY.md`. 141 proxy tests passing (6 new rate limiter tests + 5 new server tests). |
| **A-12** | Tree-sitter version migration | v0.2.0-rc1 | ✅ Migrated from `=0.20.x` pinned versions to `^0.26.x` semver ranges. All 6 tree-sitter crates updated (`tree-sitter` 0.26.10, `c-sharp` 0.23.5, `typescript` 0.23.2, `html` 0.23.2, `rust` 0.24.2, `java` 0.23.5). Parser API migrated (LanguageFn → Language, `&Language` borrows, `StreamingIterator` for QueryMatches). CI guard (`scripts/check-tree-sitter-versions.ps1`) ensures all grammars share the same `tree-sitter-language` ABI. **Unblocks F-20 (Parser is now `Send`).** |
| **A-11** | Meta-layer detection hardening | v0.2.0 | ✅ Migrated from `source.contains()` string scanning to proper AST-node matching using tree-sitter queries for Angular/Spring Boot detection. Eliminates false positives from comments and string literals. |
| **A-13** | Resource limits and memory guardrails | v0.2.0 | ✅ Added `ResourceLimits` config with max file size (10 MB default), max workspace file count (10,000 default), max memory usage. Graceful error messages instead of OOM crashes. Wired into compression entry points (`compress_code_context`, `compress_workspace_dir`, `delta_code_context`) and proxy body buffers. |
| **A-14** | CI/CD awareness | v0.2.0 | ✅ Auto-disables persistence when `CI=true` or `TF_BUILD=true` env vars are detected. Documented in default `.clean-ctx.json`. Prevents stale `persistence.db` from leaking between CI builds. |
| **A-15** | Configuration precedence documentation | v0.2.0 | ✅ Documented explicit precedence rules (tool arg > env var > config file > default) in `docs/CONFIGURATION.md`. Includes complete `.clean-ctx.json` example, env var reference, resource limits docs, CI/CD behavior, debug instructions. |
| **A-08** | Token Efficiency Audit | v0.2.0 | ✅ All 4 findings resolved: source_cache integration (Finding 1 High ✅), source hash tracking in delta path (Finding 2 Medium ✅), path resolution consistency confirmed (Finding 3 Medium ✅), source ownership verified sound (Finding 4 Low ✅). |
| **A-04** | `tracing` + `metrics` | v0.2.0 | ✅ Structured logging with `tracing` crate, OpenTelemetry-compatible spans, `MetricsRegistry` with counters/histograms/gauges, OTLP-exportable design. `init_tracing()` at server startup. See `src/observability/`. |
| **A-07** | Property-based tests with `proptest` | v0.2.0 | ✅ Fuzz-style input tests for decompressor (`word_boundary_replace`), config glob matcher (`is_excluded`), modifier stripper (`strip_modifiers`). 3 proptest modules in `src/tests/proptest/`. |
| **R-41** | Sliding Context Window (Tier 1) | v0.2.0 | ✅ Age-based tool-result truncation in the proxy pipeline, with force-preserve rules (floor + path cross-reference). Opt-in via `SLIDING_WINDOW=1`. `age_tool_results()` in `proxy/src/transform.rs`. |
| **R-42** | Sliding Context Window (Tier 2 — scored pruning) | v0.2.0 | ✅ Relevance-scored pruning beyond simple age. Dry-run mode support. Integrated with proxy dashboard stats. |
| **F-19** | Streaming workspace walk | v0.2.0 | ✅ Replaced recursive `collect_source_files_inner` with `walkdir` streaming visitor. Pre-allocates path aliases during single-threaded file-collection step for deterministic `αN` numbering. Required before F-20. |
| **F-20** | Rayon parallelization for `compress_workspace` | v0.2.0 | ✅ Applied `par_iter()` to the per-file compression loop with Mutex-wrapped manifest/errors collectors. Pre-assigns aliases deterministically (F-21) before parallel work. **Prerequisites:** F-19 (walkdir) ✅, A-12 (Parser `Send`) ✅. |
| **F-21** | Deterministic alias assignment | v0.2.0 | ✅ Pre-assigns α1, α2…αN aliases sequentially before the parallel Rayon loop. Once assigned, `get_or_create_alias` is a read-only HashMap lookup safe for concurrent access. |
| **F-22** | Workspace compression result caching | v0.2.0 | ✅ Caches the complete `WorkspaceResult` keyed by a content hash of file paths + mtimes/sizes. Subsequent calls with no file changes return the cached result instantly. Saves 5-15s per redundant call for a 100-file workspace. |
| **R-43a** | IR Evolution — Execution Semantics (Phase 1) | v0.3.0 | ✅ 4 new `CoreOp` variants (DataFlow, ControlFlow, SideEffect, ExecutionContext) for behavioral reasoning. Full wire-format support (named/positional/binary/hierarchical/string_table/compact). `SemanticIntent` delta metadata with detection in `DeltaComputer::compute()` (rename/add/remove method, change return type/signature, add injection). Compact delta intent preservation. Rust/C#/TypeScript language-layer behavioral extraction. `IRValidator` behavioral consistency checks. See `docs/COMPILER_IR.md`. |
| **R-43b** | IR Evolution — Program Graph + Inference Layer + Semantic Delta + Validation + Query (Phases 2-6) | v0.3.0 | ✅ `ProgramGraph` (local graph), `InferenceLayer` (confidence-scored ephemeral overlay), `PassPipeline` (composable `IRPass` chain), `IRValidator` (structural + behavioral invariants), `IRQueryEngine` (queryable IR). All wired into `src/ir/mod.rs`. See `docs/COMPILER_IR.md`. **Phase 3 CBM enrichment ✅:** `InferenceLayer::enrich_from_cbm()` consumes cross-file CALLS edges + importance/dead-code annotations (confidence 0.75); `GraphBridge::get_call_edges()` added; `InferenceLayerPass::with_cbm()` wires enrichment into the pipeline. Dataflow enrichment is unavailable - CBM 0.8.1 exposes no DATAFLOW edge type (the originally added `get_dataflow_edges()` was removed in the 2026-08-24 graph-intelligence audit). See `docs/CHANGELOG.md` [0.3.0]. |
| **R-44** | Angular HTML Template Compression | v0.3.0 | ✅ Fidelity-gated template compression for Angular `.component.html` files. New `template_compress.rs` module with `compress_template()` / `compress_template_with_prime_ng()`. `TemplateShape::to_marker_lines(fidelity)` produces Low (single-line), Medium (multi-line structural), High (near-full) output. `PhiLineKind` extended with `TemplateBinding`/`TemplateDirective`/`TemplateComponent`. GitDiff routes `.component.html` through the compressor (AST-level change-sets). Heuristics classify `.component.html` as Implementation/Medium, upgrade to High on `intent="edit"`. `provide_code_context` routes `.component.html` through the compressor with DB persistence. PrimeNG `Φp-<name>:` markers. Post-implementation FAANG audit fixed a word-boundary bug (`@if`/`@for` in string literals) and a persistence gap. |
| **R-23** | NgRx Meta-Layer | v0.4.0 | ✅ Shipped 2026-08-11. `src/angular_meta/ngrx.rs` — actions, reducers (incl. inline `createReducer` in `createFeature`), effects, selectors, entity adapters, NgRx Data `EntityCollectionServiceBase<T>` (`Φentity:T (data-layer)`), `{ dispatch:false }`, Store DI, dispatch/select sites. Namespaced `NgRxKind` (`Φngrx:`/`Φaction:`/`Φreducer:`/`Φeffect:`/`Φselector:`/`Φentity:`/`Φstore:`/`Φdispatch:`/`Φselect:`). |
| **R-24** | RxJS Meta-Layer | v0.4.0 | ✅ Shipped 2026-08-11. `src/angular_meta/rx.rs` — observable fields, subject instantiations, pipe chains (`ΦpipeRx:`), static combinators, creation functions. Namespaced `RxJsKind` (`Φobs:`/`Φsubject:`/`ΦpipeRx:`/`Φmap:`/`Φtap:`/`Φfilter:`/`Φcatch:`/`Φfinalize:`/`Φdelay:`/`Φcombine:`/`Φshare:`/`Φto:`/`Φwith:`/`Φscan:`/`Φdistinct:`/`Φretry:`). |
| **R-25** | Angular Ecosystem Deepening (Signals + Routing + cross-layer graph) | v0.4.0 | ✅ Shipped 2026-08-11. Signals (`src/angular_meta/signals.rs` — `signal()`/`computed()`/`effect()`/`toSignal()`/`toObservable()`/`linkedSignal()`, `Φsignal:`/`Φcomputed:`/`Φsig-effect:`/`ΦtoSignal:`/`ΦtoObservable:`/`ΦlinkedSignal:`) + Routing (`src/angular_meta/routing.rs` — `Routes` arrays, `RouterModule.forRoot/forChild`, lazy loads, guards, resolvers; `Φroute:`/`Φguard:`/`Φresolver:`). Cross-layer NgRx graph edges (`Action→Reducer`, `Action→Effect`, `Effect→Service`, `Effect→Action`, `Component→Store`, `Component→Selector`) via `NgRxEdgeKind`. Hardened through Round-5 → Round-11 FAANG audits (string/depth-aware scans, comment/string-aware extraction guards, type-annotated names, false-positive elimination). 2,263 tests passing, 0 clippy warnings. See `docs/ANGULAR_ECOSYSTEM_DEEPENING.md`. |

---

## Now (v0.3.0) — "Advanced capabilities" ✅ ALL COMPLETE

**ALL ITEMS COMPLETE.** v0.3.0 is fully shipped.

All Foundation items (A-09 through A-15), all Now items (F-19 through F-22, A-08), all Next items implemented in this cycle (R-41, R-42, R-43a, R-43b, R-44), and the IR Evolution are resolved. No remaining blockers.

---

## Next (v0.4.0) — "Advanced capabilities"

**GATE:** All Foundation (A-09 through A-15), Now (A-08, F-19 through F-22), Sliding Context Window (R-41, R-42), IR Evolution (R-43a, R-43b, R-44), and the Angular Ecosystem Deepening (R-23, R-24, R-25) items are complete. v0.4.0 work can begin. The NgRx/RxJS/Signals/Routing meta-layers shipped through Round-11 audits.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-07** | MCP `resources` support | Expose compressed snapshots as MCP resources in addition to tools, enabling LLM clients to read prior state without re-invoking tools. | 1-2 days | 🟡 Medium |
| **R-08** | Improved diff: rename detection | Detect class/method renames (same signature, different name) and emit as `~` with a `renamed from X` hint instead of a delete+add pair. | 1 day | 🟡 Medium |
| **R-01** | Python language layer | Most-requested language. Follows the 4-step guide in `DEVELOPER_DOCUMENTATION.md`. | 1-2 days | 🟡 Medium |
| **R-36** | React Meta-Layer | Additive meta-layer on TS/JS. Component/hook/context bundling, prop type compression, React-specific lifecycle markers. **A-11 ✅ (complete)** — now unblocked. | 3-4 days | 🟡 Medium |
| **R-37** | Redux Meta-Layer | Additive meta-layer on TS/JS. Action/reducer/selector compression, thunk/saga patterns, store shape compression. **A-11 ✅ (complete)** — now unblocked. | 2-3 days | 🟡 Medium |
| **R-01b** | Go language layer | Second-most requested. | 1-2 days | 🟡 Medium |

---

## Later (v1.0.0+) — "Ecosystem & integrations"

Items that add value but require demand signal before investing. **YAGNI applies** — these should not be built speculatively.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-26** | Vue Meta-Layer | Additive meta-layer on TS/JS. Single-file component bundling (`.vue` = script + template + style), Composition API markers. | 3-4 days | 🟡 Medium |
| **R-27** | ASP.NET Meta-Layer | Additive meta-layer on C# layer. Controller/service/repository bundling, DI registration graph (`services.AddScoped<IFoo, Foo>()`), route markers. | 3-4 days | 🟡 Medium |
| **R-28** | Entity Framework Meta-Layer | Additive meta-layer on C# (sits on top of ASP.NET layer). Entity model compression, DbContext graph, migration markers. | 2-3 days | 🟢 Low |
| **R-06** | Config hot-reload | File-watcher on `.clean-ctx.json`; debounce + atomic reload of `McpState.config` without restart. | 1 day | 🟡 Medium |
| **R-10** | Tool input validation via `schemars` | Replace hand-rolled `args.get("foo")` chains with `schemars`-derived JSON Schema; tool schemas in `tools.rs` auto-generate from Rust types. | 1-2 days | 🟡 Medium |
| **R-18** | Markdown / JSON / YAML compression | Extend compression to data formats using the same opcode framework with format-specific markers. | 1-2 days | 🟡 Medium |

---

## Architectural improvements (continuous)

Code health work that has no user-facing feature but improves long-term maintainability.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **A-01** | Remove `src/helpers.rs` shim | The 18-line re-export shim should be removed once all internal callers are confirmed to use `crate::compaction::*`. | 1 hour | 🟢 Low |
| **A-03** | `schemars` for tool schemas | Replaces hand-written JSON Schema in `src/mcp/tools.rs`; less drift between Rust types and advertised schema. | 1-2 days | 🟡 Medium |
| **A-05** | Workspace-aware language detection | When `compress_workspace` runs, cache language detection results across files. TS file next to another TS file is, predictably, TS. | 1 day | 🟢 Low |

---

## Documentation & community (continuous)

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **D-01** | README screenshot / GIF | Top of README showing side-by-side raw vs compressed output for a real class. Increases GitHub star rate. Especially important now with LinkedIn post driving traffic. | 1 hour | 🔴 High |
| **D-02** | Blog post: "Why we wrote a tree-sitter-based LLM compressor" | Marketing + technical deep-dive; good for HackerNews launch and SEO. | 1 day | 🟡 Medium |
| **D-03** | Cross-fidelity benchmark table in README | 50-edit simulation results (96.6% Low / 92.0% Medium / 89.9% High delta savings) already exist in `docs/PERFORMANCE.md` — surface the headline table in README where it's visible on first load. | 2 hours | 🔴 High |
| **D-04** | Competitor comparison table in README | ForgeIndex vs LeanCTX vs Clean-CTX capability matrix. Draft exists in `docs/plans/INTELLIGENCE_LAYER_PLAN.md`. | 1 hour | 🟡 Medium |
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
| **A-02** | Migrate to `walkdir` | Absorbed into F-19 scope (completed). |

---

## Prioritization rationale

### Foundation-first ordering (v0.2.0)

**✅ ALL FOUNDATION AND NOW ITEMS COMPLETE.** v0.2.0 has no remaining blockers.

All items identified by the FAANG-level architectural review plus the Token Efficiency Audit are now resolved:

| ID | Title | Shipped in | Summary |
|----|-------|-----------|---------|
| **A-09** | Multi-threaded MCP server dispatch | v0.2.0-rc1 | Production-grade `Dispatcher` with `crossbeam_channel` bounded queue + backpressure, `RwLock`-protected state, `catch_unwind` panic recovery, dedicated stdout writer thread, request tracing. |
| **A-10** | Proxy hardening: auth + rate limiting | v0.2.0-rc1 | Optional `X-Api-Key` header auth via `PROXY_API_KEY`. Per-client-IP token bucket rate limiter (configurable RPS/burst). 141 proxy tests. |
| **A-11** | Meta-layer detection hardening | v0.2.0 | Migrated from `source.contains()` to AST-node matching using tree-sitter queries. Eliminates false positives from comments/string literals. |
| **A-12** | Tree-sitter version migration | v0.2.0-rc1 | Migrated from `=0.20.x` pins to `^0.26.x`. All 6 crates updated. `Parser` is now `Send`. CI guard in place. |
| **A-13** | Resource limits and memory guardrails | v0.2.0 | `ResourceLimits` config with max file size (10 MB), max workspace file count (10,000), max memory usage. Graceful errors. |
| **A-14** | CI/CD awareness | v0.2.0 | Auto-disables persistence when `CI=true` or `TF_BUILD=true`. Documented in default config. |
| **A-15** | Configuration precedence documentation | v0.2.0 | Precedence rules documented in `docs/CONFIGURATION.md`. Tool arg > env var > config file > default. |
| **A-08** | Token Efficiency Audit | v0.2.0 | All 4 findings resolved. Source hash tracking eliminates recompilation for unchanged files (30-50% savings on repeated delta calls). |
| **A-04** | `tracing` + `metrics` | v0.2.0 | Structured logging with `tracing` crate, `MetricsRegistry` with counters/histograms/gauges, OTLP-exportable design. |
| **A-07** | Property-based tests with `proptest` | v0.2.0 | Fuzz-style input tests for decompressor, config glob matcher, modifier stripper. 3 proptest modules. |
| **R-41** | Sliding Context Window (Tier 1) | v0.2.0 | Age-based tool-result truncation with force-preserve rules. Opt-in via `SLIDING_WINDOW=1`. |
| **R-42** | Sliding Context Window (Tier 2) | v0.2.0 | Relevance-scored pruning with dry-run mode. Integrated with proxy dashboard. |

### Next list priorities (v0.4.0)

All gates clear. Meta-layer items (R-23, R-24, R-25) are now complete and shipped through Round-11 audits. Sliding Context Window (R-41, R-42) and IR Evolution (R-43a, R-43b, R-44) completed in v0.3.0.

| Priority | Item | Dependency |
|----------|------|------------|
| 🟡 Medium | R-07 MCP resources support | None |
| 🟡 Medium | R-08 Improved diff: rename detection | None |
| 🟡 Medium | R-01 Python language layer | None |
| 🟡 Medium | R-36 React Meta-Layer | A-11 ✅ |
| 🟡 Medium | R-37 Redux Meta-Layer | A-11 ✅ |
| 🟡 Medium | R-01b Go language layer | None |

### Items explicitly deferred from Next list

The following items that were previously in the Next list have been deferred to Later (v1.0.0+) to focus on Foundation + critical features:

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