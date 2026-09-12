---

## [0.3.0] — 2026-08-04 — IR Evolution: Execution Semantics & Behavioral Reasoning

### Added

#### R-43a: Execution Semantics (Phase 1)
- 4 new `CoreOp` variants in `src/ir/opcodes.rs`:
  - `DataFlow` (`DATAFLOW`) — tracks which symbols a method reads/writes
  - `ControlFlow` (`CTRL`) — control flow constructs (if, loop, match, try, await, return)
  - `SideEffect` (`EFFECT`) — method side-effect type (pure, io, mutation, async, transaction)
  - `ExecutionContext` (`CTX`) — method execution context (sync, async, thread_bound, transaction_scope, realtime)
- Full wire-format support across all 6 encodings: named, positional, binary, hierarchical, string_table, and compact delta abbreviations (DF/CT/EF/CX)
- `primary_key`/`key_tuple`/`primary_key_from_tuple`/`key_tuple_from_tuple` match arms for all 4 new variants
- `SemanticIntent` enum + `intent` field on `IRDelta` — high-level semantic delta metadata
- **Semantic intent detection** in `DeltaComputer::compute()`: rename (class/method/field), add/remove method, change return type, change signature, add injection
- **Compact delta intent preservation** — `CompactDelta` now carries `intent` through encode → decode (previously dropped)
- Language-layer behavioral extraction:
  - Rust: async/unsafe/io → SideEffect + ExecutionContext; match/loop/if/return → ControlFlow
  - C#: IAsyncEnumerable, SignalR Hub, DbSet, SaveChangesAsync, TransactionScope, IDisposable → behavioral ops
  - TypeScript: RxJS subscribe/pipe, async, Observable, @Injectable → behavioral ops
- `IRValidator` behavioral consistency checks (EFFECT("async") ↔ CTX("async"), orphan method refs)

#### R-43b: Program Graph + Inference Layer + Pipeline + Validation + Query (Phases 2-6)
- `src/ir/program_graph.rs` — lightweight local program graph (Calls, Extends, Implements, Injects, DataFlowRead/Write edges)
- `src/ir/inference_layer.rs` — ephemeral inference layer with confidence scores (1.0 structural / 0.75 CBM / 0.5 heuristic)
- `src/ir/pipeline.rs` — composable `IRPass` pipeline (Core → Language → Meta → Execution → Program Graph → Inference → Validation)
- `src/ir/validator.rs` — structural + behavioral invariant validation
- `src/ir/query.rs` — queryable IR (e.g. `find_async_methods`)
- All modules wired into `src/ir/mod.rs` and re-exported

#### R-43b Phase 3: Inference Layer CBM Enrichment
- `InferenceLayer::enrich_from_cbm()` — consumes CBM graph data into the ephemeral inference layer (cross-file CALLS edges, DATAFLOW read/write edges → `inferred_edges`; symbol importance + dead code → `annotations`)
- `GraphBridge::get_call_edges()` — returns `(caller, callee)` pairs for all CALLS relationships (TTL-cached)
- `GraphBridge::get_dataflow_edges()` — returns `(method, target, direction)` triples for DATAFLOW relationships (TTL-cached)
- `InferenceLayerPass` now accepts an optional CBM bridge via `InferenceLayerPass::with_cbm()`, wiring enrichment into Pass 6 of the pipeline
- All CBM-derived data carries `confidence = 0.75` and `source = Cbm` (invariant C3); no-op when CBM is unavailable (invariant C2)
- Mock test helper `new_mock_with_edges()` pre-seeds call/dataflow/importance/dead-code cache entries for deterministic tests

#### R-12: Multi-file / Git-Commit Diff
- New `diff_commits` MCP tool — diffs an entire workspace between two git refs in one call (`fromRef` required, `toRef` defaults to working tree)
- New `src/gitdiff/` module:
  - `refs.rs` — strict ref validation (`^[A-Za-z0-9][A-Za-z0-9._/\-~]*$`, rejects flag injection) + `rev-parse --verify` resolution
  - `runner.rs` — safe `git` subprocess execution with `--end-of-options` (never shell-interpolated)
  - `workspace.rs` — `collect_changed_files` via `git diff --name-status --find-renames` (Added/Deleted/Modified/Renamed classification, path validation)
  - `engine.rs` — `gitdiff_workspace()` orchestrator: per-file AST diff for compressible files (ts/js/cs), line-count fallback for non-compressible, compact skeleton for added files, one-line entry for deleted, rename pairing
- `§GITDIFF <from>..<to> (N files)` header + per-file `┌ FILE αN <path> (+A -D ~M)` sections
- Security: strict ref allowlist, `resolve_file_path_checked` XPIA mitigation, no-shell Command execution, fail-closed structured errors
- Resource limits: changed-file count capped by `resource_limits.max_workspace_files`, per-file size by `resource_limits.max_file_size_bytes` (excess → `_meta.skipped`)
- Tests: 30 `src/gitdiff` unit tests (real temp repos) + black-box e2e dispatch test (`test_e2e_diff_commits`, `#[ignore]`)

### Changed
- `DeltaComputer::compute()` now populates `IRDelta.intent` with detected semantic intent
- `CompactDelta` gained an `intent` field (serde `skip_serializing_if` — absent when `None`)
- `InferenceLayerPass` changed from a unit struct to a struct holding `Mutex<Option<GraphBridge>>`; `new()` still builds an empty layer, `with_cbm()` enables CBM enrichment
- Test count increased with semantic-intent detection tests and compact intent round-trip tests

### Version history
| Version | Date | Highlights |
|---------|------|------------|
| 0.3.0 | 2026-08-04 | **IR Evolution.** Execution semantics (DataFlow/ControlFlow/SideEffect/ExecutionContext), semantic delta intent, program graph, inference layer, pass pipeline, validation, query |

---

## [0.2.1-rc2] — 2026-07-03 — Meta-Layer Expansion & FAANG Hardening

### Added

#### .NET / C# Meta-Layer (R-35)
- Full `dotnet` feature gate with 38 Φ markers mirroring the Angular/Spring architecture
- `src/dotnet_meta/`: `controller.rs`, `service.rs`, `middleware.rs`, `endpoint.rs`, `attribute.rs`, `model.rs`, `mapper.rs`, `config.rs`, `entity.rs`, `program.rs`, `event.rs`, `background.rs`, `filter.rs`, `signalr.rs`, `health.rs`, `cors.rs`, `auth.rs`, `validation.rs`, `logging.rs`, `swagger.rs`, `fluent.rs`, `mediatr.rs`, `efcore.rs`, `serialize.rs`, `metric.rs`, `graphql.rs`, `grpc.rs`, `caching.rs`, `polly.rs`, `detect.rs`, `markers.rs`, `mod.rs`
- `.cs` file extension support in compression pipeline, feature-gated tests

#### Dual Meta-Layer Analysis (R-41/R-42)
- `docs/DUAL_META_LAYER_ANALYSIS.md` — Comprehensive analysis of Angular + Spring Boot + .NET meta-layers with opcode inventory, fidelity tables, and feature-gate audit
- Angular: 24 opcodes (Φcmp, Φsvc, Φmod, Φdir, Φpipe, Φin, Φout, Φmodel, Φinj, Φtpl, Φsty, Φbundle, Φgraph, Φlet, ⊕guard, ⊕sync, $a, $o, $m, $b, $P, $R, Φmap)
- Spring Boot: 38 opcodes (7 primary ⊕ stereotypes + 31 request mapping + config + profile + test + lifecycle + messaging markers)
- .NET: 30 opcodes (28 per-class Φ markers + controller routing + service DI)

#### A-08 Token Efficiency Audit Resolution
- Sliding context window proxy with configurable `CONTEXT_WINDOW_TOKENS` and `SLIDING_WINDOW_OVERLAP`
- Tool output aging: `max_age_seconds` (default 1800s) drops stale tool results outside the retention window
- Token budget enforcement: `target_tokens` soft cap trims oldest assistant-tool pairs that overflow the budget
- Cross-reference path cache: `extract_path_strings` caches extracted paths to avoid re-parsing large tool outputs
- `proxy/tests/audit_regression.rs` — 18 regression tests covering all audit findings

### Fixed

#### Feature-Gate Consistency (P1-9)
- All angular-only imports, types, and functions across 8 files correctly gated behind `#[cfg(feature = "angular")]`:
  - `workspace.rs`: `Arc`, `bundler`, `decorators`, `FooterBuilder`, `GraphCollector`, `template`, `style`, `extract_class_blocks`, `triplet_name`, `PassContextRef`, `format_manifest_footer`
  - `workspace_util.rs`: `format_manifest_footer`, `PassContextRef`, `triplet_name`, `extract_class_blocks`, `find_next_class_keyword`, `find_decorator_start`, angular crate imports
  - `template.rs`: `OnceLock`, `Language`, `Parser`, `DEFAULT_DEPTH`, and all tree-sitter helper functions
  - `decorators.rs`: `inline_template` field and `extract_graph_entries` annotated with `#[allow(dead_code)]`
- Non-angular stub implementations (`FooterBuilder`, `bundle_pass`, `graph_pass`) annotated with `#[allow(dead_code)]` where structurally necessary
- All 1489 tests pass under `--all-features` with zero clippy warnings

#### Clippy Warnings
- `server.rs`: `walk_up_for_project_root` takes `&Path` instead of `&PathBuf` (clippy::ptr_arg)
- `bridge.rs`: unused variables `c` and `status` prefixed with underscore
- `tools.rs`: `inline_tool_names` annotated with `#[allow(dead_code)]`
- `heuristics.rs`: empty line after doc comment merged into preceding comment

### Changed
- Proxy now includes sliding context window transform with configurable token budget and overlap
- Documentation updated: `docs/DUAL_META_LAYER_ANALYSIS.md`, `docs/FAANG_AUDIT_FINDINGS.md`
- Test count: 1,489 tests all passing, 0 clippy warnings

### Version history
| Version | Date | Highlights |
|---------|------|------------|
| 0.2.1-rc2 | 2026-07-03 | **Meta-Layer expansion.** .NET/C# meta-layer, Dual Meta-Layer analysis, A-08 sliding window proxy, P1-9 feature-gate hardening — 1,489 tests, 0 clippy warnings |

---

## [0.2.1-rc1] — 2026-06-30 — Foundation Complete

### Added

#### F-19: Streaming workspace walk (walkdir)
- Replaced recursive `collect_source_files_inner` with `WalkDir::new(root).max_depth(32).follow_links(false)`
- Streaming iteration eliminates collect-then-sort pattern
- Pre-allocates path aliases during single-threaded file-collection step
- Respects `MAX_WALK_DEPTH`, skips hidden dirs/node_modules/target/dist
- Regression test: symlink loop protection + depth limit verification

#### F-20: Rayon parallelization for `compress_workspace`
- Applied `par_iter()` to the per-file compression loop in `compress_pass`
- Shared manifest/errors wrapped in `Mutex` for thread-safe appending
- Pre-assigned aliases (F-21) ensure read-only HashMap lookups in parallel phase
- 38 workspace tests passing

#### F-21: Deterministic alias assignment
- Pre-assigns α1, α2…αN aliases sequentially before the parallel Rayon loop
- Once assigned, `get_or_create_alias` is a read-only lookup (no mutation, safe for concurrent access)
- Prevents non-deterministic aliases caused by thread scheduling variance

#### F-22: Workspace compression result caching
- `WorkspaceCache` stores complete `WorkspaceResult` keyed by content hash of file paths + mtimes/sizes + fidelity
- Cache check at top of `compress_workspace_dir` returns cached result instantly on HIT
- Lazy initialization: cache created on first MISS, stored for future calls
- Thread-safe via `static Mutex<Option<WorkspaceCache>>`
- Regression test: same-fidelity cache HIT + cross-fidelity cache MISS verification

#### A-14: CI/CD awareness
- `is_ci_environment()` detects 7 CI env vars: `CI`, `TF_BUILD`, `GITHUB_ACTIONS`, `GITLAB_CI`, `JENKINS_URL`, `CIRCLECI`, `TRAVIS`
- Auto-disables persistence when CI is detected, preventing stale `persistence.db` between builds

#### A-13: Resource limits and memory guardrails
- `ResourceLimits` struct with `max_file_size_bytes` (10 MB), `max_workspace_files` (10,000), `max_memory_bytes` (512 MB)
- Validation methods return descriptive error messages instead of OOM crashes
- Wired into `compress_workspace_dir` (file count + memory checks) and proxy body buffers

#### A-15: Configuration precedence documentation
- Precedence rules documented in `docs/CONFIGURATION.md`: tool arg > env var > config file > default
- Complete `.clean-ctx.json` example, env var reference, resource limits docs, CI/CD behavior, debug instructions

### Fixed
- F-22 cache key now includes `fidelity` in the hash to prevent cross-fidelity cache collisions
- clippy `single_match` warning in `WorkspaceCache::compute_hash`

### Changed
- Documentation updated: README.md, ARCHITECTURE_OVERVIEW.md, SECURITY.md, ROADMAP.md
- ROADMAP.md reorganized: Foundation section marked ✅ COMPLETE, all 11 FAANG items moved to Completed section
- Test count: 1,512 tests (1,371 unit + 18 audit regression + 1 integration + 123 proxy)
- Clippy: 0 warnings across all targets

---

