<!-- pcg:family-standing:begin -->
# Invariant family: cbm

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

<!-- pcg:family-standing:end -->
<!-- pcg:INV.CBM_ID_001:begin -->
### CBM-ID-001 Canonical CBM Project Identity & Multi-Root Lifecycle

| Property | Value |
|----------|-------|
| **Intent** | Every CBM interaction — indexing, readiness, querying, proxy routing, and cache partitioning — must address the project CBM actually indexed, regardless of how many repos are configured. |
| **Invariant** | **Never derive or invent a CBM project identifier independently of the canonical-root mapping.** CBM's canonical project slug is the single source of identity for indexing, readiness, querying, proxy routing, and cache partitioning. Specifically: (1) A CBM project identity is the slug derived from the canonical repo path (`cbm_project_slug()`), never a directory basename. (2) Every configured root (primary + `additional_roots`) maps to its own CBM project ID via the bridge's two-way identity map (`project_ids` / `project_paths`). (3) One CBM subprocess serves all configured roots. (4) Indexing begins asynchronously at bridge construction for every root (`start_indexing_roots()`). (5) Indexing/readiness state is tracked independently per CBM project; untracked projects pass through as ready rather than dead-ending in a permanent gate. (6) Graph queries and `cbm_proxy` resolve targets through the root/project mapping (`resolve_project_id`) and never invent a dirname-based identity. An exact configured-root basename is an accepted alias, but `cbm_proxy` MUST rewrite that alias to the canonical slug in the actual arguments forwarded to CBM; resolving only the readiness gate is insufficient. Unknown or partial names remain literal and CBM rejects them authoritatively. (7) Project-independent CBM tools (e.g. `list_projects`) bypass the indexing gate entirely. (8) The verified CBM 0.8.1 wire contract is preserved: `index_repository(repo_path, mode)` takes no project parameter — CBM derives the ID from the canonical path. |
| **Enforcement** | Regression tests covering: slug fidelity against live-captured CBM responses; per-root registration for primary + additional roots; dirname/path overrides canonicalizing instead of diverging; per-project readiness isolation with untracked pass-through; single-root backward compatibility; proxy gate scoping (project-less calls skip the gate). |
| **Authority** | `src/cbm/bridge.rs` (`cbm_project_slug`, `try_create_with_roots`, `resolve_project_id`, `ensure_indexed_for`), `src/cbm/proxy.rs` (`resolve_proxy_target_project`, `resolve_and_apply_proxy_target_project`), `src/tests/cbm/regression.rs`, `src/tests/cbm/proxy_errors.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 200-210, original section `CBM-ID-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `a1a1fc48550583c9e8390b9ae542919abbbd1addb08bbfe09f8f6b379a30fde9`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_ID_001:end -->
<!-- pcg:INV.CBM_LIFE_001:begin -->
### CBM-LIFE-001 Optional Provider Never Gates Core MCP Readiness

| Property | Value |
|----------|-------|
| **Intent** | Clean-CTX must establish its stdio MCP session independently of optional CBM startup, while retaining deterministic ownership of every provider process and startup task it creates. |
| **Invariant** | `McpState` construction performs no CBM launch or admission. A successfully flushed Clean `initialize` response permits `auto_launch`; it does not prove that the client will remain connected. One session-owned lifecycle authority coalesces concurrent launch requests and owns admission state, the admitted bridge, background indexing workers, and shutdown. With `auto_launch = false`, only the first provider-dependent consumer starts that same lifecycle. `get_cbm_status` is report-only. Provider consumers fail closed while status is `initializing`, `degraded`, or `unavailable`; Clean-owned semantic authority never depends on provider readiness. On transport shutdown, Clean joins any in-flight admission and all indexing workers before retiring the directly owned frontend. No provider startup work is fire-and-forget. |
| **Enforcement** | `src/tests/cbm/startup_readiness.rs` pins core readiness independence; `src/tests/mcp/cbm_lifecycle.rs` pins the initialize trigger, lazy mode, coalescing, observable initialization, and admission joining; `src/tests/cbm/frontend_lifecycle.rs` pins indexing-worker ownership and frontend retirement ordering. |
| **Authority** | `src/mcp/cbm_lifecycle.rs`, `src/mcp/state.rs`, `src/mcp/handlers.rs`, `src/mcp/server.rs`, `src/cbm/bridge/indexing.rs`, `src/cbm/bridge/lifecycle.rs` |
| **Type** | STRUCTURAL + ENFORCED (test) |
| **Gate** | Focused lifecycle regressions and the complete all-feature verification gate |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 211-221, original section `CBM-LIFE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `0fa68dcdc2f6b439a2847fbdefc7854bed58896778587dd521b54f3219c41383`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_LIFE_001:end -->
<!-- pcg:INV.CBM_CACHE_001:begin -->
### CBM-CACHE-001 Canonical Project-Owned Graph Cache

| Property | Value |
|----------|-------|
| **Intent** | Cached graph results must never cross repository boundaries, and refreshing one repository must not discard or invalidate another repository's results. |
| **Invariant** | Every structured graph-cache entry is structurally owned by exactly one canonical CBM project through `GraphCacheKey { project, query }`. Active-project and explicit-project queries use the same query text namespace; ownership is never inferred from active bridge state or encoded by manually prefixing the query string. Memory lookup, insertion, expiration, and removal require that owner. Disk lookup/write-through resolves the owner's registered root only through `project_paths`; an unregistered project may be cached in memory but is never assigned an invented disk root. A successful graph refresh invalidates all and only the target project's memory entries and registered SQLite partition. Failed refreshes leave the project dirty and do not invalidate as though fresh. Switching the active project or workspace root preserves safely isolated entries. TTL, SQLite schema, repository-scoped CBM `index_repository(repo_path, mode)` behavior, and public MCP contracts remain unchanged. |
| **Enforcement** | The Rust key type makes cross-project memory reuse structural. `src/tests/cbm/cache_ownership.rs` pins switch-time retention, explicit-project disk hydration, and target-only disk invalidation through unchanged RED→GREEN regressions. Existing freshness regressions pin failure and generation coalescing. `e2e_apply_edit_triggers_reindex_and_graph_is_fresh` crosses registered `apply_edit` and `graph_search` handlers against live CBM, verifies the fresh structured response and repeated cache hit, and preserves an unrelated-project sentinel. |
| **Authority** | `src/cbm/bridge.rs` (`GraphCacheKey`), `src/cbm/bridge/cache.rs`, `src/cbm/bridge/indexing.rs`, `src/cbm/bridge/lifecycle.rs`, `src/cbm/bridge/query.rs`, `src/tests/cbm/cache_ownership.rs`, `src/tests/cbm/regression/freshness.rs`, `src/tests/cbm/e2e_reindex.rs` |
| **Type** | STRUCTURAL + ENFORCED (test) + LIVE INTEGRATION |
| **Gate** | Focused cache-ownership and live lifecycle regressions locally; complete `cargo test --workspace --all-targets --all-features` in the final gate |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 222-232, original section `CBM-CACHE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `68c0bbdbb80c553562e6ef6d77dd5f57e716efc693bab6288486e00ed19491c6`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_CACHE_001:end -->
<!-- pcg:INV.CBM_E_001:begin -->
### CBM-E-001 Explicit CBM Error Propagation

| Property | Value |
|----------|-------|
| **Intent** | CBM unavailability or failure must never masquerade as legitimate empty graph data. |
| **Invariant** | Every graph-intelligence bridge method returns `Result<_, CbmError>`. `Ok(empty)` is reserved for valid zero-result queries; any CBM-reported tool failure (`result.isError` envelope), transport fault, timeout, or open circuit surfaces as `Err(CbmError)`. Downstream consumers (intelligence layer, inference pass, MCP handlers) propagate or explicitly handle `Err`; none may convert it into empty success data. The pipeline-level failure policy is fixed: log loudly and continue without enrichment - CBM is strictly additive to the IR. |
| **Enforcement** | `check_soft_error()` maps isError envelopes to `CbmError::ToolError` in the parsed transport path before callers observe them; `proxy_tool_error()` applies the same boundary to raw `cbm_proxy` responses before verification or compression and emits one clean MCP `isError` result; deterministic fixtures pin both envelope paths; live probes assert failure on unknown projects vs valid empty results. |
| **Authority** | `src/cbm/client.rs` (`CbmError::ToolError`, `check_soft_error`), `src/cbm/bridge.rs` (Result signatures), `src/ir/inference_layer.rs`, `src/ir/pipeline.rs`, `src/tests/cbm/graph_intel.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 233-243, original section `CBM-E-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `be36aca3ae7f249aea82db1d3669f83ecf6854e72118742eea7383817fba2e2d`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_E_001:end -->
<!-- pcg:INV.CBM_IDENTITY_001:begin -->
### CBM-IDENTITY-001 Trace Source Resolution Never Guesses

| Field | Contract |
|---|---|
| **Intent** | A trace must never silently select one same-named symbol and present that path as authoritative. |
| **Invariant** | Canonical CBM source identities are a no-search fast path. A bare trace source is resolved by exact symbol name within the explicitly selected project before tracing. Distinct physical search occurrences with one canonical ID are one identity. Exactly one canonical identity proceeds; zero returns explicit not-found; multiple return JSON-RPC `-32602` with deterministic canonical candidates. `graph_trace` and `cbm_proxy(trace_path)` share this resolution boundary. Proxy resolution is scoped to that call and must not mutate the bridge's active project. |
| **Enforcement** | `src/cbm/trace_identity.rs` owns exact-name grouping and errors; `GraphBridge::search_scoped` owns non-mutating project lookup; `src/tests/cbm/trace_identity_resolution.rs` protects wrapper/proxy parity and the canonical fast path; `verification/cbm/scripts/Investigate-DuplicateTraceNames.ps1` verifies the built stdio boundary against a controlled duplicate-name repository. |
| **Authority** | `src/cbm/trace_identity.rs`, `src/cbm/handlers.rs`, `src/cbm/proxy.rs`, `src/cbm/bridge/query.rs` |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 244-252, original section `CBM-IDENTITY-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `dc93df59ca10e63df78729b799a86693eeebcdbd6b943419e5b1ba9b3111c6d1`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_IDENTITY_001:end -->
<!-- pcg:INV.CBM_WIRE_001:begin -->
### CBM-WIRE-001 Verified CBM `trace_path` Wire Contract

| Property | Value |
|----------|-------|
| **Intent** | The typed `graph_trace` path must consume what CBM actually emits on the wire — never a presumed shape — so real call-relationship data reaches agents instead of silently collapsing to zero results while the raw proxy path works. |
| **Invariant** | **`inner["edges"]` is NOT a valid CBM 0.8.1 `trace_path` response shape and must never be assumed** by any parser, wrapper, or fixture. The verified contract (verbatim live captures from a fresh subprocess, 2026-08-24): relationships arrive as directional `callers` / `callees` ARRAYS whose entries carry exactly `name`, `qualified_name`, and `hop` (a JSON number); an empty half is a real empty array, never a missing key and never an `edges` key. Specifically: (1) **Directionality** — every `callers[i]` calls the traced function, and the traced function calls every `callees[i]`; normalized edges always orient caller → callee regardless of which array produced them. (2) **Canonical identity** — edge endpoints are `qualified_name` with fallback to the bare `name` (`map_search_result` precedent); `hop` is dropped: a `GraphEdge` represents a relationship, not traversal metadata. Note the raw wire key is `name` — `nm` exists only in Clean-CTX's own compressed proxy view. (3) **Boundary matching** — the API accepts bare names while the wire carries qualified names, so a target matches a canonical endpoint when the endpoint EQUALS the target (fully qualified form) OR its FINAL DOT SEGMENT equals the bare target; a bare `to` name with qualified wire endpoints MUST retain the edge; partial/multi-segment targets match nothing. `__file__` module pseudo-node callers are genuine relationships and pass through untouched. (4) **Both directions work** — outbound-reachable pairs resolve on the FIRST attempt (pre-fix behavior preserved byte-for-byte); inbound-only relationships are discovered through a SINGLE inbound fallback taken only when the outbound attempt succeeds but yields no edge touching the target; errors are never swapped for the other direction. (5) **Depth honesty** — depth>1 responses are FLAT hop-tagged BFS discoveries with NO parent linkage; only `hop: 1` entries convert into edges; `hop > 1` entries are unattributable and must NEVER become invented edges (they remain available via `cbm_proxy`). (6) **Ordering & dedup** — CBM emission order is preserved; ONLY exact duplicate edges may collapse, preserving first-seen order; repeated nodes are not relationships and are never merged away. (7) **Result semantics** — a valid query yielding no relationships remains `Ok(empty)`; any CBM soft error (`result.isError` envelope, e.g. `"error": "function not found"`) propagates as an explicit `Err(CbmError::ToolError)` BEFORE parsing — failure is never a valid empty result (normative generalization: CBM-E-001). |
| **Enforcement** | Deterministic pins against verbatim raw captures (`TRACE_INBOUND_WIRE_CAPTURE`, `TRACE_OUTBOUND_WIRE_CAPTURE`, `TRACE_BOTH_WIRE_CAPTURE`, `TRACE_DEEP_OUTBOUND_WIRE_CAPTURE`, `TRACE_NOT_FOUND_RESULT_ENVELOPE`): directional synthesis, hop-1-only conversion (the depth-3 capture proves flat semantics and cross-hop node repeats), qualified→name fallback, exact-duplicate dedupe ordering, absent-array leniency, not-found → `ToolError` gate, boundary predicate strictness (exact/final-segment retained; partial/multi-segment rejected; `regression_bare_to_name_with_qualified_endpoint_is_retained`). Four fresh-process `serial(cbm_live)` probes over a SYNTHETIC temp-dir fixture repo (caller → callee is the only relationship; nothing derives from this repository): typed `graph_query` CALLS rows, two-endpoint outbound preservation, single-endpoint inbound discovery, and THE regression — two-endpoint inbound-only discovery via the fallback. Finalization evidence: `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` zero warnings; `cargo test --workspace --all-targets --all-features` 2,497 passed / 0 failed / 5 ignored including `e2e_cbm_multiroot_multilingual_integration` green against live CBM 0.8.1 (commit `193f885`). |
| **Authority** | `src/cbm/client.rs` (`extract_trace_edges`, `trace_entry_endpoint`, `trace_entry_is_direct`), `src/cbm/bridge.rs` (`GraphBridge::trace_path` direction determination, `edge_touches_target`, `filter_trace_edges`), `src/tests/cbm/trace_wire.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 253-263, original section `CBM-WIRE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `88b8ee6ecba90ae0067938e81a7bae44f242187de51d71358b986b021e60b5c4`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_WIRE_001:end -->
<!-- pcg:INV.CBM_WIRE_002:begin -->
### CBM-WIRE-002 Verified CBM `query_graph` Wire Contract & Column-Shape-Driven Edge Extraction

| Property | Value |
|----------|-------|
| **Intent** | `query_graph` interprets the semantic projection described by CBM's echoed `columns`; it must NOT infer relationship semantics from row arity. The typed `graph_query` path surfaces relationship data CBM actually returns instead of collapsing it into duplicated column-0 nodes with a permanently empty edge list, and never fabricates edges from arbitrary column data. |
| **Invariant** | CBM answers `query_graph` with `{columns, rows, total}`; cells are JSON strings (numeric projections arrive stringly, e.g. in_degree `"10"`); undirected `-[r]-` patterns are supported and one result set may mix every relationship type (DEFINES / DECORATES / USAGE / CALLS — captured live 2026-08-24). Columns echo RETURN expressions VERBATIM, including inner whitespace (`"type( r )"`). ALIAS RULE (captured live): an `AS` alias REPLACES the whole expression in the echo (`type(r) AS rel_kind` ⇒ `"rel_kind"`; `a.name AS caller` ⇒ `"caller"`) — an aliased type() projection is INTENTIONALLY indistinguishable from an ordinary projected scalar at the typed layer and must never be reverse-engineered into relationship semantics. Conversion is COLUMN-SHAPE driven: a projection is relationship-shaped IFF it contains exactly ONE unaliased `type(...)` column AND ≥3 columns AND every row aligns with the echoed columns; then endpoints are the FIRST and LAST non-type projected columns (projection order rules), the type cell becomes `GraphEdge.label`, and every other projected column maps into `GraphEdge.properties` keyed by echoed column text with its projected JSON value preserved verbatim; no nodes are synthesized. ANY other shape — including zero or multiple type(...) columns (ambiguity: refuse to guess) and row/column misalignment (untrustworthy metadata) — keeps the legacy column-0 node mapping with NO edges, REGARDLESS of column count. Rationale: the retired strict-arity rule fabricated semantically invented data — a uniform numeric triple `[name, in_degree, out_degree]` became a fake edge labelled `"10"`. Deliberately excluded from this contract (separate findings): node deduplication, file-path population, endpoint normalization. Qualified/bare M-01 matching does NOT apply here because graph_query performs no target filtering. Cache compatibility: populated edges reuse the existing serialized `edges` key — no key versioning. |
| **Enforcement** | Verbatim raw captures for EIGHT shapes (fresh subprocesses, 2026-08-24): directed CALLS bare names, undirected mixed types, qualified endpoints, aliased-type (`rel_kind`), 5-column mid-projection type(), 6-column SCRAMBLED type()-at-index-2 with trailing file_path, type()-first, numeric triple `[f.name, f.in_degree, f.out_degree]`, and the whitespace variant `"type( r )"`. Deterministic pins: shape-driven conversion + property mapping (`five_column_projection_maps_extras_into_properties`, `scrambled_six_column_projection_follows_column_metadata`, `type_first_projection_still_identifies_endpoints`, `whitespace_tolerant_type_detection`), THE fabrication regression (`numeric_triple_without_type_column_is_never_an_edge`), alias pin (`aliased_type_projection_is_intentionally_node_shaped`), ambiguity/misalignment/empty/duplicate policy pins. Four fresh-process `serial(cbm_live)` probes over the SYNTHETIC temp-dir fixture repo: 3-column CALLS baseline edge, wide scrambled 4-column projection mapping middle columns into properties, aliased+numeric-triple fabrication guards live, node-only control. Full finalization gate green (fmt clean; clippy `-D warnings` zero; workspace tests incl. live CBM probes + multilingual fixture). |
| **Authority** | `src/cbm/client.rs` (`QueryRows` — columns MUST NOT be discarded, `query_graph`), `src/cbm/bridge.rs` (`single_type_column`, `convert_query_rows`, `GraphBridge::query_graph`), `src/tests/cbm/query_wire.rs`; generated live-session captures are runtime evidence under `target/**`, not authority |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 264-276, original section `CBM-WIRE-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `195a5f4a98912a3f9130a04fcf51ef6c98e74fbfb4173d923dd8a02df9a4f90f`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_WIRE_002:end -->
<!-- pcg:INV.CBM_VERIFY_001:begin -->
### CBM-VERIFY-001 Per-Result Caller Verification for `search_graph`

| Property | Value |
|----------|-------|
| **Intent** | CBM's search-side `in_degree` is not a verified caller count (it counts additional edge kinds versus the stored property, and overload conflations inflate it), so caller verification belongs to each returned symbol — never to the search response as a whole. A raw count must never be indistinguishable from a verified one, and search breadth must never decide whether verification runs. |
| **Invariant** | `search_graph` builds ONE verification request per result carrying `in_degree` (a single result is a batch of one) and reuses the shared C# arity verifier unchanged; the arity algorithm, candidate narrowing, and trusted-path reads are not duplicated. Every `in_degree` result carries its own truth: `raw_in_degree` (the verbatim CBM value, never replaced), `verified_in_degree` (the verifier's compatible-caller count for THAT symbol, `0` when nothing was established), `in_degree_evidence` (`cbm_raw` when at least one compatible caller was established, otherwise `cbm_raw_unverified`), `in_degree_resolution` (the per-result verdict), and `in_degree_reason` for every non-verified verdict. Eligibility is callable-only: results whose label is a known non-callable kind, whose own file is not C# source, which carry no usable `qualified_name`, or whose identity occurs more than once in the response are annotated as unverifiable/ambiguous and never verified — an unattributable identity is never guessed. Verification never runs for results without `in_degree` (those stay untouched). One unverifiable or ambiguous result never suppresses verification of its neighbours, and evidence is attached to the planned response position confirmed against that entry's stable `qualified_name` (a unique identity match is the defensive fallback when the array was reshaped; a duplicated identity is reported ambiguous and never guessed), so the wrong overload can never be annotated. The top-level block keeps its single-target field set — identity and arity survive only while every folded target agrees — adds `results_total` plus the disposition counters `results_verified` / `results_ambiguous` / `results_rejected_or_adjusted` / `results_unverifiable` / `results_not_eligible` (which partition `results_total`), and reports the weakest-link resolution across independently verified targets. `trace_path` and `query_graph` remain single-target surfaces. |
| **Enforcement** | Deterministic pins with the shared verifier injected as a canned closure (no CBM required): `src/tests/cbm/caller_verify_search.rs` — `RED-SG1` single-result control (existing fields and top-level verdict preserved), `RED-SG2` one eligible among non-callable kinds, `RED-SG3` several callable results verified independently with no count bleed, `RED-SG4` verified + ambiguous coexistence, `RED-SG5` verified + unverifiable coexistence (including a result with no identity), `RED-SG6` the unanchored `OrderBy` reproduction, `RED-SG7` anchored/unanchored parity for the same symbol, `RED-SG8` no bare raw `in_degree` in a mixed response plus the counter-partition invariant; `aggregate_folds_targets_without_inventing_identity` and `search_targets_are_empty_without_a_results_array_or_degree` pin aggregation and the untouched-response path. `src/tests/cbm/caller_verify_proxy.rs` pins the single-target request construction, the single-target count writer, and the stable gap-reason codes. |
| **Authority** | `src/cbm/caller_verify_search.rs` (per-result planning, orchestration, annotation, aggregation), `src/cbm/caller_verify_proxy.rs` (`VerificationRequest`, `VerificationAttempt`, `VerificationGap`, `verify_request_sources`, `verify_search_surface`), `src/cbm/caller_verify.rs` (`verify_csharp_callers` — unchanged algorithm, `CallerVerificationSummary::{empty, fold}`, `aggregate_resolution`, `caller_evidence_metadata`), plus `src/cbm/caller_verify_arity.rs` (arity extraction and the request-local `ParseMemo`), `src/tests/cbm/caller_verify_search.rs`, `src/tests/cbm/caller_verify_proxy.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 277-289, original section `CBM-VERIFY-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `5199b1dc818cc1eecbdbda42caa63cbc2d6538109473e840e715514476c79a93`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_VERIFY_001:end -->
<!-- pcg:INV.CBM_VERIFY_002:begin -->
### CBM-VERIFY-002 Verified Caller Identity Is Retained and Candidate Discovery Reuses the Project-Scoped Graph Cache

| Property | Value |
|----------|-------|
| **Intent** | Verifying a CBM candidate and then discarding which candidate verified wastes the work: the consumer must ask CBM again to learn the caller/file identity verification already knew, and that second round-trip is exactly the cost verification was meant to remove. Candidate discovery is also repeated work: an identical discovery query in the same valid cache scope must not pay a fresh provider round-trip. |
| **Invariant** | (1) **Identity retention.** `CallerVerificationSummary` keeps every established counter (`raw_candidates`, `verified`, `rejected_arity`, `ambiguous`, `unverifiable`, `verified_caller_files`, `compatible_caller_files`, `resolution`) and adds structured evidence beside them: `verified_candidates` (candidates whose source verification established compatibility) and `ambiguous_candidates` (candidates compatible with a target that stayed ambiguous). Each retained candidate carries CBM's projected caller file (`cbm_file` — the advisory candidate identity, never promoted to a semantic fact), the path Clean-CTX actually read after the trusted-path gate (`file`), the distinct accepted explicit argument counts (`argument_counts`, ascending), and its own `status`. Rejected candidates are counted, never enumerated. A candidate CBM proposed that source verification contradicted is never reported as verified, and ambiguity is never presented as a unique resolution. (2) **Authority unchanged.** The truth claim is always "CBM proposed candidate X, Clean-CTX verified X against source": CBM remains advisory discovery/narrowing only, no CBM edge is imported into `WorkspaceIndex`, and cached discovery never makes verification truth stale — source verification still runs and still rejects. (3) **Candidate discovery reuses the project-owned graph cache.** `GraphBridge::query_graph_scoped(cypher, project)` shares the same cache, TTL, disk write-through, and invalidation as `query_graph`. Both use the identical `cypher2:{cypher}` query namespace; `GraphCacheKey.project` supplies the canonical structural owner, so two repositories with identical Cypher text remain isolated without string-prefix conventions or active-project cache clearing. A request that names **no** project keeps the original raw proxy call: it is answered by CBM's own default project, which the active project does not describe, so no structured cache owner can be assigned. (4) **Batch-local parse reuse.** One verification batch (one response) owns exactly one `ParseMemo`, so a file referenced by several results is parsed once; the memo is request-local, never persisted, and never a source-freshness substitute (`McpState::read_source` still decides which bytes are current). Original arity semantics (extension-receiver subtraction, optional parameters, `params`, exact fixed arity, same-name/same-arity ambiguity) are unchanged. |
| **Enforcement** | Deterministic pins (no CBM required; the mock bridge owns no client, so a provider round-trip is observable as a recorded error): `src/tests/cbm/caller_verify.rs` — `RED-CV1` verified identity retained beside the count, `RED-CV2` rejected identity absent from `verified_candidates`, `RED-CV3` anchored and unanchored ambiguity preserved with evidence, `RED-CV4` extension-receiver evidence (receiver excluded, argc retained, identity surfaced), `RED-CV6` every established counter preserved, `RED-CV10` one batch parses a referenced file once, `RED-CV11` distinct files verified independently (the memo never collapses two paths). `src/tests/cbm/caller_verify_proxy.rs` — `RED-CV5` end-to-end (cached discovery for a project that is not the active project + real source verification + rendered response exposing the identity with no provider call), `RED-CV7` cached candidate discovery with a miss as the only provider contact, `RED-CV8` typed project ownership isolating two workspaces with identical query strings, `RED-CV9` cache ownership per provider instance, `RED-CV10` batch-level parse reuse through the real search orchestration, `RED-CV12` source verification refuting a candidate that discovery served from cache. |
| **Authority** | `src/cbm/caller_verify.rs` (`VerifiedCallerCandidate`, `CallerVerificationSummary`, `verify_csharp_callers`, `candidate_evidence`), `src/cbm/caller_verify_arity.rs` (arity extraction, `ParseMemo`), `src/cbm/caller_verify_proxy.rs` (`candidate_paths`, `caller_paths_from_query`, `LoadedCandidate`, `verify_request_sources`, `read_trusted_source`), `src/cbm/caller_verify_search.rs` (batch memo ownership, per-result evidence), `src/cbm/bridge/query.rs` (`query_graph_scoped`, `query_graph_inner`), `src/tests/cbm/caller_verify.rs`, `src/tests/cbm/caller_verify_proxy.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 290-303, original section `CBM-VERIFY-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `549c7a95248e3c6489b9e7e578d97e3b8551316f91d7c3348988bc4e6b5fc04f`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:INV.CBM_VERIFY_002:end -->
<!-- pcg:INV.CBM_OPTIONAL_001:begin -->
### CBM-OPTIONAL-001 Optional Providers Change Cost, Not Core Truth

**Status:** ENFORCED

Clean-CTX's core semantic graph must be independently sufficient to establish
every core query contract. CBM may discover or narrow candidates, but it neither
authors Clean-CTX semantics nor certifies WorkspaceIndex completeness. External
providers may accelerate or enrich graph construction, but their readiness must
never gate core correctness, availability, or completion. Optional acceleration
must not become optional correctness.

Provider evidence may be usable before provider coverage is authoritative. A
successful provider operation is not proof of complete provider coverage:
`StillIndexing`, Ready, successful search, and an empty result remain advisory
unless the provider supplies a validated proof for exact compatible coverage
units. Current CBM supplies no proof, so it cannot suppress Clean-owned incoming
TypeRef acquisition. Clean inventory, certified producer-specific candidate
selection, required compilation/publication, reconciliation, and final
proof-set evaluation establish Clean authority. A future optional provider may
substitute only the exact units it proves; Clean acquires the residual. CBM
readiness is neither necessary nor sufficient for core completion.

Warm state may improve performance. It must never improve truth. Clean-CTX owns
its frontend/session transport, not the shared account daemon.

**Enforcement:** `src/mcp/acquisition/**` owns provider-neutral requirements,
proof validation, satisfaction, and cache identity;
`src/mcp/tool_handlers/hydration/**` unions provider hints with Clean residual
acquisition and commits satisfaction only after publication. The acquisition
test suites and `workspace_query_reverse_edge_completeness.rs` cover no-proof,
empty, mixed-unit, failure, invalidation, cold/warm, and exact-result behavior;
existing search/readiness controls remain in `workspace_query_8.rs`,
`workspace_query_9.rs`, and `workspace_query_provider_readiness.rs`.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 874-907, original section `CBM-OPTIONAL-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `14c5550092bdf1649bc0d6822ae3188bee9c418ec93017cd32fa17555bd0b09d`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: INV.CBM_ID_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-ID-001 Canonical CBM Project Identity & Multi-Root Lifecycle
track: REPOSITORY
artifact: docs/invariants/cbm.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_ID_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_LIFE_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-LIFE-001 Optional Provider Never Gates Core MCP Readiness
track: REPOSITORY
artifact: docs/invariants/cbm.md
preserves:
  - INV.CBM_OPTIONAL_001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_LIFE_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_CACHE_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-CACHE-001 Canonical Project-Owned Graph Cache
track: REPOSITORY
artifact: docs/invariants/cbm.md
preserves:
  - INV.CBM_ID_001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_CACHE_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_E_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-E-001 Explicit CBM Error Propagation
track: REPOSITORY
artifact: docs/invariants/cbm.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_E_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_IDENTITY_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-IDENTITY-001 Trace Source Resolution Never Guesses
track: REPOSITORY
artifact: docs/invariants/cbm.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_IDENTITY_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_WIRE_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-WIRE-001 Verified CBM `trace_path` Wire Contract
track: REPOSITORY
artifact: docs/invariants/cbm.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_WIRE_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_WIRE_002
kind: INVARIANT
status: IMPLEMENTED
title: CBM-WIRE-002 Verified CBM `query_graph` Wire Contract & Column-Shape-Driven Edge Extraction
track: REPOSITORY
artifact: docs/invariants/cbm.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_WIRE_002
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_VERIFY_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-VERIFY-001 Per-Result Caller Verification for `search_graph`
track: REPOSITORY
artifact: docs/invariants/cbm.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_VERIFY_001
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_VERIFY_002
kind: INVARIANT
status: IMPLEMENTED
title: CBM-VERIFY-002 Verified Caller Identity Is Retained and Candidate Discovery Reuses the Project-Scoped Graph Cache
track: REPOSITORY
artifact: docs/invariants/cbm.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_VERIFY_002
  - qualification self#family-standing
```

```architecture-graph
node: INV.CBM_OPTIONAL_001
kind: INVARIANT
status: IMPLEMENTED
title: CBM-OPTIONAL-001 Optional Providers Change Cost, Not Core Truth
track: REPOSITORY
artifact: docs/invariants/cbm.md
preserves:
  - WSC-014
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#INV.CBM_OPTIONAL_001
  - qualification self#family-standing
```
<!-- pcg:INV.CBM_OPTIONAL_001:end -->
