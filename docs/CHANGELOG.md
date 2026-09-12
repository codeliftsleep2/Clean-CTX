# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Historical releases are archived per version as `CHANGELOG_<version>.md` (`_a`/`_b` suffixes mark split sub-sections of one release); the version-history registry lives in [`CHANGELOG_VERSIONING.md`](CHANGELOG_VERSIONING.md).


---

## [0.6.3] - 2026-09-11

### Added

* **Bounded hydration for `workspace_query`** — for query types carrying a searchable entity name (`find_entities`, `forward_edges`, `reverse_edges`, `transitive_dependencies`), ONE bounded CBM candidate-file discovery pass runs per request after the initial WorkspaceIndex query — eligibility is evaluated from query-type identity, independent of initial result cardinality (RED-9 partial-nonzero hydration; RED-10 fresh-index hydration). CBM supplies candidate file paths ONLY (extracted from `GraphNode.file`; CBM edge counts, relationship types, direction, hop/depth, and result cardinality are discarded before crossing the hydration boundary). Candidate paths flow through `resolve_file_path_checked` → `compile_file_ir_focused` → Clean-CTX semantic extraction → `semantic_edges` → `WorkspaceIndex` — there is no path from a CBM relationship to a WorkspaceIndex relationship without intervening Clean-CTX compilation. At most 5 previously-unindexed candidates are compiled per request (deduplicated, already-indexed excluded, deterministic lexical ordering — a bounding mechanism, not a relevance claim). The original WorkspaceIndex query reruns exactly once after hydration; there is no second hydration cycle. (`src/mcp/tool_handlers/query.rs`, `src/workspace/index.rs`)
* **Hydration metadata in `workspace_query` responses** — `hydration_attempted`, `candidates_discovered`, and `candidates_compiled` in `structuredContent` across all six query types. `entities_in_file` and `has_cycle` report `hydration_attempted: false` (they provide no entity/symbol identity for bounded candidate discovery — no guessed or broadened CBM search). Hydration failure, CBM unavailability, candidate rejection, or compilation failure degrades gracefully to the WorkspaceIndex evidence already available. Coverage remains partial — hydration improves evidence and never establishes repository-wide completeness. (`src/mcp/tool_handlers/query.rs`)
* **Constituent invariants + discovery record** — WSC-001 (authoritative facts do not imply authoritative coverage: absence from a partially populated `WorkspaceIndex` is never confirmed absence from the workspace; result cardinality never drives hydration decisions or completeness claims) and WSC-002 (CBM discovery may influence compilation scope; Clean-CTX extraction alone determines WorkspaceIndex semantics; candidate-file cardinality is not result cardinality) recorded in `docs/ARCHITECTURAL_INVARIANTS.md`; field finding recorded as DIS-2026-005 in `docs/agent/DISCOVERY_REGISTRY.md`. `workspace_query` tool contract updated in `docs/agent/tooling.md`.
* **Configured multi-root hydration discovery** — bounded `workspace_query` hydration now searches each CBM project already associated with the workspace (primary root plus every valid configured `additional_roots` entry) through a project-explicit bridge search that never mutates the active project. Project-relative `GraphNode.file` values are anchored to the canonical root mapped to the project that returned them; only normalized file identities cross the CBM boundary. All project candidates are merged before deduplication, already-indexed exclusion, one deterministic lexical sort, and the unchanged global five-file cap. Per-project search/readiness failures degrade locally and remaining projects continue. (`src/cbm/project_search.rs`, `src/mcp/tool_handlers/hydration.rs`)

### Fixed

* **CBM symbol importance score capping** — `CbmClient::get_symbol_importance` normalized the score as `in_degree / 100.0`, which exceeded the documented `[0.0, 1.0]` contract for hot symbols (e.g. `dispatch_tools_call` scored 1.08 at in_degree 108); the score is now capped at 1.0. Found in live testing (`live_symbol_importance_scores_are_nonzero_and_bounded`). (`src/cbm/client.rs`)
* **Test-harness poison-tolerant `pop_response` hardening** — the `CAPTURED_RESPONSES` response-sink drainers in `src/tests/mcp/workspace_query.rs` and `src/tests/mcp/workspace_query_2.rs` acquired the sink lock and called `.pop().expect(...)` in one held-guard expression. A panic while holding that guard poisoned the process-global sink and cascaded `PoisonError` failures across unrelated tests under full-suite CI parallelism (15 unrelated tests died on the poisoned lock, including pre-existing `economics_*` regressions in `src/tests/mcp/tool_handlers.rs` and the `transitive_dependencies` envelope tests that never touch hydration). Both `pop_response` helpers now match `CAPTURED_RESPONSES.lock()` directly, pop into a local, `drop(guard)` **before any panic**, and only then report the `Some`/`None` result — so no test can poison the sink while holding its guard. Missing-response failures still fail honestly; sticky poison no longer propagates from a single failure to the entire suite. (`src/tests/mcp/workspace_query.rs`, `src/tests/mcp/workspace_query_2.rs`)
* **Thread-isolated handler response capture** — full-suite validation exposed a second test-only race: parallel handler tests could clear or pop another thread's response from the process-global `CAPTURED_RESPONSES` vector. The existing lock-shaped API now fronts a per-test-thread queue, preserving every caller while preventing cross-test response theft; production stdout serialization and JSON-RPC output are unchanged. (`src/protocol.rs`, `src/tests/protocol.rs`)
* **`reverse_edges` inbound CBM query syntax** — the inbound-reference discovery query used `WHERE type(r) <> 'DEFINES' AND type(r) <> 'DEFINES_METHOD'`, which CBM's Cypher parser rejects (parser error lands on the `type(` expression). The query was rewritten to project `type(r)` in `RETURN` and filter client-side in a new `filter_inbound_reference_row` helper that discards `DEFINES`/`DEFINES_METHOD` rows. This eliminates the repeated per-project query failures that opened the shared CBM circuit breaker after three failures, disabling unrelated CBM-backed operations for the rest of the session. CBM still contributes candidate file paths only; relationship type is used solely to discard definition-only rows and never enters WorkspaceIndex as a semantic edge. (`src/cbm/project_search.rs`, `src/tests/cbm/project_search.rs`)


### Tests

| Area | Count |
|------|------:|
| `src/tests/mcp/workspace_query_2.rs` — RED-9 partial-nonzero hydration (non-zero initial result remains hydration-eligible; authoritative initial result survives the rerun), RED-10 fresh-index hydration (empty index is hydration-eligible; candidate compiled through the production path), RED-11 CBM authority/cardinality isolation (CBM relationship count never determines `workspace_query` results), RED-12 hard hydration bound (7 candidates > cap; ≤5 compiled; deterministic selection; no second cycle), RED-13 candidate with no matching Clean-CTX relation (zero fabricated relationships), RED-14 per-query-type hydration-eligibility classification (4 eligible types hydrate exactly once; 2 non-eligible types never hydrate) | 6 tests |
| `src/tests/mcp/workspace_query_3.rs` — RED-15 additional-root-only discovery, RED-16 active-project preservation on success/partial failure/empty/all-error exits, RED-17 merged-pool global cap/dedup/index exclusion/deterministic ordering, RED-18 per-project failure isolation | 4 tests |
| `src/tests/cbm/project_search.rs` — configured-project enumeration and project-explicit search state preservation | 2 tests |
| `src/tests/cbm/project_search.rs` — RED-R1 generated inbound query is CBM-compatible (no `WHERE type(r)`); RED-R2 definition relationships (`DEFINES`/`DEFINES_METHOD`) filtered client-side | 2 tests |

| `src/tests/protocol.rs` — deterministic parallel response-capture isolation | 1 test |

Regressions are exercised through the MCP dispatch boundary using a `cfg(test)`-only candidate-path injection seam (`TEST_HYDRATION_CANDIDATES`, following the `TEST_INJECTED_IR_FAILURE` pattern). The seam injects candidate file paths ONLY — never semantic edges, entities, precompiled IR, or results — so injected candidates flow through the full production hydration path: discovery → dedup/order/bound → `resolve_file_path_checked` → `compile_file_ir_focused` → Clean-CTX semantic extraction → `WorkspaceIndex` → original query rerun. Hydration regressions are serialized (`TEST_SERIALIZE`) because the injection seam and the global `LayerRegistry` are not safe for concurrent access across threads.

### Verification

- Hydration suite + symbol-importance focused run: **7 passed, 0 failed** (serialized).
- Full library suite: **2296 passed, 0 failed** — 2 live-CBM environmental tests (`e2e_apply_edit_triggers_reindex_and_graph_is_fresh`, `live_proxy_exercises_all_cbm_tools`) pass standalone; parallel full-suite CBM live contamination is a known environmental limitation (DIS-2026-005 live scenario).
- `reverse_edges` query-compatibility regressions: **RED-R1, RED-R2 GREEN** (generated query is CBM-compatible; definition relationships filtered client-side).

- `cargo clippy --all-targets --all-features -- -D warnings` — zero warnings.
- `cargo fmt --all -- --check` — clean.
- Multi-root hydration, project-explicit search, response-capture, and complete `workspace_query` targeted suites — all passed.
- Full suite — all non-environmental tests passed. The documented grouped-run `live_proxy_exercises_all_cbm_tools` CBM pipe failure passed when rerun standalone.
- `scripts/check-utf8.ps1` — PASS (507 text files valid UTF-8, 0 BOMs, 0 unexplained mojibake or letter-substitution signatures).
- Rust encoding suite — 7 passed, 0 failed.
- `cargo test encoding` — PASS.

---
