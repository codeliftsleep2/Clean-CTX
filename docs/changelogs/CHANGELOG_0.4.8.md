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

