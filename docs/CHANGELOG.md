# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.4.0] - 2026-08-24 - CBM Graph-Intelligence & Trace-Wire Audits

### Fixed

- **F1 (HIGH) - blast radius returned every CALLS edge in the project.** The `get_blast_radius()` Cypher filtered on an undeclared variable (`m.name`). CBM fail-opens on invalid WHERE clauses - it returns the full row set instead of erroring - so every symbol appeared to touch every file. The query now filters on `f.name`, and a live regression test pins reported caller-files to the exact ground-truth caller set computed independently on the same client.
- **F3 (MED) - dead-code detection ignored class Methods.** Only `:Function` nodes were scanned, leaving dead Methods (the majority of TS/C#/Java symbols) invisible. `get_dead_code()` now scans `Function` AND `Method` labels and merges the results; a live test asserts exact set-equality against the merged two-label ground truth.
- **F11 (MED) - CBM tool failures surfaced as confident empty results.** CBM signals tool failures inside successful JSON-RPC results (`result.isError=true` plus an inner error body). These envelopes were never mapped to `CbmError`. Added `CbmError::ToolError { tool, message }` plus a pure `check_soft_error()` gate in the parsed transport path.
- **F10 (MED) - removed the dead DATAFLOW enrichment path.** CBM 0.8.1 exposes no `DATAFLOW` edge type, and its `USAGE`/`WRITES` edges are not equivalents (no read direction). Documented as a verified upstream limitation, with a reintroduction guard that fails if a future CBM version ever exposes DATAFLOW edges.
- **Trace-wire (HIGH) - typed `graph_trace` always returned zero edges.** `CbmClient::trace_path()` parsed a phantom `inner["edges"]` key. CBM 0.8.1 answers `trace_path` with directional `callers` / `callees` arrays whose entries carry exactly `name`, `qualified_name`, and `hop` (a JSON number); there is no `edges` key. New pure `extract_trace_edges()` normalizes the real shape into `{from, to, label}` edge objects: endpoints canonicalize `qualified_name` → bare `name`, only `hop: 1` entries convert (deeper hops are flat BFS discoveries without parent linkage and are never turned into invented relationships), exact duplicate edges dedupe preserving first-seen CBM order, and `__file__` module pseudo-callers pass through untouched. Function-not-found soft errors surface as `Err(CbmError::ToolError)` before parsing (F11 gate) - failure is never a valid empty result.
- **Trace-direction - inbound-only relationships were undiscoverable.** `GraphBridge::trace_path(from, to)` hardcoded outbound whenever both endpoints were supplied. Outbound is now attempted first (byte-for-byte pre-fix behavior for outbound-reachable pairs; no-target calls still sweep `both`); when the outbound attempt succeeds but yields no edge touching the target, a single inbound fallback discovers callee ← caller relationships. Errors are never swapped for the other direction.

### Added

- **Result-propagating graph-intelligence APIs.** All five bridge queries (`get_symbol_importance_mut()`, `get_blast_radius()`, `get_dead_code()`, `get_call_edges()`, `get_architecture()`) return `Result<_, CbmError>`. Contract: `Ok(empty)` = a valid query with zero results; `Err` = CBM failure (transport fault, timeout, open circuit, or CBM-reported tool error). `InferenceLayer::enrich_from_cbm()` propagates `Err` instead of converting failure into empty data; `InferenceLayerPass` owns the failure policy (log loudly, continue without enrichment - CBM stays strictly additive); `handle_get_architecture()` returns an explicit error response.
- **Live audit suite** `src/tests/cbm/graph_intel.rs` - 9 probes (serial `cbm_live`), each spawning a fresh CBM subprocess and re-indexing first (fresh process, fresh index): blast-radius truth set, unknown-project `Err` vs valid `Ok(empty)`, dual-label dead-code equality, DATAFLOW absence guard, wire-shape pins (`in_degree` cells arrive as JSON strings), deterministic soft-error fixtures, architecture parsing, and disk-cache project isolation across project switches.
- **CBM compatibility/limitations documentation** verified live against 0.8.1: supported node labels and edge types, absent DATAFLOW edge type, absent Razor nodes, aggregation-free Cypher subset, fail-open behavior on invalid WHERE clauses, and wire quirks.
- **Wire-contract regression suite** `src/tests/cbm/trace_wire.rs` - 16 tests registered in both mod mirrors: verbatim raw captures pin the parser against inbound/outbound/both/deep-hop/function-not-found envelopes; synthetic policy pins cover endpoint fallback identity, exact-duplicate dedupe ordering, absent-array leniency, hop filtering, and M-01 boundary strictness (`regression_bare_to_name_with_qualified_endpoint_is_retained`; partial/multi-segment targets match nothing); four fresh-process `serial(cbm_live)` probes run over a SYNTHETIC temp-dir fixture repo (caller -> callee is the only relationship; nothing derives from this repository): typed `graph_query` CALLS rows, two-endpoint outbound preservation, single-endpoint inbound discovery, and two-endpoint inbound-only discovery via the fallback.
- **Architectural invariant `CBM-WIRE-001`** in `docs/ARCHITECTURAL_INVARIANTS.md` - the verified CBM 0.8.1 trace_path wire contract is now normative: `inner["edges"]` is not a valid response shape and must never be assumed by any parser, wrapper, or fixture.

### Changed

- Mock helper `new_mock_with_edges()` dropped its dataflow parameter; regression/e2e/inference-layer/pipeline tests updated to the new Result semantics.
- Documentation refreshed across four areas (graph intelligence, multi-root lifecycle, CBM compatibility/limitations, testing/verification); stale claims of working DATAFLOW enrichment removed rather than caveated; stale test counts corrected everywhere (previously 2,263 in general docs, 1,512 in SECURITY.md).
- **M-01 target-predicate boundary normalization.** CBM identifies symbols by QUALIFIED names while `graph_trace` accepts bare names; the post-filter now matches exact-equal OR final-dot-segment endpoints (partial/multi-segment targets match nothing), so a bare-to name retains edges whose wire endpoints are qualified. Public API signatures, cache keys, handler output contracts, compression, and graph_query semantics are otherwise unchanged.
- Stale `trace_path` / `get_architecture` wire-shape rows corrected in `extradocs/CBM_API_AUDIT_AND_PHASE2_PLAN.md` (on-disk reference copy; `extradocs/` is untracked by design).

### Verification

- Fresh-process/fresh-index live audit against a rebuilt CBM binary: all 9 audit probes green; self-contained multilingual fixture green through step 16 including cross-language resolution and primary-project health after project switches.
- Trace-wire audit verified separately: verbatim raw-capture fixtures (fresh subprocess) pin the parser deterministically, and four fresh-process live probes over a synthetic temp-dir fixture prove typed `graph_query` CALLS rows, outbound preservation, single-endpoint inbound discovery, and the two-endpoint inbound-only fallback regression.
- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` zero warnings.
- `cargo test --workspace --all-targets --all-features`: **2,497 passed / 0 failed / 5 ignored** (core library 2,157 + CLI binary 11 + proxy crate 329: lib 155, bin harness 155, audit-regression 18, e2e integration 1).

---

## [0.3.1] — 2026-08-23 — C-22 Canonical Source-Span Contract & Multi-Class Fix

### Added

#### C-22: Canonical Decorator/Annotation/Attribute-Inclusive Class Source Contract

- **Architectural invariant established:** `MetaLayerPass` derives class source spans from `PassContext.captures` (the canonical `CapEntry` capture identity) — NOT from `CoreOp::DefClass.name`. This eliminates the semantic corruption where `DefClass.name` carried full source text instead of a class name.
- **`class_source_from_capture()`** in `src/meta_util.rs` — trilingual backward scan for TS `@Name(...)`, Java `@Name`, C# `[Name]`. Non-decorated classes use the declaration-keyword byte as fallback (backward compatible).
- **`MetaLayer::enrich()`** now receives `class_captures: &[String]` directly, eliminating the `DefClass.name` round-trip that previously corrupted framework marker detection for Java and C# meta-layers.
- **Architectural debt resolved:** P0 design document (`docs/P0_SOURCE_SPAN_CONTRACT_DESIGN_2026-08-22.md`) marked as IMPLEMENTED with all 5 recommended steps complete.

#### R-43b: Multi-Class-Per-File Support & Per-Class Isolation

- **Cross-contamination fix:** Three architectural defects allowed markers to leak between classes in multi-class files:
  1. `find_class_source_start` backward scan could walk past preceding class's closing `}`
  2. `find_class_body_open` only scanned for `class ` keyword (missing `interface`/`enum`/`record`/`struct`)
  3. `MetaLayer::enrich` extracted `class_captures` from `DefClass.name` (a semantic round-trip)
- **Class boundary guard:** `find_class_source_start` now stops at a closing `}` — `}` is never a valid annotation prefix.
- **Type-aware body scanning:** `find_class_body_open` handles all type keywords (`class`, `interface`, `enum`, `record`, `struct`) for Spring Boot and Angular.
- **Per-class metadata invariant:** A meta-layer may inspect only the exact source span belonging to the type it is enriching. It must never infer ownership from neighboring or whole-file text.

#### Multi-Class Verification Tests (9 new)

- **Java/Spring Boot (3 tests):** `MultiClassFixture.java` (6 classes) — verifies `@RestController`/`@Service` markers don't leak to `AppConfig` or `HealthController` at Low/Medium/High.
- **TypeScript/Angular (3 tests):** `MultiClassFixture.ts` (5 classes) — verifies `@Component`/`@Injectable` markers don't cross-contaminate at Low/Medium/High.
- **C#/.NET (3 tests):** `MultiClassFixture.cs` (5 classes) — verifies `[ApiController]` markers don't leak to `NotificationHub` or `InventoryDbContext` at Low/Medium/High.
- Each test asserts document-order preservation and per-class marker isolation through the production `compress_text` pipeline.

### Fixed

- **UTF-8 encoding corruption** in 4 test assertions (`src/tests/compression/pipeline.rs`): mojibake (corrupt `0xCE+0xA6`→`Φ`, `0xCE+0xB1`→`α`, `0xC2+0xA7`→`§`) caused false-negative failures in `compress_text_with_aliases_medium_fidelity`, `compress_text_with_aliases_high_fidelity`, `compress_text_emits_angular_component_markers`, and `compress_text_emits_angular_injectable_markers`. Restored via `git checkout` and re-appended with explicit UTF-8 encoding.

### Changed

- `docs/ARCHITECTURAL_INVARIANTS.md` — C-22 added as ENFORCED invariant.
- `docs/ARCHITECTURE_OVERVIEW.md` — System diagram now shows Spring Boot and .NET meta-layer boxes plus the `LayerRegistry` dispatch pattern.
- `docs/COMPILER_IR.md` — §3.1 pipeline description updated to include `DotNetMetaLayer`, C-22 class-captures derivation, and multi-class invariant. §9 meta-layer table expanded with .NET and full Angular ecosystem markers.
- `docs/P0_SOURCE_SPAN_CONTRACT_DESIGN_2026-08-22.md` — Status changed from DESIGN ONLY to IMPLEMENTED; Executive Summary table updated to reflect all three meta-layers producing markers on both IR and text paths.
- `extradocs/FAANG_AUDIT_FINDINGS.md` — P0-4 (dual meta-layer systems) marked as resolved.
- All 406 existing tests remain green.

### Version history

| Version | Date | Highlights |
|---------|------|------------|
| 0.3.1 | 2026-08-23 | **Architectural fixes.** C-22 canonical source-span contract, R-43b multi-class per-class isolation, class boundary guards, type-aware body scanning, 9 multi-class verification tests |

---

## [0.3.0] — 2026-08-12 — Angular Ecosystem Deepening (R-23/R-24/R-25)

### Added

#### RxJS Meta-Layer (R-24) — `src/angular_meta/rx.rs`
- New `RxJsKind` marker namespace (`Φobs:`, `Φsubject:`, `ΦpipeRx:`, `Φmap:`, `Φtap:`, `Φfilter:`, `Φcatch:`, `Φfinalize:`, `Φdelay:`, `Φcombine:`, `Φshare:`, `Φto:`, `Φwith:`, `Φscan:`, `Φdistinct:`, `Φretry:`) with `PhiMarker` impl
- `RxShape` struct + `extract_rx_shape()` — import-gated on `from 'rxjs'` / `rxjs/operators`
- Observable field detection (type annotations, `$` suffix, creation functions, service calls), subject instantiations (`Subject`/`BehaviorSubject`/`ReplaySubject`/`AsyncSubject` with initial value), pipe chains (`.pipe(` with depth/string-aware body capture), static combinators (`combineLatest`/`forkJoin`/`merge`/`zip`/`race`)
- `render(fidelity)` / `render_with_config(fidelity, min_pipe_operators)` — pipe chains suppressed below the configurable operator threshold (default 2)

#### NgRx Meta-Layer (R-23) — `src/angular_meta/ngrx.rs`
- New `NgRxKind` marker namespace (`Φngrx:`, `Φaction:`, `Φreducer:`, `Φeffect:`, `Φselector:`, `Φentity:`, `Φstore:`, `Φdispatch:`, `Φselect:`) with `PhiMarker` impl
- `NgRxShape` struct + `extract_ngrx_shape()` — import-gated on `@ngrx/store|effects|entity|data` with barrel-import fallback
- Action creators, reducers (standalone + inline `createReducer` in `createFeature`), effects (source action → service call → success/failure action, `{ dispatch: false }`), selectors, entity adapters, Store DI, dispatch/select call sites (multi-line + string-aware)
- NgRx Data `EntityCollectionServiceBase<T>` → `Φentity:T (data-layer)` (auto-generated CRUD)
- Cross-layer graph edges (`NgRxEdgeKind`) wired into `AngularGraph` via `to_graph_edges()`: Action→Reducer, Action→Effect, Effect→Service, Effect→Action, Component→Store, Component→Selector

#### Signals Meta-Layer — `src/angular_meta/signals.rs`
- New `SignalKind` marker namespace (`Φsignal:`, `Φcomputed:`, `Φsig-effect:`, `ΦtoSignal:`, `ΦtoObservable:`, `ΦlinkedSignal:`) with `PhiMarker` impl
- `SignalShape` + `extract_signal_shape()` — import-gated on `@angular/core` + signal function usage
- `signal()` / `computed()` / `effect()` / `toSignal()` / `toObservable()` / `linkedSignal()` declarations; `effect()` disambiguated from NgRx `createEffect` (identifier-preceded guard)

#### Routing Meta-Layer — `src/angular_meta/routing.rs`
- New `RouteKind` marker namespace (`Φroute:`, `Φguard:`, `Φresolver:`) with `PhiMarker` impl
- `RouteShape` + `extract_route_shape()` — import-gated on `@angular/router`
- `Routes` arrays, `RouterModule.forRoot/forChild`, lazy `loadComponent`/`loadChildren`, class + function guards, class + function resolvers; field-order-agnostic object-key parsing with escape-aware quoted paths

#### Shared infra
- `src/meta_util.rs` — layer-agnostic string/depth-aware parsing primitives: `split_top_level`, `find_matching_brace`, `find_first_top_level`, `find_enclosing_brace`, `collect_call_body`, `consume_call_expression`, `extract_first_quoted`, `extract_entity_type`, `extract_decl_name`, `is_inside_comment_or_string` (Round-8 structural refactor + Round-11)
- `src/angular_meta/phi.rs` — generic `PhiMarker` trait + `PHI_EXPANDERS` registry (registering a new sub-layer is a 1-line change)
- `src/angular_meta/util.rs` — re-export shim for the meta-layers
- Config sub-layers (`RxJsConfig`, `NgRxConfig`, `SignalsConfig`, `RoutingConfig`) honored via `render_with_config`; `layers/meta/mod.rs` threads them through `run_meta_layer_with_config`

### Fixed

#### Round-7 → Round-11 FAANG audits (the 4 extraction layers)
- **Round-7:** string-aware pipe/brace scans (`collect_call_body`), named effects, multi-line call sites, depth-aware combinator args
- **Round-8:** structural refactor — all string/depth parsing centralized into `src/meta_util.rs`; no per-layer hand-rolled scanners
- **Round-9:** type-annotated assignment names (`users$: Observable<T> = this.http.get(...)` → `Φobs:users$`, not the type token), array-map false-positive guard in effect success-action detection, partial-identifier guard in signals (`= signalName(` ≠ `signal()`)
- **Round-10:** string-aware `@Component` scanner (`find_matching_brace`), comment-skip guards for `path:`/`implements`/`Resolve<`, nearest-class lookup in `class_name_before`, flaky timing-test fix (`audit8_state_new_is_fast`)
- **Round-11:** systematic comment/string-awareness — new `is_inside_comment_or_string` threaded through every scan site (rx, ngrx, signals, routing); `is_routes_context` gate in routing so `path:` in an unrelated object literal is not treated as a route; 16 new regression tests

### Tests
- `src/tests/angular_meta/rx.rs` — 26 tests (observable/subject/combinator/pipe/fidelity/marker round-trip + Round-9/11 regressions)
- `src/tests/angular_meta/ngrx.rs` — 24 tests (actions/reducers/effects/selectors/entity/data-layer/inline-feature/marker round-trip + Round-9/10/11 regressions)
- `src/tests/angular_meta/signals.rs` — 21 tests (signal/computed/effect/toSignal/toObservable/linked/render + Round-9/11 regressions)
- `src/tests/angular_meta/routing.rs` — 23 tests (routes/guards/resolvers/fidelity + Round-10/11 regressions)
- `src/tests/angular_meta/graph_ngrx.rs` — 7 cross-layer graph tests
- `src/tests/angular_meta/util.rs` — 6 new `is_inside_comment_or_string` tests
- Flagged tests: `ngrx` optional-method syntaxes (`OptionalMethod`) and `missing_docs` gates verified

**Verification:** 2,263 tests passing (up from 2,141), 0 clippy warnings under `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

---

## [0.3.0] — 2026-08-07 — Angular HTML Template Compression

### Added

#### R-44: Angular HTML Template Compression
- New `src/angular_meta/template_compress.rs` module — fidelity-gated Angular template compression entry point:
  - `compress_template(html, fidelity)` — Low → single-line shape summary, Medium → multi-line structural Angular semantics, High → near-full template with HTML scaffolding stripped
  - `compress_template_to_string(html, fidelity)` — joined-string convenience wrapper
  - `compress_template_with_prime_ng(html, fidelity)` — appends PrimeNG `Φp-<name>:` markers
  - `is_prime_ng_component(tag)` / `extract_prime_ng_markers(shape)` — PrimeNG pattern recognition (Phase 4)
- `TemplateShape::to_marker_lines(fidelity)` in `src/angular_meta/template.rs` — fidelity-gated rendering:
  - Low → byte-identical to existing `to_marker_line()` (non-regression)
  - Medium → preserves `@if(cond)`, `@for(var of iter)`, custom elements with binding expressions, structural directives
  - High → preserves all elements, all bindings, all conditions, all event handlers, interpolation count
  - Empty template → `["Φtpl:empty"]` at all fidelities
- `TemplateShape` extended with: `elements` (structured `TemplateElement`), `if_conditions`, `for_loops`, `prop_binding_exprs`, `event_binding_exprs`, `two_way_binding_exprs`
- `TemplateElement` struct with `render()` — compact per-element line with bindings/directives
- `PhiLineKind` extended with `TemplateBinding` (`Φtbind:`), `TemplateDirective` (`Φtdir:`), `TemplateComponent` (`Φtcmp:`) — full vocabulary wiring (marker_prefix, expansion, expand order, token lookup)
- `src/angular_meta/mod.rs` — exports `template_compress` module; `run_meta_layer()` uses `shape.to_marker_lines(fidelity)`

#### R-44 Phase 2: GitDiff Integration
- `src/gitdiff/engine.rs` — `.component.html` files routed through the Angular template compressor:
  - `is_angular_template(path)` — detects `.component.html` (feature-gated)
  - `diff_two_contents()` — modified `.component.html` files produce compressed old/new template change-sets (not line-count deltas)
  - `compress_added_file()` — added `.component.html` files emit compressed template skeleton

#### R-44 Phase 3: Heuristics + provide_code_context Integration
- `src/mcp/heuristics.rs`:
  - `is_angular_template_path()` — detects `.component.html`
  - `classify_file()` — `.component.html` classified as `FileClass::Implementation` with `Fidelity::Medium` default (checked before Config classification)
  - `resolve_fidelity()` — `intent="edit"` on `.component.html` → `Fidelity::High` (template editing trigger)
- `src/mcp/tool_handlers/core.rs` — `handle_provide_code_context()` routes `.component.html` files through `compress_template_with_prime_ng()`:
  - Fidelity resolution: explicit arg > edit-intent High > Medium default
  - Records compression stats to the `angular_template` domain
  - Persists to SQLite DB (`queue_save_context` + `flush_persistence`) so `context_stats` and cross-session dashboards report Angular template savings
  - Injects baseline cache breakpoint

#### R-44 Phase 4: PrimeNG Pattern Recognition
- `Φp-<name>:` markers for PrimeNG components (`p-table`, `p-card`, `p-message`, etc.)
- `is_prime_ng_component()` operates on `custom_elements` (tags containing a hyphen) — safe from `<p>`/`<picture>` false-positives

#### Dashboard: `angular_template` domain
- `src/mcp/session_stats.rs` — `angular_template` domain added to the per-domain breakdown rendering (`Angular Templates: {raw} → {comp} ({savings}%↓)`)

### Fixed
- **Word-boundary bug** in `src/angular_meta/template.rs`: `@if`/`@for` inside string literals (e.g. `{{ "@if (x)" }}`) or identifiers (e.g. `@formatter`) were falsely captured into `if_conditions`/`for_loops`. Gated extraction behind the same `contains_at_keyword` word-boundary check used for `control_flow_blocks`. Added regression tests.
- **Persistence gap** in `src/mcp/tool_handlers/core.rs`: `.component.html` compressions were recorded in-memory but never persisted to the SQLite DB. Added `queue_save_context` + `flush_persistence`.

### Tests
- `src/tests/angular_meta/template_compress.rs` (new) — 17 tests: Low/Medium/High fidelity rendering, PrimeNG detection/markers, empty-template edge cases
- `src/tests/angular_meta/template.rs` — 2 new regression tests for the word-boundary fix
- `src/tests/angular_meta/markers.rs` — `phi_line_kind_uniqueness` updated to 17 variants
- `src/tests/mcp/heuristics.rs` — 3 new tests for `.component.html` classification + edit-intent fidelity
- `src/tests/gitdiff/engine.rs` — 2 new tests for `.component.html` diff change-sets
- `src/tests/mcp/session_stats.rs` — 1 new test for `angular_template` domain rendering

**Verification:** 2,141 tests passing, 0 clippy warnings. Live E2E verified: `provide_code_context` on `.component.html` returns fidelity-gated output with PrimeNG markers + baseline breakpoint. Measured compression: High 47.4% byte / 34.2% token reduction, Medium 32.1% byte reduction.

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

## [0.1.7] — 2026-06-20

### Added

#### Multi-Platform Proxy Support
- **Platform-agnostic proxy**: The proxy now supports any AI provider (Anthropic, OpenAI, DeepSeek, etc.) via the `PlatformAdapter` trait.
  - `proxy/src/platform/mod.rs` — `PlatformAdapter` trait with `is_tool_result()`, `extract_tool_result()`, `intercept_path()`, `platform_headers()`, `is_platform_model()`, `platform_name()` methods
  - `proxy/src/platform/anthropic.rs` — Anthropic API adapter (`type: "tool_result"` blocks, `cache_control`, `anthropic-beta` header)
  - `proxy/src/platform/openai.rs` — OpenAI API adapter (`role: "tool"` messages, `tool_call_id`)
  - `proxy/src/platform/generic.rs` — Generic fallback adapter (heuristic detection, multiple content field locations)
  - `platform::detect_platform()` — Auto-detection from model name in request body
  - `PLATFORM` env var — Manual override (`anthropic`, `openai`, `generic`)
  - Server now intercepts `/v1/messages` (Anthropic), `/v1/chat/completions` (OpenAI), and `/chat` (Generic)

#### Tool Output Filtering (R-38)
- **TOML-based filter engine**: 7 built-in filters that compress verbose tool output by 70-90%.
  - `proxy/src/filters.rs` — 7-step filter pipeline: `replace → match_output → strip/keep_lines → group_by → head/tail → max_lines → on_empty`
  - `proxy/src/filter_rules.rs` — TOML parsing, `CompiledFilter` struct, `compile_filter_file()` with regex validation
  - `proxy/src/filter_registry.rs` — Most-specific-match-wins filter selection with priority tiebreaker
  - `proxy/src/filter_loader.rs` — Built-in filter loading from `filters/` directory at startup
  - `proxy/src/filter_stats.rs` — Per-program token/line savings tracking with dashboard summary
  - `proxy/src/community_filters.rs` — Community filter loading from `.clean-ctx/filters/`
  - `filters/cargo.toml` — Compact cargo build/test/check/clippy output
  - `filters/npm.toml` — Compact npm/yarn/pnpm/bun install/build output
  - `filters/git-diff.toml` — Compact git diff/show output
  - `filters/pytest.toml` — Compact pytest output
  - `filters/tsc.toml` — Compact TypeScript compiler output
  - `filters/dotnet.toml` — Compact dotnet build/test/run output
  - `filters/ng.toml` — Compact Angular CLI build/test/lint output
  - `TOOL_FILTERS` env var — Enable/disable tool output filtering

#### Secret Scrubbing (R-37)
- **Platform-agnostic secret scrubbing**: Detects and redacts secrets in tool results before they reach the LLM.
  - `proxy/src/scrub.rs` — `scrub_secrets()` engine with `ScrubResult`, `ScrubHit`, `ScrubFailClosed` semantics
  - `proxy/src/scrub_patterns.rs` — Compiled `OnceLock<Regex>` statics for AWS keys, GitHub tokens, JWTs, PEM keys, etc.
  - `might_contain_secret()` pre-filter — Cheap literal-substring check before expensive regex passes
  - `SCRUB_SECRETS` env var — Enable/disable secret scrubbing
  - Now uses `PlatformAdapter::is_tool_result()` for cross-platform detection

#### Pluggable Transform Pipeline
- **`Pipeline` abstraction**: Makes the transform chain pluggable and testable (OCP compliance).
  - `proxy/src/pipeline.rs` — `Pipeline::build()` returns composed transforms, `Pipeline::run()` executes all
  - New transforms added via closure without modifying existing code
  - Each transform receives `&dyn PlatformAdapter` for format-aware operations

#### FAANG Audit & Regression Tests
- **Comprehensive code audit**: 243 total tests (112 lib + 112 bin + 18 audit regression + 1 integration)
  - `proxy/tests/audit_regression.rs` — 18 regression tests covering all audit findings
  - Critical bugs fixed: hardcoded Anthropic `tool_result` detection in `strip_ansi` and `scrub_secrets`
  - Security fix: exact path matching instead of `ends_with` for intercept routing
  - All transforms now platform-agnostic via `PlatformAdapter`

### Changed
- `ANTHROPIC_BASE_URL` env var renamed to `UPSTREAM_URL` conceptually (backward-compatible)
- `server.rs` now uses `Pipeline::build()` and `Pipeline::run()` instead of inline transform calls
- `transform::strip_ansi()` now takes `&dyn PlatformAdapter` parameter
- `transform::scrub_secrets()` now takes `&dyn PlatformAdapter` parameter
- `transform::apply_tool_filters()` now uses `PlatformAdapter::is_tool_result()` for cross-platform detection
- All filter modules (filters, filter_rules, filter_registry, filter_loader, filter_stats, community_filters) exported from `lib.rs`
- Documentation updated at `docs/PROXY.md` with multi-platform, filtering, scrubbing, and IDE integration details

### Test count
- 243/243 tests pass (112 proxy lib unit + 112 proxy bin unit + 18 audit regression + 1 integration)
- 0 new clippy warnings (pre-existing warnings only: dead-code on API surface types)

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
| 0.4.0 | 2026-08-24 | **CBM Graph-Intelligence & Trace-Wire Audits.** Blast-radius CALLS-edge fail-open fixed; Result-propagating intel APIs (`Ok(empty)` vs `Err(ToolError)`); dual-label dead code; dead DATAFLOW path removed; typed `graph_trace` parses the real callers/callees wire contract (phantom `edges` key eliminated); inbound-fallback direction determination; M-01 qualified/bare boundary normalization — invariant `CBM-WIRE-001`, 16-test `trace_wire.rs` suite, 2,497 tests, 0 clippy warnings |
| 0.3.0 | 2026-08-12 | **Angular Ecosystem Deepening (R-23/R-24/R-25).** RxJS/NgRx/Signals/Routing meta-layers, cross-layer NgRx graph edges, Round-7→Round-11 FAANG audit hardening — 2,263 tests, 0 clippy warnings |
| 0.3.0 | 2026-08-07 | **Angular HTML Template Compression (R-44).** Fidelity-gated `.component.html` compression, PrimeNG markers, GitDiff integration, `angular_template` dashboard domain — 2,141 tests, 0 clippy warnings |
| 0.3.0 | 2026-08-04 | **IR Evolution (R-43a + R-43b).** Execution semantics, program graph, inference layer, pass pipeline, validation, query, semantic delta intent |
| 0.2.1-rc2 | 2026-07-03 | **Meta-Layer expansion.** .NET/C# meta-layer, Dual Meta-Layer analysis, A-08 sliding window proxy, P1-9 feature-gate hardening — 1,489 tests, 0 clippy warnings |
| 0.2.1-rc1 | 2026-06-30 | **Foundation complete.** A-09 through A-15, F-19 through F-22 — 1,512 tests, 0 clippy warnings |
| 0.1.7 | 2026-06-20 | Multi-platform proxy, tool output filtering (R-38), secret scrubbing (R-37), pluggable tokenizers (R-19), Java/Rust language layers (R-01d/R-01c) |
| 0.1.6 | 2026-06-10 | Zero-touch workflow + SQLite persistence + XHTML fix + inline template — 798 tests, 0 clippy warnings |
| 0.1.5 | 2026-06-08 | FAANG Audit Compiler IR Phase E (F-30 through F-47) — 318 tests, 0 clippy warnings |
| 0.1.4 | 2026-06-08 | Tracks C+D: Phi marker centralisation + god-function split — 301 tests, 0 clippy warnings |
| 0.1.3 | 2026-06-08 | Track B: `AngularGraphBuilder` typestate split — 293 tests, 0 clippy warnings |
| 0.1.2 | 2026-06-07 | Angular Meta-Layer Phase 3 (cross-file DI + selector graph) — 279 tests, 0 clippy warnings |
| 0.1.1 | 2026-06-07 | Angular Meta-Layer Phase 2.5 (modern Angular 17–21 syntax) — 244 tests, 0 clippy warnings |
| 0.1.0 | 2026-06-07 | Initial release — all 5 FAANG audit phases complete, 121 tests, 0 clippy warnings |
| 0.0.0 | 2026-06-06 | Audit baseline — 58 tests, 41 findings |
