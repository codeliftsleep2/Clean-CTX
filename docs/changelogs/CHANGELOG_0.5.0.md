---

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

