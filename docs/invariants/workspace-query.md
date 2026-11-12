<!-- pcg:family-standing:begin -->
# Invariant family: workspace-query

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

<!-- pcg:family-standing:end -->
<!-- pcg:WSC-002:begin -->
### WSC-002 CBM Discovers Candidates; Clean-CTX Alone Determines WorkspaceIndex Semantics

| Property | Value |
|----------|-------|
| **Intent** | CBM or the workspace filesystem may discover candidate files so Clean-CTX's own semantic graph becomes better populated, but advisory discovery semantics must never become WorkspaceIndex facts. The candidate-file boundary is deliberately narrow. |
| **Invariant** | Healthy CBM discovery may span every configured CBM project associated with the workspace (primary root plus `additional_roots`) and may influence compilation scope only. When CBM cannot cover a configured root, deterministic filesystem discovery scans that trusted root for literal query-name occurrences in source extensions accepted by Clean-CTX's language registry. Filesystem discovery prunes its exact built-in metadata/dependency/build/cache directory set before enumeration and also honors additive configured exclusions. Either provider supplies candidate file paths only; graph relationships and textual matches are advisory and never become WorkspaceIndex facts. Candidate discovery is query-semantic aware: CBM declaration-oriented queries (`find_entities`, `forward_edges`, `transitive_dependencies`) discover target declarations, `reverse_edges` discovers possible inbound referrers, and filesystem fallback naturally considers every non-excluded source file containing the literal identity. There is no path from either provider to a WorkspaceIndex relationship without intervening `resolve_file_path_checked` → `compile_file_ir_focused` → Clean-CTX semantic extraction → `semantic_edges` → `WorkspaceIndex.add_edges`. A candidate that compiles but yields zero matching semantic edges contributes zero fabricated relationships. Provider and completion metadata are deterministic and observable, so zero candidates cannot silently conceal unavailable discovery. That metadata reaches an LLM as a SPARSE projection: expected state is omitted entirely and only exceptional, decision-relevant deviation is serialized (one optional `discovery` object with a compact completeness status, the engaged provider, the fallback reason, non-zero candidate counts and exceptional-only project entries), while the internal `HydrationReport` keeps the complete provider/completion/candidate/coverage record for debugging, telemetry and the regressions — LLM context cost is reduced without reducing observability. |
| **Enforcement** | `src/mcp/tool_handlers/hydration.rs` (`discover_candidate_paths` — selects declaration or inbound-reference discovery and searches each configured project explicitly despite advisory local readiness; `select_candidates` — merged-pool normalization/dedup/exclude-indexed/deterministic sort with no semantic candidate ceiling; `compile_candidate` — trusted-root validation + existing compiler path; WorkspaceIndex updated only from returned `semantic_edges`); `src/mcp/tool_handlers/hydration/filesystem.rs` (exact safe-default directory pruning plus additive configured exclusions, no symlink following, authoritative language registry); `src/cbm/project_search.rs` (project-explicit declaration search and path-only inbound graph query do not mutate active-project state); `src/tests/mcp/workspace_query_2.rs::red11_cbm_authority_cardinality_isolation` and RED-12/13; `src/tests/mcp/workspace_query_3.rs` RED-15 through RED-22; `src/tests/mcp/workspace_query_4.rs` RED-23 through RED-28; `src/tests/mcp/workspace_query_5.rs` RED-C1 through RED-C6; `src/tests/mcp/workspace_query_7.rs` RED-S1 through RED-S9; `src/tests/mcp/workspace_query_diagnostics.rs` RED-DIAG1 through RED-DIAG8 and RED-DIAG14/15 (the LLM-facing projection: omission at expected state, compact status vocabulary, verbatim reason codes, non-zero-only counts, exceptional-only minimized project entries, structured truncation, and the pinned field set of a degraded answer) and `src/tests/mcp/workspace_query_diagnostics_response.rs` RED-DIAG9 through RED-DIAG13/15 (the same contract over the real dispatch path, including that the semantic payload stays byte-identical to the index answer). |
| **Authority** | `src/mcp/tool_handlers/hydration.rs`, `src/mcp/tool_handlers/hydration/filesystem.rs`, `src/cbm/project_search.rs`, `src/tests/mcp/workspace_query_2.rs`, `src/tests/mcp/workspace_query_3.rs`, `src/tests/mcp/workspace_query_4.rs`, `src/tests/mcp/workspace_query_5.rs`, `src/tests/mcp/workspace_query_7.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 523-535, original section `WSC-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `82aa57950c13f861af5f0abf6f8d29a90a4067cf4a38137916c66d311cfd871e`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-002:end -->
<!-- pcg:WSC-003:begin -->
### WSC-003 Hydration Discovery Is Session-State Work

| Property | Value |
|----------|-------|
| **Intent** | Discovery is the expensive half of hydration: a CBM project search (including its lazy-reindex gate) or a full filesystem walk-and-read of a root. Candidate publication is already deduplicated by current required semantic projections in `WorkspaceIndex`; discovery must be deduplicated too, or every repeated identical query pays the full discovery cost again only to discard the same candidates as already current. |
| **Invariant** | Within one session/workspace state, when discovery for the same semantic target, discovery scope (provider + canonical root identity), discovery mode (`Declaration` for `find_entities` / `forward_edges` / `transitive_dependencies`, `InboundReference` for `reverse_edges`), and workspace generation has already completed **successfully**, that discovery MUST NOT be repeated until the scope's generation changes. **HYDRATION-CACHE:** discovery completion is committed only after every selected candidate has either demonstrated a current required semantic projection or successfully published one. A candidate publication failure leaves discovery incomplete and the candidate retryable. Only successful discovery and publication are recorded — including successful discovery that found zero candidates; failed, partial, still-indexing, CBM-unavailable, and candidate-publication-failed cycles are never recorded and remain retryable. A hydration cycle containing a candidate publication failure MUST report partial discovery status and MUST NOT establish reusable discovery completion for that cycle. Recorded state is completion only: no candidates, entities, edges, or query answers are cached, and every `workspace_query` answer is still evaluated against the live `WorkspaceIndex`. Each scope (CBM project or filesystem root) is invalidated independently, and the provider is part of the scope identity so a CBM result never suppresses the filesystem fallback for the same root. **HYDRATION-SINGLE-FLIGHT-001:** concurrent attempts may coalesce only when either execution could safely perform the other's hydration work. A flight communicates only that an attempt ended; it is never discovery, acquisition, proof, or publication authority. Every follower re-enters the ordinary authoritative checks after waking, and an incomplete or failed leader leaves remaining work retryable. Unrelated work-equivalence identities remain concurrent. |
| **Enforcement** | `src/mcp/discovery_cache.rs` (generation-based scope/entry model; unit regressions in `src/tests/mcp/discovery_cache.rs`); `src/mcp/hydration_flight.rs` (session-owned keyed execution coordination with drop-safe retirement and coherent wait/notification); `src/mcp/tool_handlers/hydration.rs` + `src/mcp/tool_handlers/hydration/publication.rs` (two-phase completion commit and distinct published/current/failed outcomes); `src/mcp/tool_handlers/hydration/cache.rs` (the single check/mark boundary) and `src/mcp/tool_handlers/hydration/invalidation.rs` + `src/mcp/tool_handlers/edit.rs` (`apply_edit`) + `src/cbm/proxy.rs` (`index_repository`) + `src/mcp/state/source_cache.rs` (source-cache mtime/size staleness invalidates the whole cache); `src/tests/mcp/hydration_single_flight.rs` (equivalent work counts, incomplete-leader recheck/retry, and unrelated-key concurrency); `src/tests/mcp/workspace_query_8.rs` RED-H1…RED-H10 (counted discovery invocations through the cfg(test) project-search injection); `src/tests/mcp/workspace_query_9.rs` RED-FS1…RED-FS5 plus negative inbound refresh coverage (thread-local `scan` invocation counter plus `TraversalStats` file counts); `src/tests/mcp/workspace_query_publication_retry.rs` (successful discovery followed by one forced publication failure, no failed projection, equivalent-query retry, successful publication, already-current control, and subsequent completion-cache hit). |
| **Authority** | `src/mcp/discovery_cache.rs`, `src/mcp/hydration_flight.rs`, `src/mcp/tool_handlers/hydration.rs`, `src/mcp/tool_handlers/hydration/cache.rs`, `src/mcp/tool_handlers/hydration/invalidation.rs`, `src/mcp/tool_handlers/hydration/filesystem.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

For exact incoming builtin TypeRef acquisition, the provider-bearing discovery
cache above is only an execution optimization. It cannot satisfy a coverage
unit or suppress authoritative Clean acquisition. The provider-neutral
`AcquisitionCompletionCache` separately keys and retains proof-backed unit
satisfaction after successful publication; its root generations share the edit,
delete, restore, refresh, and external-change invalidation lifecycle. Current
CBM searches contribute hints only because CBM exposes no compatible inventory,
generation, producer, or exhaustiveness proof.

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 536-557, original section `WSC-003`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `34f440a75292392fa60b0a22e9b337de9ba6e721fdcf89a7667455241f64018b`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-003:end -->
<!-- pcg:WSC-004:begin -->
### WSC-004 Workspace Queries Respect Declared Workspace Scope

| Property | Value |
|----------|-------|
| **Intent** | `WorkspaceIndex` deliberately retains the evidence of every repository a session has compiled — that retention is what makes entity ambiguity and cross-file edge evidence truthful — so a query must state which workspace it is asking about. Without a declared scope, a query issued FOR one repository answered with real, semantically valid facts authored by another repository that merely happened to share the session (`reverse_edges` returning unrelated `OrderBy` callers; `find_entities` returning a same-named entity of another project; `transitive_dependencies` walking through another repository's edges; `has_cycle` reporting a cycle assembled from two repositories' halves). |
| **Invariant** | When `workspace_query` explicitly declares a workspace root, occurrence-bearing results MUST be restricted to semantic evidence belonging to that root plus its configured `additional_roots`. Scoping is a query-time VIEW over globally retained facts: it MUST use source/file provenance — `StoredEdge::asserting_file` for edge occurrences, `EntityRef.file` for entity occurrences — decided by the single shared `WorkspaceScope::admits` rule; it MUST NOT alter Model C identity (`(domain, entity_type, name)`), MUST NOT delete or mutate unrelated workspace evidence, MUST NOT introduce query-specific root parsing, per-repository partitioning, new caches or secondary indexes, and MUST NOT let discovery evidence (CBM) become authoritative. One root set is built per query so the initial answer and the post-hydration rerun are scoped identically. For graph traversal the scope MUST be applied DURING traversal, never to the returned list, because reachability and cycle membership are properties of the walk (`A -> X` in the workspace plus `X -> Y` elsewhere must not report `Y`). `entities_in_file` is the exception that proves the rule: its explicit `file_path` is already validated against the trusted-path/root boundary (component-wise containment), so the accepted set is exactly primary root plus configured additional roots and the path itself is the scope — no redundant occurrence filter is added. A query that declares NO workspace root remains unscoped (the global/session view) and no fallback root may be substituted. |
| **Enforcement** | `src/workspace/scope.rs` (`WorkspaceScope::new` / `admits` — the one root-set authority, canonicalized through `canonical_identity_key`, component-wise containment so `repo` never admits `repo-old`); `src/workspace/index.rs` (`selected_occurrences` for edge occurrences, `entity_occurrence_admitted` + `find_entities_by_name_in_scope` for entity occurrences); `src/workspace/index/traversal.rs` (`has_cycle_in_scope`, `transitive_dependencies_in_scope` — adjacency-level filtering plus the start-occurrence guard); `src/mcp/tool_handlers/query.rs` (`query_scope` is the only root parser; every occurrence-bearing handler dispatches to the scoped variants); regressions `src/tests/mcp/workspace_query_scope.rs` + `workspace_query_scope_provenance.rs` (RED-SCOPE1–RED-SCOPE10: edge isolation, root identity, lifecycle, object-provenance and root-less controls), `src/tests/mcp/workspace_query_scope_entities.rs` (RED-WSC4-1–RED-WSC4-4), `src/tests/mcp/workspace_query_scope_traversal.rs` (RED-WSC4-5–RED-WSC4-12 plus the start-occurrence guard). |
| **Authority** | `src/workspace/scope.rs`, `src/workspace/index.rs`, `src/workspace/index/traversal.rs`, `src/mcp/tool_handlers/query.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 571-583, original section `WSC-004`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `f7f140495f4592b17483463c477c948fe951473e1268e7a7f34d4f996f5a7e45`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-004:end -->
<!-- pcg:WSC-005:begin -->
### WSC-005 File-Local Call Inspection Retains Canonical Owner and Occurrence Evidence

| Property | Value |
|----------|-------|
| **Intent** | Model C WorkspaceIndex identity is intentionally global and owner-agnostic for methods, so cross-file graph queries cannot distinguish same-named methods owned by different types in one file or isolate overload declarations. Precise file-local inspection must answer from the richer canonical representation without changing global identity or pretending that a post-projection filter can recover discarded evidence. |
| **Invariant** | `workspace_query(type="calls_in_file")` MUST resolve one trusted source file under `workspaceRoot` plus configured `additional_roots`, honor the shared optional `withinPath` narrowing, and compile a read-only High-fidelity canonical candidate for that request. It MUST NOT hydrate, read answers from, or publish facts into `WorkspaceIndex`; install alias/version/IR-baseline state; mutate source; acknowledge delta transport; expose canonical IDs; or claim resolved callee identity. Selection MUST use typed owner kind+name and method name. An omitted signature selector returns the complete same-name overload family in declaration order; supplied `parameters` and `return_type` selectors match exactly; a missing match returns zero; duplicate matching typed owners are ambiguous (`-32602`) and MUST NOT be guessed. Results MUST preserve overload occurrence, call order, duplicates, callee spelling, written argument-node count, and spread evidence. Existing cross-file operations and Model C identity remain unchanged. |
| **Enforcement** | `src/mcp/tool_handlers/query/calls.rs` (trusted path, shared scope, canonical candidate compilation, exact typed-owner/signature selection, occurrence projection); `src/mcp/tool_handlers/query.rs` (production dispatch without hydration); `src/mcp/tools.rs` (public schema); `src/tests/mcp/workspace_query_calls_in_file.rs` (same-file owner isolation, overload-family and exact-selector behavior, duplicate/order/spread preservation, fresh source observation without state publication, ambiguity rejection, and registered schema contract). |
| **Authority** | `src/mcp/tool_handlers/query/calls.rs`, `src/ir/hierarchical.rs`, `src/mcp/tool_helpers.rs::compile_file_ir_candidate` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features mcp::tool_handlers::query::tests_calls_in_file` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 584-596, original section `WSC-005`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `1124bb375847d31a3972c7c411880fb1d8c0a0fcad849f21bc87e1dd9ce1f88c`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-005:end -->
<!-- pcg:WSC-007:begin -->
### WSC-007 Dependency-Cycle Results Are Witnessed, Scoped Index Evidence

| Property | Value |
|----------|-------|
| **Intent** | `has_cycle` must return an actionable architectural dependency cycle without implying that the retained index is a complete or fresh inventory of every source file in the workspace. |
| **Invariant** | `workspace_query(type="has_cycle")` defaults to and currently accepts only `kind="dependency"`; unknown kinds return `-32602`. A dependency cycle may contain only `SemanticRelation::Injects` and `ImportsModule`; native calls, containment, routing, mapping, testing, event flow, and `Autowired` are excluded. The existing boolean and the ordered closed witness share one index-owned iterative traversal. Nodes use semantic tuple identity `(domain, entity_type, name)`; every witness edge retains its asserting file. Multiple admitted physical occurrences of a witness identity are disclosed through `identity_ambiguous` and `identity_ambiguities`, never presented as unique physical resolution. Workspace scope and `withinPath` filter edge occurrences during traversal. Stable node and query-local edge ordering make witness selection independent of insertion history; the full query includes node/edge sorting rather than claiming plain DFS complexity. Responses explicitly report `indexed_evidence_only` and `source_complete=false`. The operation performs no discovery, hydration, source compilation, persistence, rendered-context publication, or statistics mutation. |
| **Enforcement** | `src/workspace/index/traversal.rs` owns relation eligibility, deterministic witness reconstruction, scope filtering, and identity-ambiguity projection. `src/mcp/tool_handlers/query/graph.rs` owns kind validation and the additive MCP response. `src/tests/workspace/index_cycle_policy.rs` covers approved/excluded/mixed relations, self-loops, acyclic graphs, deterministic insertion-order independence, provenance, and scoped isolation. `src/tests/mcp/workspace_query_cycle_witness.rs` covers the compatible boolean, ordered witness, invalid kind, honest coverage, collision disclosure, empty response, and absence of compile/cache/statistics side effects. Existing WSC-004 suites retain root/additional-root/`withinPath` security authority. |
| **Authority** | `WorkspaceIndex::dependency_cycle_witness`, `WorkspaceIndex::dependency_cycle_witness_in_scope`, `prepare_has_cycle` |
| **Type** | ENFORCED (state ownership + test) |
| **Gate** | `cargo test --all-features cycle_policy_tests` and `cargo test --all-features tests_cycle_witness` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 610-622, original section `WSC-007`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `9b160c06f8b804bc6a4ae44d33561e383460369a83e61da0af1a6d228283ba2b`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-007:end -->
<!-- pcg:WSC-008:begin -->
### WSC-008 Incomplete Query Identity Resolves Once Without Guessing

| Property | Value |
|----------|-------|
| **Intent** | A caller that knows an entity name should not need a separate `find_entities` round trip solely to discover domain/type, while incomplete identity must never be resolved by arbitrary occurrence ordering. |
| **Invariant** | `forward_edges`, `reverse_edges`, and `transitive_dependencies` preserve the complete `(domain, entity_type, name)` fast path. When either classification field is omitted, the request ensures its existing registered hydration occurs exactly once per equivalent request-local requirement, then selects occurrences from the exact-name bucket inside the effective workspace/`withinPath` scope, applies every supplied classification field as an exact filter, and groups physical occurrences by semantic identity. Exactly one identity proceeds; zero returns explicit not-found and multiple return `-32602` with distinct deterministic candidates. Repeated files carrying the same semantic identity are not ambiguity. The response exposes `resolved_identity`; resolution never changes Model C identity, scope, edge provenance, or traversal semantics. |
| **Enforcement** | `src/mcp/tool_handlers/query/identity.rs` owns shared scoped resolution against a caller-provided final index view; `src/mcp/tool_handlers/query/{edges,graph}.rs` retain query-family hydration and semantics. `src/tests/mcp/workspace_query_identity_resolution.rs` covers forward/reverse/traversal resolution, occurrence deduplication, partial filtering, deterministic ambiguity, and explicit not-found. `src/tests/mcp/workspace_query_batch_preparation.rs` ensures batched index-backed operations share the post-preparation view. Existing WSC-004 suites protect scope/provenance and existing exact-identity suites protect compatibility. |
| **Authority** | `IdentityRequest::resolve`, `WorkspaceIndex::find_entities_by_name[_in_scope]`, exact identity query methods |
| **Type** | ENFORCED (shared boundary + test) |
| **Gate** | `cargo test --all-features tests_identity_resolution` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 623-635, original section `WSC-008`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `88d2f52c1ce09dad1c3f1bc43f30066c6603682ff9093d225830d83b7c0b326e`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-008:end -->
<!-- pcg:WSC-009:begin -->
### WSC-009 Workspace Query Batches Isolate Outcomes and Share Final Evidence

| Property | Value |
|----------|-------|
| **Intent** | A caller may combine independent workspace questions into one request without allowing one invalid item to erase valid siblings or allowing preparation order to give index-backed items different evidence views. |
| **Invariant** | `workspace_query` accepts either one legacy operation or one heterogeneous `queries` batch, never both. A batch contains 1..=32 uniquely identified items and owns one top-level `workspaceRoot`/`withinPath` scope; item scope overrides are rejected before work. Structurally valid items prepare sequentially. Equivalent hydration requirements (same discovery mode and exact name under that shared root configuration) execute once per request, including failed jobs, without adding persistent cache state. After all preparation, every index-backed item—including incomplete identity selection—evaluates in input order under one final WorkspaceIndex read view. File-local `calls_in_file` remains a fresh unpublished canonical candidate. Every accepted item produces exactly one ordered `ok` or `error` outcome; item-local validation, discovery, compilation, not-found, or ambiguity failures do not suppress independent siblings. The legacy single request and `clean-ctx/workspace-query-answer` v1 envelope remain unchanged; batch content uses `clean-ctx/workspace-query-batch-answer` v1. |
| **Enforcement** | `src/mcp/tool_handlers/query/batch.rs`, `prepare.rs`, `identity.rs`, and `content.rs`; public schemas in `src/mcp/tool_schemas.rs` and `src/mcp/tools.rs`; `src/tests/mcp/workspace_query_batch.rs` and `workspace_query_batch_preparation.rs` cover request exclusivity, ordered heterogeneous results, failure isolation, schema/envelope shape, hydration deduplication, one final view, and file-local non-publication. Existing single-query suites protect compatibility. |
| **Authority** | `batch::try_execute`, `PreparationContext`, `PreparedQuery`, and the registered `workspace_query` schema |
| **Type** | STRUCTURAL + ENFORCED (request boundary + test) |
| **Gate** | `cargo test --all-features mcp::tool_handlers::query` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 636-648, original section `WSC-009`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `e7d3a4c45a906556542ee7ed3785399f9007e37a41a43abc501bd274125c5f0b`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-009:end -->
<!-- pcg:WSC-013:begin -->
### WSC-013 Authoritative Query Truth Is History-Independent

**Status:** ENFORCED FOR CERTIFIED DOMAINS; LOWER-BOUND FAMILIES EXPLICIT

Warm state may reduce acquisition cost but may not improve semantic truth.
Authoritative batch items union acquisition requirements before final
interpretation and use one compatible post-acquisition generation. Generic
indexed graph families remain explicitly lower-bound where closure is unproved.
Exact reverse builtin TypeRef queries may become authoritative only through the
proof-backed acquisition lifecycle for every active producer unit.

**Enforcement:** query preparation, batch coordination, proof-backed acquisition
cache, and coverage projection; `workspace_query_reverse_edge_completeness.rs`
pins exact/partial and cold/warm equivalence.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 776-790, original section `WSC-013`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `d79499d20b749ecdb03d29e70d93cb2bfd33bf929788eefcf3474db80274f33c`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-013:end -->
<!-- pcg:WSC-018:begin -->
### WSC-018 Coverage Detail Cannot Strengthen Completeness

**Status:** ENFORCED

`coverage_detail` is an additive, bounded explanation of exact-identity
coverage. It may classify processing as terminal, incomplete, not attempted, or
indeterminate and proof as complete, structural, operational, mixed,
unsupported, or indeterminate. It must never authorize `source_complete=true`,
`result_semantics=exact`, or `omitted_possible=false`; only a
`CompletenessCertificate` may do that. Returned positive facts remain
`authoritative_observed` while absence authority follows only the legacy tuple.
Reasons use a closed public vocabulary, deterministic ordering, and no source,
root, provider, or owner identities. Residual counts exist only when an actual
acquisition requirement exists.

`AcquisitionOutcome` is request-local and retains typed proof dispositions plus
processing state. It is not persisted or negative-cached. The completion cache
continues to store only complete `AcquisitionSatisfaction`; warm reuse may
reconstruct terminal complete detail but cannot persist structural or transient
barriers. Provider readiness and discovery counters cannot determine proof
state or retryability.

**Enforcement:** `src/mcp/acquisition/satisfaction.rs`;
`src/mcp/tool_handlers/hydration/{acquisition,execution,publication}.rs`;
`src/mcp/tool_handlers/query/{coverage,content}.rs`; the `workspace_query`
output schema; `workspace_query_coverage.rs`,
`workspace_query_exact_identity_coverage.rs`, `workspace_query_content.rs`, and
`workspace_query_reverse_edge_completeness.rs`.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 822-850, original section `WSC-018`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `0971bb7bb4ea302098e9e35f5eaef6544ebd011bafe2f4de19b5fbf6de62b33f`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-018:end -->
<!-- pcg:WSC-010:begin -->
### WSC-010 Authoritative Refresh Retracts Absent Semantic Owners

| Property | Value |
|----------|-------|
| **Intent** | Evidence owned by a source that has been deleted, renamed, or otherwise removed from the current semantic source universe must not remain queryable as current workspace truth after an authoritative refresh. |
| **Invariant** | A successful authoritative source-universe refresh structurally retracts `WorkspaceIndex` entity, edge, and coverage ownership for sources under the refreshed root before targeted hydration republishes current evidence. Retraction is a refresh-boundary lifecycle operation, not per-query filtering, filesystem watching, historical-state promotion, or a CBM semantic decision. CBM may discover candidate locations; only Clean-CTX compilation and authority publication establish semantic truth. Other roots retain independent ownership. |
| **Enforcement** | `src/mcp/tool_handlers/hydration/invalidation.rs`; `src/workspace/index/remove.rs`; `src/tests/mcp/workspace_query_9.rs` covers stale positive evidence, deleted owners, cycle evidence, non-hydrating queries, and root isolation. |
| **Authority** | `reconcile_external_refresh_for_root`; `WorkspaceIndex::remove_files_in_root` |
| **Type** | ENFORCED (structural reconciliation boundary + tests) |
| **Gate** | `cargo test --all-features workspace_query_9` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 934-946, original section `WSC-010`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `ce29e2255798f8958a255f8771fa568d2043f0fd8d211958859d23a1c94c5d7d`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:WSC-010:end -->
<!-- pcg:WSC-011:begin -->
### WSC-011 Direct Declaration Authority Is Independent of Derived Relationships

| Property | Value |
|----------|-------|
| **Intent** | A current, authoritative declaration must remain discoverable even when a derived relationship is absent, replaced, retracted, or recomputed at another lifecycle boundary. |
| **Invariant** | Direct current declaration evidence must not depend on survival of a derived durable relationship to remain discoverable. A declaration proven directly by current structured source is materialized from that source authority; relationship edges may enrich or connect it but are not its existence certificate. Losing or replacing a derived projection must never erase the independently proven declaration. Optional provider or hydration completeness may change acquisition cost but never suppress declaration evidence already proven by the current admitted source. This rule does not permit inference from names, filenames, proximity, or unresolved ancestry: the direct declaration evidence itself must be authoritative. |
| **ASP.NET application** | A resolved `ApiControllerAttribute`, `ControllerAttribute`, `Controller`, or `ControllerBase` proof registers a durable `dotnet/Controller` role independently of route/action payload. Generation reconciliation rebuilds that role from current admitted artifacts when exact-call or other derived projections are replaced. |
| **Enforcement** | The shared C# resolver and `src/dotnet_meta/semantic.rs` establish the role; `src/mcp/semantic_compilation/artifact.rs` preserves it during generation reconciliation. `src/tests/mcp/workspace_query.rs::red_direct_controller_declaration_survives_derived_edge_replacement`, `src/tests/mcp/workspace_query_builtin.rs::builtin_framework_coexistence`, and the R9/R11 lifecycle suites pin independence and coexistence. |
| **Authority** | CSHARP-RESOLUTION-001; `src/dotnet_meta/semantic.rs`; `src/mcp/semantic_compilation/artifact.rs`; `src/workspace/index/edges.rs` |
| **Type** | ENFORCED (current-source declaration authority + tests) |
| **Gate** | Focused workspace-query regressions plus `cargo test --workspace --all-targets --all-features` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 1010-1023, original section `WSC-011`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `4cc6adf52ca2d8fc6d1eb40a2acc0ecf61b8317cce36afa654d365ce1f0c5c0c`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: WSC-002
kind: INVARIANT
status: IMPLEMENTED
title: WSC-002 CBM Discovers Candidates; Clean-CTX Alone Determines WorkspaceIndex Semantics
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-002
  - qualification self#family-standing
```

```architecture-graph
node: WSC-003
kind: INVARIANT
status: IMPLEMENTED
title: WSC-003 Hydration Discovery Is Session-State Work
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-003
  - qualification self#family-standing
```

```architecture-graph
node: WSC-004
kind: INVARIANT
status: IMPLEMENTED
title: WSC-004 Workspace Queries Respect Declared Workspace Scope
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-004
  - qualification self#family-standing
```

```architecture-graph
node: WSC-005
kind: INVARIANT
status: IMPLEMENTED
title: WSC-005 File-Local Call Inspection Retains Canonical Owner and Occurrence Evidence
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-005
  - qualification self#family-standing
```

```architecture-graph
node: WSC-007
kind: INVARIANT
status: IMPLEMENTED
title: WSC-007 Dependency-Cycle Results Are Witnessed, Scoped Index Evidence
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-007
  - qualification self#family-standing
```

```architecture-graph
node: WSC-008
kind: INVARIANT
status: IMPLEMENTED
title: WSC-008 Incomplete Query Identity Resolves Once Without Guessing
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-008
  - qualification self#family-standing
```

```architecture-graph
node: WSC-009
kind: INVARIANT
status: IMPLEMENTED
title: WSC-009 Workspace Query Batches Isolate Outcomes and Share Final Evidence
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-009
  - qualification self#family-standing
```

```architecture-graph
node: WSC-013
kind: INVARIANT
status: IMPLEMENTED
title: WSC-013 Authoritative Query Truth Is History-Independent
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-013
  - qualification self#family-standing
```

```architecture-graph
node: WSC-018
kind: INVARIANT
status: IMPLEMENTED
title: WSC-018 Coverage Detail Cannot Strengthen Completeness
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-018
  - qualification self#family-standing
```

```architecture-graph
node: WSC-010
kind: INVARIANT
status: IMPLEMENTED
title: WSC-010 Authoritative Refresh Retracts Absent Semantic Owners
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-010
  - qualification self#family-standing
```

```architecture-graph
node: WSC-011
kind: INVARIANT
status: IMPLEMENTED
title: WSC-011 Direct Declaration Authority Is Independent of Derived Relationships
track: REPOSITORY
artifact: docs/invariants/workspace-query.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#WSC-011
  - qualification self#family-standing
```
<!-- pcg:WSC-011:end -->
