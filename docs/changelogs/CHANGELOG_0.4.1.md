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

