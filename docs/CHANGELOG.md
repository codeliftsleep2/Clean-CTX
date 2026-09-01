# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.6.0] - 2026-08-31

### Added

#### Semantic architecture (Phases 1–6)

* **Typed `SemanticEdge`, `EntityRef`, and `SemanticRelation` model** (`src/layers/meta/semantic.rs`) — structured semantic relationship representation with provenance tracking per meta-layer.
* **`extract_semantic_edges()` on `MetaLayer` trait** — meta-layers now produce structured semantic edges alongside existing Phi text output. Default implementation returns an empty vector.
* **`InferenceLayer.semantic_edges` field** — `InferenceLayer` now carries both `inferred_edges` (CBM/structural/heuristic) and `semantic_edges` (meta-layer facts). `all_edges_for()` returns both collections.
* **`PassContext.semantic_edges` field** (`pipeline.rs`) — semantic edges flow through the compression pipeline into the inference layer.
* **Angular semantic edge extraction** — Angular, NgRx, Signals, and Routing meta-layers emit `SemanticEdge` objects: Injects, UsesService, HasSelector, HandlesAction, Dispatches, TriggersReducer, ProducesAction, HasStore, RouteMapsTo, GuardedBy, ResolvedBy, Defines, and more.
* **.NET semantic edge extraction** (`src/dotnet_meta/semantic.rs`) — Controller, Minimal API, and AutoMapper semantic relationships represented as typed edges.
* **Spring Boot semantic edge extraction** (`src/spring_meta/semantic.rs`) — REST endpoint mappings, service injections, and configuration relationships represented as typed edges. Class-level `@RequestMapping` prefix correctly combined with method-level mapping annotations.

#### WorkspaceIndex (Phases 4a–4c)

* **`WorkspaceIndex`** (`src/workspace/`) — framework-agnostic cross-file semantic index. Provides entity/edge lookup, forward/reverse traversal, transitive dependency resolution, and deduplication by semantic identity.
* **Cross-file resolution** — `WorkspaceIndex` replaces the role previously served by the deleted legacy graph modules for semantic cross-file queries.
* **Graph traversal** — `transitive_dependencies()` and cycle detection for DI and routing relationships.

#### Workspace query interface (Task C)

* **`workspace_query` MCP tool** (`src/mcp/tool_handlers/query.rs`) — read-only cross-file semantic query API over `WorkspaceIndex`. Supports `find_entities`, `forward_edges`, `reverse_edges`, `entities_in_file`, `transitive_dependencies`, and `has_cycle`. Uses the same `(domain, entity_type, name)` identity model. No changes to the write lifecycle, CBM module, or WorkspaceIndex internals.
* **Handler registration** — registered through the standard handler registry in `src/mcp/tool_handlers/registry.rs`, exposed via `tool_list()` in `src/mcp/tools.rs`.
* **Docs** — `docs/agent/tooling.md` updated with `workspace_query` entry in the Workspace Tools section.

#### MCP token-economics correction

* **Post-compression token-economics gate** — after the compressed/hybrid representation is rendered, its actual token cost is compared against the raw source. If the candidate costs more tokens than raw, `raw_passthrough` is selected.
* **Uniform fidelity coverage** — the post-compression check applies to Low, Medium, High, and Edit fidelity compressed/hybrid output.
* **Pre-compression heuristic retained** — the existing BPE-based preflight heuristic for Edit fidelity remains as an optimization hint only.
* **Genuinely cheaper compression preserved** — compression is still selected when the candidate representation saves tokens.

### Fixed

- **Token-economics gate scope** — the pre-compression heuristic previously only applied to Edit fidelity. The new post-compression verification applies uniformly to all fidelity levels. See `src/mcp/tool_handlers/core.rs` (`maybe_economics_fallback`), `src/mcp/token_economics.rs`.

### Removed

- **Legacy meta-layer graph infrastructure** — removed after semantic coverage was verified through `extract_semantic_edges()` and `WorkspaceIndex`. The following deleted modules had no remaining imports after migration:
  - `src/angular_meta/graph.rs` — `AngularGraph`, `AngularGraphBuilder`, `ClassEntry`, `ClassKind`
  - `src/angular_meta/graph_state.rs` — `AngularGraphHandle`
  - `src/dotnet_meta/graph.rs` — `.NET DI graph`
  - `src/dotnet_meta/graph_state.rs` — `DotnetGraphHandle`
  - `src/spring_meta/graph.rs` — `SpringGraph`
  - `src/spring_meta/graph_state.rs` — `SpringGraphHandle`
- **`graph_state` re-exports and McpState integration fields** for all three legacy graph types.
- **`workspace.rs` graph-building pass** — the post-bundling `GraphCollector`/graph-build step no longer exists.
- **`compress_workspace` graph footer emission** — `§ΦGRAPH` and `Φgraph:` marker lines are no longer emitted. Phi text markers (`Φcmp:`, `Φsvc:`, etc.) remain.

### Changed

- `MetaLayer` trait now has `extract_semantic_edges()` with default empty implementation. All existing meta-layers (Angular, NgRx, Signals, Routing, .NET, Spring Boot) implement it.
- `Docs/developer_documentation.md`, `ARCHITECTURE_OVERVIEW.md`, `ANGULAR_META_LAYER.md`, `DOTNET_META_LAYER.md` updated to reflect the current architecture.
- `Cargo.toml` version bumped to `0.6.0`.

### Tests

| Area | Count |
|------|------:|
| `src/tests/angular_meta/semantic.rs` — Angular semantic edge extraction | 12 tests |
| `src/tests/dotnet_meta/semantic.rs` — .NET semantic edge extraction | ✓ |
| `src/tests/spring_meta/semantic.rs` — Spring semantic edge extraction | ✓ |
| `src/tests/workspace/index.rs` — WorkspaceIndex resolution, dedup, traversal | 20+ tests |
| `src/tests/mcp/tool_handlers.rs` — token-economics regression | 9 tests |
| **Total** | **2,200+ tests passing** |

---

## [0.5.2] - 2026-08-29

### Fixed

- **CBM transport pipeline deduplication — unified write/read/flush/timeout/disconnect state machine.** Two separate implementations of the same low-level pipe I/O loop existed in `call_tool_inner()` and `call_tool_raw_inner()`, causing stale-pipe divergence, retry inconsistency, and dead-subprocess recovery asymmetry. Extracted the shared transport into `send_and_receive_raw()` — the single method both entry points delegate to. `call_tool_inner()` remains the parsed-`Value` semantic entry point; `call_tool_raw_inner()` remains the raw-response-text entry point. Retry loops in `call_tool()` / `call_tool_raw()` are unchanged. (`src/cbm/client.rs`)

---

## [0.5.1] - 2026-08-29

### Fixed

- **`graph_query` node-result data fidelity — projected `file_path` no longer erased.** `GraphBridge::convert_query_rows()` read only column 0 on node-shaped Cypher projections (`RETURN f.name, f.file_path`) and built `GraphNode { id, name, label: "", file: "", properties: {} }`, silently discarding the `f.file_path` cell CBM was returning. The 0.5.0 structured-output migration made the defect visible (empty `structuredContent.nodes[*].file`) against an `outputSchema` that already requires `file`. Node-shaped projections now resolve from the echoed column metadata exactly like the relationship-shaped path: column 0 keeps its legacy `id`/`name` role, a `file_path` projection populates `GraphNode.file`, a clearly recognizable `label` projection populates `GraphNode.label` (legacy empty default otherwise), and every other projected column is preserved in `GraphNode.properties` keyed by the echoed column text with its projected JSON value verbatim. The relationship/edge-shaped conversion path is unchanged. **Bug/conformance fix only — no MCP response schema change.** (`src/cbm/bridge.rs`, `src/tests/cbm/query_wire.rs`, `src/tests/cbm/handlers.rs`)

### Cache

- **Query-cache namespace bumped `cypher:` -> `cypher2:`** so results cached before this fix (always-empty `file` cells) can never be served after upgrade. SQLite schema unchanged; pre-0.5.1 `cypher:` rows remain in the database untouched (never read, never migrated). (`src/cbm/bridge.rs`)

## [0.5.0] - 2026-08-28

### Changed

- **Canonical MCP structured-output architecture — `graph_search` reference implementation.**
  Tool responses now use the MCP 2025-11-25 `structuredContent` channel instead of
  ad-hoc `result` fields. `graph_search` is the golden path: `content` carries a
  human-readable node summary while `structuredContent` carries the typed `{nodes,
  count, cbm_status}` payload. An `outputSchema` is declared in `tools/list` for the
  first time. Error responses use `isError: true` + `structuredContent` instead of
  ad-hoc `error` fields. New wire-level contract tests validate the `CallToolResult`
  shape and would have caught the previous invisible-data problem.
  (`src/cbm/handlers.rs`, `src/cbm/tools.rs`, `src/tests/cbm/handlers.rs`)

### Changed

- **CBM graph tools to `structuredContent` — `graph_query`, `graph_trace`, `get_architecture`.**
  Following the `graph_search` golden path, the remaining structured CBM handlers now use
  the canonical response envelope. Primary payload and `cbm_status` moved into
  `structuredContent` (`{nodes, edges}` for `graph_query`; `{edges, count}` for
  `graph_trace`; `{modules, dependencies}` for `get_architecture`); all ad-hoc
  result-level fields removed; error paths use `isError: true` + `structuredContent.error`.
  Each tool now declares an `outputSchema` in `tools/list`. (`src/cbm/handlers.rs`,
  `src/cbm/tools.rs`)

- **`apply_edit` to `structuredContent` + `_meta`.** Success response is now
  `content` + `structuredContent.operations` + `_meta{filePath, fileHash, version,
  applied, syntaxGated}`. An `outputSchema` declares the operation contract (`kind`
  enum over `replace_body`/`insert_after`/`insert_before`/`delete`; required
  `kind`/`target`/`startByte`/`endByte`/`byteDelta`; optional `newText`). Errors
  remain JSON-RPC with structured `error.data`. (`src/mcp/tool_handlers/edit.rs`,
  `src/mcp/tools.rs`)

- **Response metadata to `_meta` across the remaining tools.** `get_cbm_status`,
  `still_indexing` (shared indexing gate, cross-cutting to every gated CBM query),
  `save_context`, `purge_old_deltas`, `replay_history`, `restore_context`,
  `apply_delta`, and all seven `provide_code_context` response sites moved ad-hoc
  result-level fields into `_meta`. `save_context` and `purge_old_deltas` gained the
  previously-missing `content` field. Existing JSON-RPC error paths and `error.data`
  payloads are preserved unchanged. (`src/cbm/handlers.rs`,
  `src/mcp/tool_handlers/core.rs`, `src/mcp/tool_handlers/persistence/mod.rs`)

- **MCP-layer dispatch/error-path unification (D1/D2/D5).** Architectural cleanup of
  the MCP handler layer consolidating three previously-duplicated mechanisms:
  - **D1 — JSON-RPC error-envelope unification.** Added a shared `jsonrpc_error(...)`
    builder primitive; eliminated inline JSON-RPC error-envelope construction
    throughout the MCP handler layer (`edit`, `persistence`, `core`, `error.rs`
    `to_jsonrpc_error`, and the newly migrated `gitdiff` handler). Existing error
    codes, messages, IDs, and `data` payloads are preserved exactly.
  - **D5 — parameter-extraction unification.** Added shared `arg_str` /
    `arg_str_or_empty` helpers and migrated the duplicated
    `params.arguments["filePath"]` / `["workspaceRoot"]` extraction sites across
    `core`, `context`, `persistence`, and `stats`.
  - **D2 — registry dispatch unification.** Moved `diff_commits` and
    `index_repository` out of the inline dispatch path in `tools.rs` into the
    canonical tool registry: `diff_commits` gained a dedicated `gitdiff.rs` handler
    module and `index_repository` moved into `cbm/handlers.rs`. Dispatch behavior
    and existing contracts are preserved exactly.
  (`src/mcp/tool_helpers.rs`, `src/error.rs`, `src/mcp/tools.rs`,
  `src/mcp/tool_handlers/*`, `src/cbm/handlers.rs`)

### Tests

- **Contract conformance tests for `graph_search`:** Three new handler-level tests
  validate the MCP `CallToolResult` contract — success shape (`structuredContent`
  with `nodes`/`count`/`cbm_status`, no ad-hoc fields), error shape (`isError: true`
  with error details in `structuredContent`), and CBM-unavailable handler path
  through `CAPTURED_RESPONSES`.

- **Phase 1 CBM contract tests (`src/tests/cbm/handlers.rs`):** success/error shape
  and live-dispatch tests for `graph_query`, `graph_trace`, `get_architecture`, plus
  a `tools/list` `outputSchema`↔`structuredContent` consistency test, using shared
  envelope/structured-content helpers.

- **Phase 2 `apply_edit` tests (`src/tests/mcp/apply_edit.rs`):** success-envelope
  shape, `outputSchema` consistency, live in-process dispatch via
  `CAPTURED_RESPONSES`, `newText` present/absent by `verify`, and an error-path pin
  proving the JSON-RPC `unit_mismatch` contract is unchanged.

- **Phase 3 contract suite (`src/tests/mcp/phase3_contract.rs`):** envelope
  conformance, `_meta` placement, meaningful content, and error-path stability for
  all eight migrated tools — no `structuredContent`/`outputSchema` introduced.

- **Phase 4 helper consolidation:** duplicated private CBM envelope helpers removed
  in favor of the shared `crate::tests` versions (`assert_valid_mcp_envelope`,
  `assert_structured_content_has`); the CBM-specific `assert_error_structured_content`
  stays local.

### Verification

- Focused suites: `cbm::tests::handlers` 13/13; `mcp::apply_edit` 7 passed / 3
  ignored; `phase3_contract` 8/8; phase_a + phase_b + tool_handlers 81/0; default
  `cargo test --lib` 1932 passed / 0 failed.
- `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings.
- `cargo fmt --all -- --check` — clean.
- `scripts/check-utf8.ps1` — 503 text files valid UTF-8, 0 BOMs, 0 mojibake.
- Full workspace `cargo test --workspace --all-targets --all-features` — 2238
  passed / 0 failed / 8 ignored (the known environmental CBM pipe flake did not
  occur this run).
- Post-cleanup: `cargo check --lib`, `cargo fmt --all -- --check`, and the
  targeted `phase3_contract` suite (8/8) passed after the final MCP-layer
  cleanup.
- Release-binary black-box MCP sweep: every registered tool exercised against the
  freshly rebuilt binary via the MCP JSON-RPC interface, including deliberate
  error paths (missing/invalid arguments, invalid git refs, un-gated edits), with
  raw-wire verification of the 0.5.0 response contracts (`content` /
  `structuredContent` / `_meta` / JSON-RPC error envelope) and `id` preservation.
  Working tree clean afterward.

---

## [0.4.9] - 2026-08-28
### Fixed

- Edit-fidelity `raw_passthrough` no longer prevents subsequent `apply_edit`
  operations by failing to establish tracked IR state.
- `§PATHMAP` footers for single-file context responses are now scoped to
  aliases actually required by that response instead of serializing the
  entire session-wide path dictionary.
- Working-tree `diff_commits` now includes non-ignored untracked files
  without requiring them to be staged or added with `git add -N`.
- Fixed pre-existing `clippy::manual_range_contains` warning in
  `src/tests/mcp/token_economics.rs`.

### Added

- **`diff_commits` working-tree diff now includes non-ignored untracked files.**
  When `toRef` is omitted, `collect_changed_files()` additionally runs
  `git ls-files --others --exclude-standard` and treats discovered paths as
  `FileChange::Added`, entering the existing diff pipeline unchanged. Ignored
  files remain excluded. No index mutation, no staging, no `git add -N`
  required. (`src/gitdiff/workspace.rs`)

### Tests

- **Workspace-level untracked-file tests:** Discovery in working-tree diff,
  exclusion from commit-to-commit diff, coexistence with tracked modifications,
  .gitignore exclusion, no-tracked-changes edge case.
- **Engine-level untracked-file tests:** Untracked `.ts` file produces AST
  skeleton in manifest; untracked non-compressible file falls back to line-count
  delta; ignored files excluded from manifest; mixed tracked + untracked changes
  counted correctly.
- Replaced the old `git add -N` dependency test
  (`gitdiff_workspace_working_tree_added_file_read_from_disk`) with a no-staging
  untracked variant.
- **Regression test for Edit-fidelity raw-passthrough:**
  `apply_edit_works_after_raw_passthrough_on_small_file` proves small Edit files
  remain editable after the token-economics gate returns raw passthrough.
  Applies to both compressed and raw-passthrough paths.
- **CBM E2E test correction:**
  `e2e_apply_edit_triggers_reindex_and_graph_is_fresh` now calls
  `ensure_indexed()` before `bridge.search()` to respect the architectural
  layering — the MCP handler owns the freshness precondition, not the low-level
  `search()` primitive. No production CBM code was modified.
  (`src/tests/cbm/e2e.rs`)
- **PATHMAP scoping tests:** 6 unit tests in `src/tests/dictionary/path.rs` and 3
  integration tests in `src/tests/mcp/tool_handlers.rs` verify scoped footers,
  no cross-request leakage, and full-footer retention.

### Documentation

- Removed the `git add -N` limitation from `docs/DIFF_COMMITS_GUIDE.md`.
- Documented the new untracked-file behavior in `docs/DIFF_COMMITS_GUIDE.md`.

---


## [0.4.8] - 2026-08-28 - Per-Project Lazy CBM Graph Freshness & Token-Economics Gate

### Added

- **Cheap token-economics gate before IR compression.** A new preflight check (`src/mcp/token_economics.rs`) predicts whether compression at verbatim-body-preserving fidelities (Edit, Verbatim) will produce a net token savings. If not, compression is skipped entirely and the raw file is returned through the existing response contract with `content_kind: "raw_passthrough"`. The estimator uses a single BPE token-count pass — no AST parse, no IR compile, no render. Calibration is per-language via a simple parameter table (fixed_overhead + expected_savings_ratio) with a 15% conservative bias toward compression. Structural fidelities (Low, Medium, High) are not gated — they always attempt compression. (`src/mcp/token_economics.rs`, `src/mcp/tool_handlers/core.rs`)

### Changed

- **CBM graph freshness is now lazy and per-project.** `apply_edit` no longer performs a synchronous CBM `index_repository` on the write path. Instead, it marks the affected project dirty. The next graph query (`graph_search`, `graph_query`, `graph_trace`, `get_architecture`, `cbm_proxy`) automatically triggers a synchronous fast reindex before executing. Multiple edits to the same project coalesce into a single reindex. Freshness is tracked independently per CBM project slug — editing project A does not affect project B's graph. (`src/cbm/bridge.rs`, `src/mcp/tool_handlers/edit.rs`)

- **`get_cbm_status` now reports project freshness information.** The response includes a `freshness` field with per-project `dirty_generation`, `indexed_generation`, and `is_stale` status. This endpoint remains read-only and does not trigger indexing. (`src/cbm/handlers.rs`)

- **`ProjectFreshness` struct added.** Per-project state tracking `dirty_generation` and `indexed_generation`, independent of `IndexingState`. (`src/cbm/bridge.rs`)

### Tests

- 11 regression tests for freshness behavior: dirty tracking, generation invariants, multi-project isolation, failure survival, and mock-level `apply_edit` marking. (`src/tests/cbm/regression.rs`)

- Updated E2E test `e2e_apply_edit_triggers_reindex_and_graph_is_fresh`: now verifies the full lazy contract — dirty after `apply_edit`, fresh after first graph query, clean after second. (`src/tests/cbm/e2e.rs`)

- **13 token-economics regression tests:** below-threshold skip, above-threshold compress, conservative bias near boundary, structural-fidelity bypass, per-language calibration, unknown-extension defaults, leading-dot normalization, zero-token edge case, verbatim pass-through, all-languages structural bypass, and structural-threshold-is-zero invariant. (`src/tests/mcp/token_economics.rs`)

### Documentation

- `docs/agent/tooling.md` updated: "Lazy CBM Graph Freshness after `apply_edit`" section replaces "Automatic Reindex". `apply_edit` tool table description updated. (`docs/agent/tooling.md`)

- `src/cbm/tools.rs` `index_repository` description updated to reflect lazy freshness. (`src/cbm/tools.rs`)

### Observability

- **Token-economics gate now emits tracing fields.** The existing `provide_code_context` tracing span (`tracing::info_span!`) now captures `prediction` (`"favorable"`, `"unfavorable"`, or `"bypass"`) and `threshold` (the biased token count used for the decision). Structural fidelities (Low, Medium, High) and Verbatim log `"bypass"` / `0`. This allows correlation of predictions with actual compression outcomes using structured log output, supporting future calibration of per-language `fixed_overhead` and `expected_savings_ratio` values from real-world usage data. No gate behavior, calibration, or response contract was changed. (`src/mcp/tool_handlers/core.rs`, `src/mcp/token_economics.rs`)

### Verification

- CBM regression: 77 passed, 0 failed (11 new freshness tests + 66 existing).
- Token-economics: 13/13 passed, 0 failed.
- `cargo clippy --all-targets -- -D warnings` zero warnings.
- `cargo fmt --all` clean.
- Encoding guard: 498 text files valid UTF-8 without BOM.

---

## [0.4.7] - 2026-08-28 - Automatic CBM Reindex after apply_edit & Schema Documentation

### Added

- **Automatic CBM reindex after `apply_edit`.** After a successful filesystem mutation, Clean-CTX now synchronously performs a CBM `fast` `index_repository` when CBM is available. The reindex runs after the commit lock is released, so it never blocks the filesystem mutation. The edit remains successful even if reindexing fails (failure is logged as a warning only). See `src/cbm/bridge.rs` (`reindex_for_file`), `src/mcp/tool_handlers/edit.rs` (post-commit reindex wiring). (`src/cbm/bridge.rs`, `src/mcp/tool_handlers/edit.rs`)

- **`list_projects` MCP tool.** Lists all CBM-indexed projects with their canonical identity, path, and status. Project-independent — no project parameter required. This is the authoritative mechanism for discovering the CBM project slug. (`src/cbm/tools.rs`, `src/mcp/tools.rs`)

- **`index_repository` MCP tool.** Triggers CBM to index (or reindex) a repository. Accepts `repo_path` (required) and `mode` (`"fast"` for normal refresh, `"full"` for rebuild/recovery). Agents should not normally need this after `apply_edit` (automatic reindex covers it), but it is available for external edits performed outside Clean-CTX. (`src/cbm/tools.rs`, `src/mcp/tools.rs`)

### Changed

- **CBM schema updated.** The `cbm_proxy` tool's `cbm_tool` parameter now documents six agent-facing CBM operations: `search_graph`, `query_graph`, `trace_path`, `get_architecture`, `list_projects`, `index_repository`. The `get_symbol_importance` and `get_dead_code` exclusions are preserved. (`src/cbm/tools.rs`)

- **Agent documentation refreshed.** `docs/agent/tooling.md` now includes: `list_projects`/`index_repository` in the CBM tool table, a project discovery workflow, index repository modes, automatic reindex after `apply_edit` with workflow diagrams, and the external-edit limitation. Proxy aliases (`graph_search` → `search_graph`, etc.) are documented as compatibility aliases, not preferred names. (`docs/agent/tooling.md`)

- **Integration rules updated.** `docs/CLAUDE_INTEGRATION_RULES.md` (RULE 2) lists all six forbidden direct calls, their corresponding `cbm_proxy` equivalents, and the Quick Reference includes automatic reindex and external-edit guidance. (`docs/CLAUDE_INTEGRATION_RULES.md`)

- **Architecture overview clarified.** `docs/ARCHITECTURE_OVERVIEW.md` now explicitly distinguishes the six agent-facing CBM operations from internal bridge helpers (`get_symbol_importance_mut`, `get_blast_radius`, `get_dead_code`, `get_call_edges`). (`docs/ARCHITECTURE_OVERVIEW.md`)

### Tests

- 4 regression tests for `reindex_for_file` behavior (CBM unavailable, extra-root resolution, fallback to active root, cache invalidation). (`src/tests/cbm/regression.rs`)
- 1 end-to-end test `e2e_apply_edit_triggers_reindex_and_graph_is_fresh` proving the real production `handle_apply_edit` path triggers CBM reindexing. (`src/tests/cbm/e2e.rs`)
- Schema parity test `p3_21_tool_names_match_tool_list_and_registry` updated to include `list_projects` and `index_repository`. (`src/tests/mcp/tools.rs`)

### Verification

- CBM regression: 54 passed, 0 failed.
- CBM E2E: 19 passed, 0 failed.
- apply_edit: 3 passed, 0 failed, 3 pre-existing ignored.
- `cargo check --lib` clean.
- `cargo clippy --lib -- -D warnings` zero warnings.

---

### Fixed

- **A C# constructor initializer clause was mis-selected as the parameter group.** `find_method_params` returned the LAST balanced depth-0 paren group, so for `Greeter(string prefix) : base(prefix)` — and every `: base(...)` / `: this(...)` constructor — the initializer's OWN parentheses won the selection: Low rendered the constructor as `base(prefix)`, Medium welded the clause onto the label as a fake return-type annotation after space-collapse (`Greeter(string prefix):base(prefix)`), `method_key` grouped every base-initializing constructor under `base` (distinct overloads merged into one diff entry), the IR compiler named the constructor `base` and fed the initializer argument through as a parameter, the diff body fingerprint keyed off the initializer's own parens (initializer-only edits were invisible to the differ), and the class primary-constructor peel guard peeled a base call's `(Value)` because its own tail was "empty". The parameter list is now the FIRST balanced depth-0 paren group ANCHORED TO THE DECLARED NAME — its opening `(` immediately preceded (after whitespace) by `>` (a generic-type close) or by an identifier that is neither `base` nor `this` — scanned literal-aware via the shared `skip_quoted_literal` so a default value like `void M(string s = "a (", int n)` cannot break it (`src/compaction/method.rs`). New shared `strip_base_initializer_clause` drops the `: base(...)` / `: this(...)` call-site clause from the header before Medium compaction and before `parse_method_sig` name/params/return-type derivation (`src/compaction/method.rs`, `src/ir/pipeline.rs`).

- **Behavior preservation.** Tuple-return signatures still select the method's own group (the tuple's top-level `(` is preceded by `<` and skipped); every non-constructor signature is byte-identical; High-fidelity output remains byte-exact including the full initializer clause.

- **Deliberate Medium-tier revision.** The ff2a29a Medium expectation was revised by design: the compacted label now DROPS the initializer clause (call-site metadata, not a return annotation) instead of carrying the interpolation holes, while High keeps the byte-exact header. The ff2a29a truncation fix itself is untouched.

### Tests

- Name-anchored selection RED→GREEN: base-initializer (`Greeter(string prefix) : base(prefix)` → `(string prefix)`), this-initializer, literal-paren default value (`void M(string s = "a (", int n)` → full group), generic-`>` anchor (control), generic-`new()` constraint exclusion, and the existing tuple-return test unchanged.
- Tier and consumer regressions RED→GREEN: Low label `Greeter(prefix)`, Medium label `Greeter(string prefix)`, `method_key("Greeter(string prefix) : base(prefix)")` → `Greeter`, IR identity (constructor compiles named `Greeter` with `string prefix` parameter and void return), class peel guard leaves `Example(string Value) : Base(Value)` untouched, and the body-fingerprint test fails on initializer-only edits.
- Revised ff2a29a pair: unit `high_fidelity_base_initializer_interpolation_keeps_full_header` (High byte-exact including the clause; Medium = bare declaration) and e2e `gitdiff_ctor_base_initializer_interpolation_not_truncated` (bare label `ExampleException(string value,object context)`; `{value}`/`{context}` absent; body statements absent).

### Verification

- compaction 95 passed; diff 80 passed (2 feature-ignored); gitdiff 42 passed; ir::compiler 20 passed; ir::pipeline 12 passed; `cargo test encoding` 6 passed; `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` (default and `--features rust`) 0 warnings; UTF-8 guard PASS (473 files, strict UTF-8, 0 BOMs, 0 mojibake).

---

## [0.4.5] - 2026-08-27 - Constructor Base-Initializer Signature Truncation

### Fixed

- **Interpolation hole inside a constructor base-initializer terminated the signature.** A brace-bodied C# constructor whose base-initializer argument is an interpolated string (`: base($"Unexpected value: {value}, context: {context}")`) truncated at the FIRST `{` anywhere in the capture — the interpolation hole INSIDE the literal — rendering the diff member as `method value: …:base($"Unexpected value:` with both holes and the initializer tail destroyed while raw `git diff` was correct. The body-brace boundary now shares `find_depth_zero_arrow`'s literal-aware contract: new sibling `find_depth_zero_brace` skips string/char literals via the existing `skip_quoted_literal` and tracks paren/bracket depth, so only a structural-depth-0 `{` OUTSIDE quoted/interpolated literals ends the signature (`src/compaction/method.rs`). Renderer/differ untouched; no fixture special-casing; the expression-bodied min-boundary fix and normal brace-bodied legacy extraction are byte-identical.

### Tests

- Focused unit regression `high_fidelity_base_initializer_interpolation_keeps_full_header` (`src/tests/compaction/method.rs`): the High-fidelity signature span ends at the true body brace with the full initializer intact; the Medium-tier compacted label retains both interpolation holes.
- End-to-end regression `gitdiff_ctor_base_initializer_interpolation_not_truncated` (`src/tests/gitdiff/engine.rs`): established failing against the unfixed tree (manifest dropped `{value}`/`{context}` and mis-shaped the member marker), promoted GREEN with assertions unchanged; constructor body statements asserted absent from every signature line.
- Harness hardening (same build): `CAPTURED_RESPONSES` consumers are now poison-tolerant via `protocol::captured_responses()` and drain pop-first-then-assert, so a genuine test failure can no longer poison the shared sink while holding its guard; and both Phase A/B retirement suites gate their dispatching tests on one shared `protocol::HANDLER_RESPONSE_SERIAL` (Phase B previously had no serialization at all, and Phase A's gate covered only its own file) — eliminating the empty-pop → `PoisonError` cascade observed under full-suite parallelism, where one test's clear/dispatch/pop raced a sibling's and three unrelated tests died on a poisoned lock.

### Verification

- compaction 87 passed; diff 62 passed (2 feature-ignored); gitdiff 41 passed; `cargo fmt --all -- --check` clean (two EOF-newline fixes applied via the formatter); UTF-8 guard PASS (473 files, strict UTF-8, 0 BOMs, 0 mojibake); `cargo test encoding` 6 passed; `cargo clippy --all-targets -- -D warnings` 0 warnings.

---

## [0.4.4] - 2026-08-27 - git_diff Signature-Bleed Fixes

### Fixed

- **Signature bleed from expression-bodied members in `git_diff` output.** Diffing a file whose expression-bodied member interpolates a primary-constructor parameter name (`public string Display() => $"Value: {Value}";` under `record Example(string Value)`) emitted corrupted change-set lines — the method name duplicated with `=>` and interpolated-string fragments welded onto the signature (`method Display Display():=> $"Value:`), plus a corrupted `~ class Example(string` label — while raw `git diff` was correct. Root cause: `extract_method_sig` treated the first `{` as end-of-header; expression-bodied members have none, so the interpolation hole became the boundary, truncating mid-literal. The signature span now ends at min(first `{`, first depth-0 `=>` outside string/char literals) via new `find_depth_zero_arrow` + `skip_quoted_literal` helpers (`src/compaction/method.rs`); paren depth keeps TS callback-typed parameters intact and unmatched quotes fail safe to the legacy behavior. Byte-identical output whenever no arrow exists.
- **C# primary-constructor parameter lists leaked into class labels.** After modifier/keyword stripping, whitespace tokenization turned `Example(string Value)` into the label `Example(string`, surfacing corrupted `~ class …` rows for every record or class with a primary constructor. New `strip_trailing_param_list` peels a trailing balanced `(…)` group using the EXISTING last-depth-0 locator `find_method_params` — activated only when the group closes out the declaration (empty tail or lone `;`), leaving `: Base(args)` and `where` clauses untouched — wired into both `extract_class_name` and `extract_class_meta` so name and base metadata derive from identical declaration text (`src/compaction/class.rs`).

Renderer/differ untouched; no fixture special-casing; no output deduplication.

### Tests

- RED→GREEN end-to-end regression `gitdiff_interpolation_does_not_bleed_into_signature_line` (`src/tests/gitdiff/engine.rs`): established failing against the unfixed tree (asserts the added member surfaces yet no `=>` or `$"Value:` fragment reaches any diff/signature line), promoted GREEN by these fixes with assertions unchanged.
- Unit-level regressions: `extract_method_sig("public string Display() => $\"Value: {Value}\";", Medium)` returns `Display()` with a brace-bodied control proving preserved legacy extraction; `extract_class_name("public sealed record Example(string Value)")` yields `Example`, and `class Foo<T>(int x) : Base` yields `Foo` with `:Base` carried in `class_meta` (`src/tests/compaction/method.rs`, `src/tests/compaction/class.rs`).

### Verification

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean for all changed targets (two pre-existing lints remain in unrelated `src/tests/encoding.rs`); compaction suite 86 passed; focused `compaction:: diff:: gitdiff::` suites 143 passed including the end-to-end regression; UTF-8 guard PASS (473 files, strict UTF-8, 0 BOMs, 0 mojibake).

---

## [0.4.3] - 2026-08-25 - `apply_edit` Write Path + Phase C1/C2 Legacy Retirements

Clean-CTX-native single-unit editing per `docs/plans/APPLY_EDIT_PLAN.md`: an agent that already received byte-exact bodies via `provide_code_context(fidelity="edit"|"verbatim")` can now write through Clean-CTX itself instead of paying the host's full-file raw-read precondition on every edit. Capability and tool-selection guidance shipped together (RULE 1b + SYSTEM_PROMPT rules 3-4) so the tool actually gets routed through.

**Phase C1/C2 (2026-08-26)** - `compress_workspace` and `decompress_code_context` retired entirely as MCP tools. Net -4748 lines across 39 files: inline dispatch arms, tool-list registration, handler functions, and all legacy-workspace-compression machinery removed. Remaining code references retagged as dead/stale (`#[allow(dead_code)]` with test-only notes). The Phase A changelog entry below stated "Remaining legacy emitters until Phases B/C: `compress_workspace`, `delta_text_context`" - Phase B removed `delta_text_context`; Phase C1/C2 completed the retirement by removing `compress_workspace` and `decompress_code_context`. The only remaining legacy-notation emitters are internal-only test helpers.

### Added

- **`apply_edit` MCP tool** (`src/mcp/tool_handlers/edit.rs`, schema + registry wiring): operations `replace_body` / `delete` (both requiring byte-exact `expectedOldText`) and `insert_after` / `insert_before` anchored to a named unit. Unit-granular optimistic concurrency — only the bytes being changed are verified, so two agents editing different units of one file no longer serialize. Hard pre-commit tree-sitter syntax gate (`has_error()`) rejects malformed splices before any byte hits disk. Minimal response (fileHash, version, per-op spans + byte deltas); `"verify": true` echoes new text back. Structured bounded mismatch payloads (`kind`, expected/actual snippets capped at 512 chars) on rejection — a rejection means the unit changed underneath the caller, never a silent overwrite. v1 policy: requires prior tracked state (`provide_code_context` first), refuses otherwise.
- **Span-tracked `CoreOp::Body`** (`(method_id, verbatim_text, start_byte, end_byte)` with a both-or-neither pairing invariant exposed via `body_span()`): bodies are now splice-addressable. `pipeline.rs` gained `locate_method_body()` returning `(text, offset-within-capture)` — C# attribute-strip aware — and emission threads absolute spans (`capture.start_byte + offset .. capture.end_byte`). Dual-shape wire compatibility: span-less bodies keep the legacy 3-tuple, spanned bodies emit 5-tuples; `tuple_to_op` accepts exactly 3 or 5 (single gate covering named/tagged/positional/string-table decoders). Binary wire bumped to **v0x03** (has-span flag varint + two raw offset varints); v0x01/v0x02 streams still decode (pre-span bodies decode span-less and are `apply_edit`-ineligible until recompressed). Hierarchical format carries spans as optional `bs`/`be` on `MethodNode`.
- **`src/edit/` pure write-path core**: `ops.rs` (serde-tagged operation model), `locate.rs` (`UnitTable` keyed on qualified name + structural fingerprint — containing class + ordered param types — with bare-name resolution only when unambiguous; span-less bodies excluded by design), `apply.rs` (expected-text verification, back-to-front splicing, disjointness validation, syntax gate). Deliberately free of `McpState`/I/O; the handler is a thin adapter.
- **Deterministic unit ordering**: unit tables materialize in instruction order (a HashMap-iteration nondeterminism was caught by the ambiguity test — candidate listings in `Ambiguous` error payloads are now stable across runs).
- **Tests** (`src/tests/edit/`, `src/tests/mcp/apply_edit.rs`, updates to opcodes/wire/binary_wire/hierarchical/round_trip/tool_handlers suites): wire round-trips both shapes + malformed-tuple rejection; legacy-version decode; fingerprint disambiguation of same-named methods; splice/delete/insert/mismatch/overlap unit coverage; in-process no-tracked-state refusal; black-box e2e over a persistent server session (provide → apply_edit → delta transport picks up the change; same-unit second edit rejected with structured mismatch; different-units independence) behind the existing `#[ignore]` e2e harness.
- **`examples/apply_edit_comparison.rs`**: measures the actual WRITE-side token delta of `apply_edit` (operations JSON) vs the read→edit convention (full raw read per edit) across the 50-edit simulation categories — the write-side numbers the plan flagged as unmeasured.
- **Guidance**: RULE 1b added to `docs/CLAUDE_INTEGRATION_RULES.md`; SYSTEM_PROMPT Edit-mode rules now steer single-unit edits to `apply_edit` (with host-tool fallback for signatures/imports/cross-file work).

### Fixed

- **Latent self-deadlock in the retired legacy-fallback branches of all three interactive tools** (surfaced BY this phase's fault-injected tests; never hit historically because the branches were unreachable): each fallback passed `state.dict_lock()` / `state.cache_write()` as arguments to the legacy compressor, and Rust keeps those guard temporaries alive for the ENTIRE `match` statement — so the Ok arm's `format_dict_footer()` (same dictionary mutex) deadlocked single-threaded on first execution. Live proof: injected IR failure stalled between `fallback OK len=306` and `footer done`. Branch removal (Phase A below) eliminates it by construction.

- **`replace_body` permanently rejected byte-exact body copies on Allman-style files** (reported on LF-only sources): `locate_method_body` backed up to the START OF THE LINE when `{` sat alone on its own line, embedding leading indentation in the tracked `CoreOp::Body` text/span. Any natural agent extraction (`{` through `}`) was then shorter by exactly that padding and failed the byte-exact comparison forever — identical across retries, LF/CRLF-independent, safety gate holding throughout. Fix at the single choke point (`src/ir/pipeline.rs`): body units are BRACE-DELIMITED — text and span start AT the opening brace; end remains the capture's closing `}`. Wire format unchanged; spans tighten. Regression: `src/tests/edit/spans.rs::lf_csharp_allman_attributes_spans_address_exact_disk_bytes` (RED pre-fix: expected 161 vs actual 165 = exactly the 4-space indent) plus an env-gated `probe_real_file` test for reconciling byte counts on real reported files.
- Side-discovery during investigation: `McpState::read_source` Phase 3 uses `cache.entry(k).or_insert(...)`, so an existing cache entry is never updated after a file changes — every subsequent read takes the stale branch and re-reads from disk (caching/perf defect only; returned bytes are always fresh). **FIXED in this release — see the source-cache bullet under Fixed.**
- **Line-ending-width rejections on CRLF files** ("actual == expected + number_of_newlines"): transport layers (editors/clipboards/LLM clients) normalize CRLF↔LF, so a content-identical body copy arrived with 1-byte separators against the file's 2-byte ones and failed the byte-exact gate — e.g. `expected 137 / actual 143` = exactly the body's 6 internal separators. Trace confirmed no `\r\n`-width arithmetic exists anywhere in span/length math (hierarchical copies op spans verbatim; outcomes echo measured widths); the divergence was purely representational at the comparison. Fix at the verification choke point (`src/edit/apply.rs`): (1) `verify_expected` compares content MODULO EOL width — mismatch payloads keep RAW bytes; (2) incoming `newText`/`unitText` is adapted to the FILE's measured EOL convention before splicing (`to_unit_eol`; separator-less units fall back to whole-file convention), so endings are never rewritten as a side effect and never mixed, and `byteDelta`/outcome spans reflect the adapted widths. Genuine content changes are still rejected regardless of EOL. Regression: `edit::spans::{crlf_file_accepts_lf_normalized_copy_and_preserves_crlf_on_disk, lf_file_accepts_crlf_padded_copy_and_preserves_lf_on_disk, content_changes_are_still_rejected_regardless_of_eol}` (first two RED pre-fix with the newline-count delta).
- **`read_source` source-cache permanently self-defeating after any external file modification** (the side-discovery ticket above, RED → trace → GREEN): Phase 3 used `cache.entry(k).or_insert(entry)`, a NO-OP whenever the key already exists — precisely the stale case Phase 1 had just detected via mtime/size mismatch. After any write bypassing `invalidate_source_cache` (host-editor saves; only `apply_edit` invalidates), read #2 detected stale → re-read fresh bytes → failed to refresh the entry → reads #3…∞ repeated identically: permanent cache-miss (stat + full disk read + double lock acquisition per read) with the stale content `Arc<String>` pinned in memory indefinitely. Returned bytes were always fresh, so this was caching/performance/memory, never correctness. Minimal choke-point fix at `src/mcp/state.rs` Phase 3: plain `cache.insert(...)` overwrites the entry; two-phase locking, lock scope, and `CacheEntry` shape unchanged. Regressions (`src/tests/mcp/state.rs`, all RED pre-fix where marked): `read_source_third_read_is_served_from_the_refreshed_cache_entry` (behavioral: `Arc::ptr_eq` across an external modification — failed pre-fix because every read re-allocated), `read_source_phase3_updates_existing_entry_metadata_and_content` (mechanism: stored entry must hold current content/size/mtime — failed pre-fix with the exact stale-content diff), and `read_source_returns_fresh_bytes_after_external_modification` (freshness contract pinned; passed pre- and post-fix by design).

### Changed

- **Phase B — `delta_text_context` / `TextDelta` / `§Δ` transport removed entirely**: the MCP tool, its handler (`handle_delta_text_context`), schema entry, registry wiring, `compression::text_delta` module (TextDelta/TextDeltaComputer/apply_text_delta/§Δ wire codec), the orphaned `compress_text_body` helper, and `McpState::text_delta` are gone. `heuristics::decide` dropped its TextDelta parameter (strategy now keyed solely on IR baselines); `context_history` reports IR versions instead of text-delta versions. ~40 legacy-transport tests deleted or converted to IR-seeded baselines; new contract suite `tests/mcp/phase_b_retirement.rs` pins unregistration, catalog absence of `§Δ`, and the surviving `provide/compress → IRDelta → apply_delta` flow. Docs updated (README, ARCHITECTURE_OVERVIEW, COMPILER_IR, CONFIGURATION, meta-layer scope notes, invariants wording). IR-native delta machinery (IRDelta, ContextState, replay, apply_delta, SQLite persistence) untouched.
- **Phase A — legacy `$`/`⊕`/`§` notation retired from all interactive surfaces**: `compress_code_context`, `provide_code_context`, and `restore_context` return a structured JSON-RPC error on IR-compilation failure — `{ code: -32603, message: "IR compilation unavailable for <path>: <reason>…", data: { reason: "ir_unavailable", path, ir_compiler } }` — instead of silently degrading to legacy text. SYSTEM_PROMPT dropped the Legacy Notation tables entirely (Rules 1/3 rewritten; intro states the single-notation contract); `clean-ctx-vocabulary` prompt rebuilt around SCHEMA v2 symbols plus current α/Φ systems (`generate_vocabulary_text`; both description strings updated). README legacy subsection carries the new boundary note. Remaining legacy emitters until Phases B/C: `compress_workspace`, `delta_text_context`. Contract pinned by `src/tests/mcp/phase_a_retirement.rs` using cfg(test)-only fault injection (`TEST_INJECTED_IR_FAILURE`) and a response-capture sink (`protocol::CAPTURED_RESPONSES`); written RED-first — pre-fix runs showed legacy payload emission and stale prompt/vocab tables.

- **Concurrency deviation from plan**: the plan's "acquire the same RwLock" deadlocks today — `compile_file_ir_focused` internally takes an `ir_context` READ lock via `state.file_version()`, so a caller cannot hold that lock's WRITE guard across compilation. Commits serialize through a module-local mutex instead (same guarantee, no lock-order change). Documented in-handler and in the plan's new Implementation Notes.
- **Relocation strategy**: the whole-file hash fast path is subsumed — every call relocates against a fresh Edit-fidelity compile of current bytes (strictly safer; costs one local parse, not client tokens). Session baseline refresh post-commit (ir_context `load_ir` + hash registry + source-cache invalidation + llm-text eviction) makes the next `provide_code_context` a delta.
- **Open Question resolutions**: OQ2 — prior tracked state IS required (new files go through the host write tool); OQ3 — SQLite baseline persistence deliberately deferred to the next `provide_code_context` call (existing fire-and-forget pattern; avoids writing empty-baseline rows).

### Verification

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets --workspace -- -D warnings` zero warnings.
- Full workspace suite green under `--all-features`, including the new `src/tests/edit/` suites (apply/locate/ops plus the 6-test `edit::spans` span-invariant & boundary suite — 5 always-run + 1 env-gated probe) and the black-box `apply_edit` e2e suite; two pre-existing flaky live-CBM pipe tests confirmed environmental (orphaned subprocess) and green serially after cleanup.
- Follow-up fix cycle (Allman body boundaries): RED reproduced first (`expected 161 bytes, actual 165 bytes` = exactly the 4-space indent), GREEN after the brace-delimited choke-point fix; focused `edit::spans` and full gate re-verified.
- Second follow-up fix cycle (EOL-width rejections): RED reproduced first (`expected 137 / actual 143` = exactly the body's 6 internal separators counted 2-vs-1 bytes), GREEN after EOL-insensitive verification + file-convention splice adaptation; `edit::spans` grew to 9 tests (3 transport regressions incl. content-change guard); full gate re-verified clean after clippy doc-lint fixes. Suite totals moved 2,513 → 2,522 workspace (2,173 → 2,182 core).
- Cache-refresh fix-cycle gate: focused `mcp::state` 11/11 (RED first: 2 failed / freshness probe passed); `cargo fmt --all -- --check` clean (one rustfmt nit fixed in `tests/mcp/prompts.rs`); `cargo clippy --workspace --all-targets --all-features -- -D warnings` zero warnings; full lib suite 2,235 passed / 8 ignored with exactly 2 failures, both classified ENVIRONMENTAL: `cbm::tests::e2e::live_proxy_exercises_all_cbm_tools` (external codebase-memory-mcp binary's named pipe dropping mid-run — `ConnectionLost os error 232`, circuit breaker opening `circuit_open_after_3_failures`; `src/cbm/` IPC shares no code path with `McpState::read_source`, and the same pipe-loss JSON errors appear on sibling tests in isolation) and `e2e_proxy_handler_returns_compressed_result` (PoisonError cascade from the former's panic under parallel execution; PASSES on isolated rerun). No failure touches the source-cache subsystem.
- Phase-B transport-removal gate: RED first — `phase_b_delta_text_context_is_no_longer_a_registered_tool`, catalog-absence, and `§Δ`-marker contract tests all failed against the pre-fix tree while the IR end-to-end guard passed untouched; GREEN focused set 55/0; `cargo fmt --all -- --check` clean after repairing layout drift in script-edited files; `cargo clippy --workspace --all-targets --all-features -- -D warnings` zero warnings; full lib suite 2,222 passed / 8 ignored with the single documented environmental CBM pipe flake recurring unchanged (unrelated `src/cbm/` IPC subsystem). Executed-test total moved 2,235 → 2,222, accounting for ~40 deleted/converted legacy-transport tests net of the new Phase A/B contract suites.

### Documentation

- **SYSTEM_PROMPT notation corrected to match delivered output.** Investigation confirmed the primary LLM-facing path renders SCHEMA v2 (`X`/`M`/`F`/`I` structure letters, `fl:` flags, High-fidelity `cf:/df:/se:/ec:`, verbatim bodies at Edit) while the prompt taught the retired `$`-opcode/`⊕`-marker tables from the legacy text compressor. SYSTEM_PROMPT now teaches SCHEMA v2 as PRIMARY; the `$` primitive table and `⊕` markers moved into an explicitly scoped "Legacy Notation (text-compressor pipeline)" section naming their only producers (`compress_workspace`, `delta_text_context`) and fidelity behavior (`⊕` at Medium/High, `§` micro-codes at Low). Example replaced with a real SCHEMA v2 fragment. Contract regression tests added (`src/tests/mcp/prompts.rs`, wired as `prompts_tests`): legend presence, High/Edit coverage, and ordering enforcement that every retired token appears only after the legacy-section header — silent drift back to the retired vocabulary now fails CI.
- README "Opcode Reference" split into **Response Notation (SCHEMA v2 — primary)** + scoped **Legacy notation** subsection (tables preserved for decoding); MCP Prompts bullets updated likewise.
- `ANGULAR_META_LAYER.md` / `DOTNET_META_LAYER.md` prefix tables annotated with notation-scope callouts (legacy vs SCHEMA v2 vs current `Φ` meta vocabulary).
- README version banner and QA-gate table test counts refreshed to 2,522 / 2,182 (date-stamped 2026-08-25) and R-45 `apply_edit` added to the feature list.

---

## [0.4.2] - 2026-08-25 - Non-CBM Tool Audit Fix Cycle

Evidence-driven fix cycle over the 2026-08-25 non-CBM tool audit. Every fix was reproduced with a failing regression test before implementation.

### Fixed

- **`diff_commits` emitted access modifiers as changed-class labels** (`~ class internal`, `~ class public`). Two compounding label-derivation defects: (1) `MODIFIERS_CLASS` lacked `internal `, so `strip_modifiers` stopped immediately on `internal static class Foo` and the first whitespace token became the "name"; (2) the diff snapshot builder routed `struct.root`/`enum.root` through `extract_rust_struct_name` for ALL languages — that helper only strips Rust visibility prefixes, so `public enum Foo` labeled as `public`. Fixes: `internal ` added to `MODIFIERS_CLASS`; `enum `/`struct ` keyword stripping added to `extract_class_name`/`extract_class_meta`; `diff::builder::try_build_with` now receives its parser label and routes non-Rust struct/enum/trait/impl through the shared class-name extractor while Rust keeps byte-identical behavior. Unchanged-class rendering untouched.
- **`compress_workspace` at Low fidelity lost method identifiers** for C# signatures like `internal static async Task<(A section, …, Guid requestId)> CreateRecordWithDefaults(…)`, rendering them as `internal static async Task(scope)`. Mechanism: with a named tuple return type whose last element is lowercase, `is_csharp_return_type(tokens[len-2])` misfires and the fallback split the WHOLE signature at the first `<`, yielding the type prefix as the "name". Fix: when the parameter list is located structurally (`find_method_params`), the name is the last whitespace token before it regardless of naming convention; the no-parameter-list legacy fallback is preserved; Medium/High/Edit outputs are pinned unchanged by test.
- **Alias registry path fragmentation**: the same physical file could hold two aliases (visible as duplicate `α` entries in `§PATHMAP`) because alias keys used the raw caller-supplied string while handlers mix absolute and workspace-relative spellings. Fix: `PathDictionary::get_or_create_alias` now resolves keys via canonicalize-or-fallback (Windows verbatim `\\?\` prefix stripped for readable PATHMAP output); exact-string repeats fast-path without filesystem access. Alias-keyed state (IR context versions, text-delta baselines, LLM text cache) converges onto one identity per file.
- **`decompress_code_context` destroyed class boundaries on round-trip**: the blanket skip-all-`//`-comments rule removed the IR renderer's structural `// ── ClassName ──` markers, turning a multi-class skeleton into an unattributed flat field list. Boundary markers are now preserved verbatim; opcode expansion around them is unchanged.
- **`list_sessions` returned a static `"Persistence DB active."` status line** despite promising an enumeration. The persistence model has NO session concept (the `sessions` table is dead schema nothing writes), so instead of inventing session data the tool now lists what genuinely exists: one row per persisted file context (path, fidelity, token counts, delta count, last-update timestamp) via new `SqliteStore::list_contexts()` + `BufferedStore` passthrough; description corrected accordingly.
- **`context_stats` clamped negative savings to `0.0%`**: compression can legitimately COST tokens (small file / unfocused edit fidelity). Per-file, domain-aggregate and session-total percentages are now signed — the dashboard shows the true negative figure while raw/compressed token measurements are untouched.
- **`diff_commits` attributed enclosing-class methods to nested declarations**: language queries capture class/struct/enum/record declarations at ANY nesting depth, but the snapshot walker attached every method via `classes.last_mut()` — so after a nested `enum`/`class`, ALL subsequent methods of the enclosing class were owned by the nested type and rendered as `~ class OrderStatus` instead of `~ class OrderService`. Fix (structural): `CapEntry` now carries `end_byte`; the walker keeps an open-scope stack keyed by source span and attaches members to the INNERMOST declaration whose span CONTAINS them (a nested type that closed before the member starts can never own it). Nested types keep their own correctly-labeled entries; top-level/nested labels from finding #1 are pinned unchanged.
- **`SessionStats` fragmented per-file tracking across path spellings**: `files` was keyed by the RAW string passed to `record_compression()`, so absolute / workspace-relative / redundant-segment spellings of ONE physical file produced separate rows and double-counted session totals (the long-documented CONTEXT_STATS plan issue #2). Fix at the choke point: `record_compression`/`file_stats` now resolve through the shared `dictionary::path::canonical_identity_key` (exact-string hits fast-path without filesystem access), giving stats the same canonical identity as the alias registry.

### Changed

- **`delta_text_context` contract made explicit**: the tool is a text-based diff strategy for the SAME source-code registry as `delta_code_context` — not a generic text tool. Description and README row updated to state the code-only scope; no behavior change.
- Documented (no code change) that `diff_code_context` maintains its own baseline local to itself (canonical path + fidelity), independent of `provide_code_context`/`compress_code_context` (audit finding #8), and that bare `F` field markers at low fidelity remain intentional compression semantics, not a defect (audit finding #7).

### Added

- Regression suites: gitdiff end-to-end label tests (changed internal class + unchanged sibling in one file); compaction tests for modifier/keyword extraction and compound-signature fidelity pins; diff-builder language-split tests (C# internal class, C# enum, Rust enum guard); alias identity convergence + unresolvable-fallback tests; decompression multi-class boundary round-trip; `SqliteStore::list_contexts` store tests; `src/tests/mcp/tool_contracts.rs` pins locking both corrected tool descriptions and the enumeration behavior.

### Verification

- Focused regression suites green under default features AND `--features rust` (the diff-builder module is feature-gated).
- Full gate: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, complete workspace test suite.

---

## [0.4.1] - 2026-08-24 - Typed graph_query Edge Extraction

### Fixed

- **Typed `graph_query` reported "N node(s), 0 edge(s)" for every relationship-returning Cypher.** `GraphBridge::query_graph()` read only column 0 of each result row into nodes while its `edges` field was a literal empty vec — despite the doc comment claiming rows were interpreted "as either node or edge data". Live side-by-side on an indexed project: the same `MATCH (a)-[r:CALLS]->(b) RETURN a.name, type(r), b.name LIMIT 5` returned full CALLS rows through `cbm_proxy(query_graph)` while the typed path collapsed them into duplicated column-0 nodes. The client (`CbmClient::query_graph`) was always correct; the bridge now converts via pure `convert_query_rows()`. Defect previously documented as FAANG audit M-03 and CBM_API_AUDIT open question #4; never implemented until now.

### Added

- Column-shape wire convention (**invariant `CBM-WIRE-002`**, docs/ARCHITECTURAL_INVARIANTS.md): a projection is relationship-shaped IFF it contains exactly ONE echoed literal `type(...)` column (whitespace-tolerant — CBM echoes `"type( r )"` verbatim). Relationship projections become edges: endpoints = FIRST and LAST non-type projected columns (projection order rules), type cell → `label`, every other projected column → `GraphEdge.properties` keyed by echoed column text with values preserved verbatim. Scrambled 5/6/N-column orders resolve purely from column metadata. ALIAS PIN: an `AS` alias REPLACES the whole expression in the echo (`type(r) AS rel_kind` ⇒ `"rel_kind"`), making aliased type() projections intentionally indistinguishable from ordinary scalars at the typed layer — they fall back to nodes by design and must never be reverse-engineered. Undirected `-[]-` projections supported and return all relationship types verbatim.
- Regression suite `src/tests/cbm/query_wire.rs` (18 tests): verbatim raw-capture fixtures for EIGHT verified shapes (directed CALLS, undirected mixed DEFINES/DECORATES/USAGE, qualified endpoints, aliased-type, 5-column, 6-column scrambled with trailing file_path, type-first, numeric triple `[name, in_degree, out_degree]`, whitespace variant) + policy pins (numeric-triple-never-fabricates, multiple-type-columns refuse to guess, row/column misalignment fallback, empty results, duplicates pass through untouched) + four fresh-process `serial(cbm_live)` probes over a SYNTHETIC fixture repo (3-column baseline edge, wide scrambled projection with property mapping, aliased+numeric fabrication guards, node-only control).

### Fixed (during this cycle)

- The first implementation of this fix used a strict positional/arity rule (exactly-three-cell uniform rows ⇒ `[from, type, to]`). Live shape auditing proved that rule semantically dangerous: a uniform numeric triple like `RETURN f.name, f.in_degree, f.out_degree` would fabricate an edge labelled `"10"`. Retired before release in favor of column-shape-driven extraction; `CbmClient::query_graph` now returns the full `{columns, rows}` table (`QueryRows`) so callers can interpret the semantic projection instead of guessing from arity.
- **CI flake (`decompression` proptest, run 32805773191)** — unrelated to CBM work, pre-existing: `word_boundary_replace_never_panics` asserted `!result.is_empty() || text.is_empty()`, a FALSE invariant — an empty replacement over a pattern covering the whole text legitimately yields `""` (removal semantics pinned by its own sibling test). Proptest's per-run randomized inputs eventually drew the counterexample on linux. The function was never wrong; the property was. Corrected to assert the actual contract (no panic + sound size bound); verified green at 5,000 cases per property.

### Changed

- Relationship-shaped projections now report their edges INSTEAD of column-0 nodes (column-shape semantics); no-type(...) projections keep the legacy node mapping byte-for-byte REGARDLESS of column count. Node deduplication, file-path population, and endpoint normalization are deliberately NOT included — tracked as separate findings. No public API, cache-key, or compression changes; cached `cypher:*` entries remain deserializable (populated edges reuse the existing serialized `edges` key).

### Verification

- Fresh-process live probes green over synthetic fixture repos: typed CALLS projections surface edges end-to-end (endpoint cells exactly as projected — bare under `.name`, qualified under `.qualified_name`), wide scrambled 4-column projection maps middle columns into properties, aliased type() and numeric-triple projections stay node-shaped; verbatim raw-capture fixtures pin the column-shape conversion deterministically across all eight captured shapes.
- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` zero warnings.
- `cargo test --workspace --all-targets --all-features`: **2,513 passed / 0 failed / 5 ignored** (core library 2,173 + CLI binary 11 + proxy crate 329: lib 155, bin harness 155, audit-regression 18, e2e integration 1), including `e2e_cbm_multiroot_multilingual_integration`.

### Test-output noise reduction

- Commented out per-run debug output in the test fixtures: the multilingual audit probe's Step 1–16 narration and result dumps (`src/tests/cbm/e2e.rs`), path-resolution dumps (`debug_bundler_paths`), AST-dump tests (`dump_html_ast`, `dump_angular_template_ast` — removed from the active count, −2), the P0-1 dispatcher progress line, and the `[TIMING]` compression line. Root cause of the largest flood fixed structurally: three `observability::tracing` tests raced to install a process-global DEBUG-level tracing subscriber for the whole suite; their env-mutating bodies are commented out with rationale. Skip/error-path diagnostics (e.g. "Skipping — CBM not installed") intentionally retained.

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
| 0.5.1 | 2026-08-29 | **graph_query node-result data fidelity.** Node-shaped Cypher projections (`RETURN f.name, f.file_path`) no longer collapse to column-0-only nodes with hard-coded empty `file`/`label` and dropped columns: `file_path` populates `GraphNode.file`, a recognized `label` column populates `GraphNode.label`, and extra projected columns are preserved in `GraphNode.properties`; the query-cache namespace was bumped `cypher:` -> `cypher2:` to invalidate pre-fix entries. 0.5.0 conformance fix — no MCP schema change |
| 0.5.0 | 2026-08-28 | **MCP Structured-Output Migration.** Canonical `content` + `structuredContent` + `_meta` response envelope across MCP tools; `graph_search` reference plus `graph_query`/`graph_trace`/`get_architecture`/`apply_edit` gain `structuredContent` + `outputSchema`; remaining eight tools' ad-hoc metadata moved to `_meta` (`save_context`/`purge_old_deltas` gained `content`); JSON-RPC error paths preserved unchanged; shared `crate::tests` envelope helpers consolidate Phase 1–4 coverage |
| 0.4.6 | 2026-08-27 | **Constructor Initializer Mis-Tokenized as Parameter Group.** `find_method_params` selects the FIRST name-anchored balanced depth-0 paren group (preceded by `>` or an identifier other than `base`/`this`, scanned literal-aware) — `: base(...)`/`: this(...)` call sites are no longer chosen as the parameter list: Low/Medium labels, diff grouping, IR constructor identity, initializer-sensitive body fingerprints, and the class primary-constructor peel guard all corrected; new `strip_base_initializer_clause` wired into Medium compaction and `parse_method_sig`; tuple-return behavior and High byte-exact output preserved; the ff2a29a Medium tier deliberately drops the clause while High keeps it byte-exact — compaction 95, diff 80, gitdiff 42, ir 32 passed, 0 clippy warnings |

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
| 0.4.5 | 2026-08-27 | **Constructor Base-Initializer Signature Truncation.** `find_depth_zero_brace` (sibling of `find_depth_zero_arrow`, reusing `skip_quoted_literal`) locates the body brace only at structural depth 0 outside quoted/interpolated literals — interpolation holes in `: base($"Unexpected value: {value}")` no longer truncate constructor signatures onto diff labels; RED→GREEN e2e regression kept assertion-exact — focused compaction/diff/gitdiff suites 190 passed, 0 clippy warnings |
| 0.4.4 | 2026-08-27 | **git_diff Signature-Bleed Fixes.** Expression-bodied members no longer leak `=>`/interpolation fragments onto change-set signatures (`find_depth_zero_arrow`+`skip_quoted_literal` min-boundary slicing); C# primary-constructor parameter lists no longer corrupt class labels (`strip_trailing_param_list` peels the trailing balanced group via existing `find_method_params`); RED→GREEN end-to-end regression kept assertion-exact — focused compaction/diff/gitdiff suites 143 passed, 0 clippy warnings on changed targets |
| 0.4.3 | 2026-08-25 | **Clean-CTX-native `apply_edit` Write Path.** Single-unit byte-exact editing (replace_body/delete/insert_after/insert_before) with unit-granular optimistic concurrency; EOL-preserving expected-text verification; `read_source` permanently-self-defeating cache entry fixed (stale `Arc<String>` pinned forever after external writes); Phase A legacy notation retired from interactive surfaces (structured `ir_unavailable` errors); Phase B `delta_text_context`/`§Δ` transport removed entirely |
| 0.4.2 | 2026-08-25 | **Non-CBM Tool Audit Fix Cycle.** Access-modifier class-label corruption + nested-declaration method attribution fixed in diff_commits; compound-signature identifier recovery at Low fidelity; canonical path identity unifies alias registry & SessionStats; decompression preserves class-boundary markers; `list_sessions` enumerates persisted file contexts; `context_stats` signs true negative savings — every fix RED-first |
| 0.4.1 | 2026-08-24 | **Typed graph_query Edge Extraction.** Relationship-shaped Cypher projections convert into edges instead of duplicated column-0 nodes; column-shape wire convention invariant `CBM-WIRE-002` + 18-test `query_wire.rs` suite + fresh-process live probes; CI decompression-proptest flake root-caused to a false property — corrected contract verified green at 5,000 cases |
| 0.4.0 | 2026-08-24 | **CBM Graph-Intelligence & Trace-Wire Audits.** Blast-radius CALLS-edge fail-open fixed; Result-propagating intel APIs (`Ok(empty)` vs `Err(ToolError)`); dual-label dead code; dead DATAFLOW path removed; typed `graph_trace` parses the real callers/callees wire contract (phantom `edges` key eliminated); inbound-fallback direction determination; M-01 qualified/bare boundary normalization — invariant `CBM-WIRE-001`, 16-test `trace_wire.rs` suite, 2,497 tests, 0 clippy warnings |
| 0.3.1 | 2026-08-23 | **C-22 Canonical Source-Span Contract & Multi-Class Fix.** Class source spans derived from capture identity rather than `DefClass.name` (`class_source_from_capture()` trilingual decorator scan); marker cross-contamination eliminated in multi-class Spring Boot/Angular/.NET fixtures (9 isolation tests) |
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