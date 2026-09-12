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

