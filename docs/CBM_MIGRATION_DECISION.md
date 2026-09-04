# CBM Migration Decision

| Field | Value |
|---|---|
| **Decision** | Migrate the Clean-CTX ↔ Codebase-Memory-MCP (CBM) integration from the CBM 0.8.1 integration model to the CBM 0.10.x daemon-backed architecture, targeting **CBM 0.10.8**. |
| **Status** | **Proposed** (decision document — implementation not authorized by this document) |
| **Date** | 2026-09-04 |
| **Stakeholders** | Clean-CTX architecture; CBM integration maintainers |
| **Supersedes** | Implicit 0.8.1-era assumption that Clean-CTX owns the full lifetime of an independent CBM runtime |
| **Related docs** | `docs/SEMANTIC_SUBSTRATE_ARCHITECTURE.md` (SEM-001…SEM-019), `docs/ARCHITECTURAL_INVARIANTS.md` (CBM-ID-001, CBM-E-001, CBM-WIRE-001, CBM-WIRE-002, CBM-QUERY-001), `docs/plans/CBM_INTEGRATION_PLAN.md`, `extradocs/CBM_Clean-CTX_0.8.1_Findings.md`, `docs/plans/SEMANTIC-RELATIONSHIP-IMPLEMENTATION-PLAN.md` |

---

## 1. Context

Clean-CTX integrates with **Codebase-Memory-MCP (CBM)**, an external code-knowledge-graph server. The integration was built against **CBM 0.8.1**, which Clean-CTX launched as a subprocess and communicated with over stdio JSON-RPC/MCP. Under that model Clean-CTX effectively owned the full lifetime of an independent CBM runtime.

CBM has since evolved. A dedicated investigation compared the **CBM 0.8.1** baseline with **CBM 0.10.7** (source at `C:/Users/MNasty/Desktop/codebase-memory-mcp-0.10.7`), and a focused follow-up reviewed the **v0.10.7 → v0.10.8** delta. Key findings:

- **0.10.x introduces a daemon-backed MCP architecture.** A launched CBM executable is now an MCP *session/frontend* that coordinates through a per-account *daemon*. Multiple sessions share one daemon. The daemon owns watchers, shared indexing, and the optional UI. MCP servers are daemon-backed with no opt-out; only one-shot `cli` commands stay daemon-free.
- **Executable compatibility is coordinated.** All active CBM processes must share the same version, build fingerprint, coordination ABI, and canonical cache root. A conflicting start fails before doing work and records a conflict.
- **Storage moved to a persistent cache root.** `CBM_CACHE_DIR` (default `~/.cache/codebase-memory-mcp`, `%USERPROFILE%\.cache\codebase-memory-mcp` on Windows) replaces the old `%TEMP%`-based storage.
- **Retrieval output changed materially.** `search_graph`, `trace_path`, and `get_architecture` now emit a prefix-grouped *tree* model (text by default; `{cols, rows}` JSON under `format="json"`). `query_graph` retains the compatible `{columns, rows, total}` shape.
- **Richer intelligence, same gap.** CBM now exposes ranking, importance, architecture analysis (hotspots, Leiden clusters, boundaries, layers, cycles, services), data-flow and cross-service tracing, broader language coverage (159), and coverage/missed-graph information — but it still does **not** perform Clean-CTX's token-budget-aware context compilation.
- **The v0.10.7 → v0.10.8 delta is a single bugfix** (PR #1326, `fix(index): preserve persisted coverage summaries`): 2 files (`src/pipeline/coverage_contract.c` + 1 test), zero wire/tool/lifecycle changes. **0.10.7 is therefore an adequate architectural reference; 0.10.8 is the practical migration target.**

The investigation established the architectural boundary that governs this decision:

> **CBM is an intelligence provider, not a semantic authority.**

Clean-CTX retains ownership of semantic identity, semantic meaning, language/framework-aware extraction, authoritative relations, provenance, traversal, normalization, context compilation, and fidelity. CBM provides *optional* intelligence — discovery, ranking, architecture, retrieval, coverage — that must not automatically become Clean-CTX semantic facts.

---

## 2. Problem

The existing integration was designed around assumptions that CBM 0.10.x no longer holds:

1. **Obsolete lifecycle model.** The bridge assumes Clean-CTX owns the full lifetime of an independent CBM runtime. Under 0.10.x, a spawned executable is a daemon-coordinated *session*. Process startup, shutdown, daemon reuse, build/ABI compatibility, coordination ABI, and cache-root selection are now shared concerns.
2. **Broken wire contracts.** Three whitelisted tools (`search_graph`, `trace_path`, `get_architecture`) changed their response shapes. Existing Clean-CTX parsers cannot continue assuming the 0.8.1 formats.
3. **Duplicated intelligence.** Internal Cypher-based helpers (importance, dead-code, blast radius, call-edge) re-derive intelligence that CBM 0.10.x now exposes natively — if they still worked at all (the 0.8.1 `Function`-label bug returned empty results).
4. **No clean failure contract for the new daemon/session surface.** Daemon admission conflicts, cache-root pinning, and session teardown need explicit, retryable handling.
5. **Unclear boundary.** Without an explicit decision, there is a risk of importing CBM graph facts into the semantic substrate, redefining Clean-CTX identity from CBM FQNs, or reproducing CBM intelligence inside Clean-CTX.

---

## 3. Options considered

### Option A — Stay on CBM 0.8.1 integration model
Keep launching CBM as an independent subprocess and parsing 0.8.1 wire formats.

**Rejected.** CBM 0.8.1 is no longer the current architecture; the 0.8.1 binary is not the supported target. The daemon-backed model, the `Function`-label fix, richer intelligence, and the LLM-oriented tree output are all on 0.10.x. Staying means living with known-empty Cypher helpers and an obsolete lifecycle. CBM will not back-port.

### Option B — Adopt 0.10.x, full-bridge rewrite
Migrate everything at once: all six whitelisted tools, all internal helpers, plus the daemon/session lifecycle, importing CBM architecture intelligence into Clean-CTX wherever overlap exists.

**Rejected as a single step.** It collapses two distinct decisions — the *boundary* (what belongs to Clean-CTX vs CBM) and the *mechanics* (wire formats, lifecycle) — and risks importing CBM facts into the semantic substrate. The decision must be *bounded* first.

### Option C — Adopt 0.10.x with an explicit, narrowed boundary **(chosen)**
Migrate the lifecycle and the wire contracts, consume native CBM intelligence through a defined adapter surface, and *explicitly* keep CBM facts out of the semantic substrate. Retire/replace duplicated Cypher helpers. Preserve all ten architectural principles (§9).

### Option D — Decouple entirely; treat CBM as a pure external retrieval backend
Strip the bridge down to `query_graph` + raw proxy only.

**Rejected as premature.** CBM 0.10.x offers intelligence (architecture, ranking, data-flow, coverage) that is genuinely useful and not owned by Clean-CTX. A pure-retrieval stance discards usable capability. This option remains available as a future narrowing if the broader integration proves costly.

---

## 4. Decision

**Clean-CTX will migrate from the CBM 0.8.1 integration model to the CBM 0.10.x daemon-backed architecture, targeting CBM 0.10.8.**

The migration is **bounded**: it adopts the new lifecycle and the new wire contracts, consumes native CBM intelligence through an explicit adapter boundary, and preserves Clean-CTX's semantic authority. It does **not** import CBM graph facts into the semantic substrate, does **not** redefine Clean-CTX identity from CBM FQNs, and does **not** reproduce CBM intelligence that CBM already provides natively.

### 4.1 Decision rationale

1. **The 0.8.1 lifecycle assumption is factually obsolete.** The daemon-backed model is how 0.10.x works; the bridge must reflect it or fail on startup coordination, build conflicts, and cache-root pinning.
2. **Three wire contracts are already broken.** `search_graph`, `trace_path`, `get_architecture` changed shape regardless of any decision; the bridge must be updated to read them. Doing so against 0.10.8 is no more work than patching 0.8.1 parsers that CBM no longer emits.
3. **Native CBM intelligence is cheaper and more correct than re-deriving it.** The `Function`-label fix, degree/relationship filters on `search_graph`, and architecture analysis are first-class in 0.10.x. Clean-CTX should consume them, not reproduce them with fragile Cypher.
4. **The boundary is the load-bearing decision.** The most important outcome is *not* which tools are called but the rule that **CBM facts are advisory**. Everything else (surface, formats, sequencing) follows from that.
5. **0.10.7 → 0.10.8 adds no architectural risk.** A 2-file bugfix delta means the 0.10.7 investigation is fully representative; targeting the released 0.10.8 avoids building on an unreleased tag.

---

## 5. Chosen CBM target version

**CBM 0.10.8** (tag `v0.10.8`, commit `46ae198fc`, released 2026-08-19).

Rationale: it is the latest released version with a published release page and release assets. The v0.10.7 → v0.10.8 delta is a 2-file bugfix (`coverage_contract.c` + 1 test) with zero wire/tool/lifecycle changes, so the architectural findings from the 0.10.7 investigation carry over unchanged. Targeting a released version (rather than the unreleased `v0.10.7` tag) ensures installable assets and a stable reference.

---

## 6. Subprocess / session lifecycle model

This is the critical lifecycle decision. The old model — *"Clean-CTX owns the full lifetime of an independent CBM runtime"* — is replaced:

> **Clean-CTX owns its CBM MCP session/process; CBM may coordinate that session through a shared local daemon.**

### 6.1 Verified behavior (CBM 0.10.x)

| Phase | Verified CBM behavior | Evidence class |
|---|---|---|
| Process startup | `Command::new(binary)` still starts a CBM executable. | `src/main.c` |
| Daemon coordination | MCP servers are daemon-backed (no opt-out). On start, the frontend runs startup coordination ("Waiting for CBM startup coordination...") against the per-account daemon, with a deadline. | `src/main.c` (`main_local_transition_acquire`), `src/daemon/*`, README |
| Daemon reuse | One shared daemon serves multiple sessions; first session starts it, last session stops it. | README |
| Build/ABI compatibility | All active processes must share version + build fingerprint + coordination ABI + canonical cache root. A conflicting start fails before work and logs `daemon-conflicts.ndjson`. | `src/daemon/version_cohort.c`, `foundation/private_file_lock.c`, README |
| Cache root | `CBM_CACHE_DIR` (default `~/.cache` / `%USERPROFILE%\.cache\codebase-memory-mcp`) — persistent, no longer `%TEMP%`. | `cli.c`, `application.c`, README env table |
| Existing CBM instances | Honored via daemon reuse; conflicts only on genuine version/build/cache-root mismatch. | version_cohort, README |
| Startup failure | "A client that cannot reach the daemon must say so, in the caller's own protocol" — failure reported over JSON-RPC, not a silent hang. | `src/main.c` |
| Process shutdown | Clean-CTX still terminates its own process; daemon-owned work for that session is canceled; daemon stops when last session exits. | README |

### 6.2 Clean-CTX responsibilities in the new model

Clean-CTX **continues to launch CBM as a subprocess** (a session/frontend), but must:

1. **Pin `CBM_CACHE_DIR`** in the spawn environment to a stable, Clean-CTX-managed canonical root (a `cbm.cache_root` config concept) so project slugs and persistence are deterministic and do not collide with another tool's CBM cache.
2. **Treat daemon/session conflicts as retryable.** A `daemon-conflicts.ndjson` admission failure (version/build/cache-root mismatch) is distinct from "CBM absent"; it should surface as a specific, retryable `CbmError` variant — *not* be silently converted to empty success (preserving CBM-E-001).
3. **Honor the compatibility invariant.** Clean-CTX must not assume it can run a different CBM build than an already-active daemon. In practice: one CBM version per cache root.
4. **Handle startup-coordination deadline.** The daemon-coordination wait has its own deadline; Clean-CTX's query timeout should not double-count it. A distinct startup-timeout classification is warranted.
5. **Graceful degradation when the daemon is unreachable.** Per CBM's own contract, an unreachable daemon is reported over JSON-RPC; Clean-CTX treats this as CBM-unavailable and continues without enrichment (the existing additive-only policy).

### 6.3 Should Clean-CTX attach to an already-running daemon?

**Yes, implicitly — and it already will.** Launching the CBM executable as an MCP server automatically joins the shared daemon if one is running (daemon reuse), or starts one if not. Clean-CTX does **not** need a separate "attach" path; it owns its *session*, and CBM handles daemon discovery/reuse transparently. Clean-CTX must only ensure its `CBM_CACHE_DIR` and CBM build are compatible with any daemon it may join.

---

## 7. Clean-CTX vs CBM responsibility boundary

| Responsibility | Owner | Principle |
|---|---|---|
| Semantic identity `(domain, entity_type, name)` | **Clean-CTX** | SEM-001 |
| Semantic meaning / framework & language extraction | **Clean-CTX** | SEM substrate |
| Authoritative semantic relations | **Clean-CTX** | SEM-001…SEM-008 |
| Source provenance (`EntityRef.file`) | **Clean-CTX** | SEM-002 |
| Semantic traversal / normalization | **Clean-CTX** | WorkspaceIndex, `workspace_query` |
| Context compilation & fidelity / compression | **Clean-CTX** | IR pipeline, fidelity engine |
| FQN / path-qualified identity (external) | **CBM** (external) | SEM-019 |
| Graph discovery / BM25 / semantic retrieval | **CBM** | optional intelligence |
| Importance / ranking / degree | **CBM** | advisory only |
| Architecture analysis (hotspots, clusters, boundaries, layers, cycles, services) | **CBM** | advisory only |
| Data-flow / cross-service / cross-repository intelligence | **CBM** | advisory only |
| Coverage / missed-graph information | **CBM** | advisory only |
| Indexing & project lifecycle (CBM side) | **CBM** | daemon-managed |
| Session / daemon coordination (CBM side) | **CBM** | transparent to Clean-CTX |

**Authority rule:** when both systems describe overlapping concepts, **Clean-CTX semantic facts win**. CBM-derived facts are advisory and may inform context compilation but must not be imported into the semantic substrate as authoritative relations, entities, or provenance.

---

## 8. Integration surface

### 8.1 Production proxy surface (whitelisted tools)

All six whitelisted tools **remain** on CBM 0.10.8. CBM 0.10.8 adds one tool (`check_index_coverage`) not currently whitelisted.

| Tool | Status | Wire change |
|---|---|---|
| `search_graph` | KEEP | **Response shape changed** (tree/`{cols,rows}`); new `query`/`semantic_query`/`relationship`/`direction`/`min_degree`/`max_degree`/`fields`/`format` params |
| `query_graph` | KEEP | Compatible (`{columns, rows, total}`); new `max_rows`, `graph="missed"` |
| `trace_path` | KEEP | **Response shape changed** (tree/`{cols,rows}`); new `mode` (`calls`/`data_flow`/`cross_service`), `format` |
| `get_architecture` | KEEP | **Response shape changed** (aspects/tree/`{cols,rows}`); new `aspects`, `scope`; `boundaries` now real |
| `list_projects` | KEEP | Additive pagination (`offset`/`limit`/`total`/`has_more`) |
| `index_repository` | KEEP | Additive (`mode` enum extended with `cross-repo-intelligence`, `name`, `persistence`, COVERAGE block) |

### 8.2 Surface capabilities (unchanged in role)

- Raw CBM proxy (`cbm_proxy` pipe-level interception + `compress_cbm_response`) — keep.
- Project slug derivation (`cbm_project_slug`) — keep (canonical-path scheme unchanged).
- Cache-store concepts (`query_graph` `cypher:`/`cypher2:` namespace) — keep.
- Timeout / retry / circuit-breaker concepts — keep, subject to daemon/session changes (§6.2).

### 8.3 Internal Cypher helpers — retire / replace

| Helper | Current impl | 0.10.7+ status | Action |
|---|---|---|---|
| `get_symbol_importance` | Cypher `MATCH (f:Function) WHERE in_degree >= …` | Now returns data (`Function` label exists), but natively redundant | **Retire**; consume via `search_graph(relationship=…, direction=inbound, min_degree=…)` when/if needed |
| `get_dead_code` | Cypher `Function` + `Method` | Now fully works | **Retire** as internal Cypher; natively available via `index_status`/coverage + `search_graph(min_degree=0)` |
| `get_blast_radius` | Cypher `MATCH (caller:Function)-[:CALLS]->(f:Function)` | Now returns data, but redundant | **Retire**; native `trace_path(depth=n)` replaces it |
| `get_call_edges` | Cypher `MATCH (a:Function)-[:CALLS]->(b:Function)` | Now returns data, but redundant | **Retire**; native `search_graph`/`query_graph` replaces it |

Rationale: these helpers were either broken or duplicating native capability. Per principle 6, Clean-CTX must not reproduce CBM intelligence that CBM already provides natively. They should not be migrated to 0.10.x; they should be removed and their (currently nonexistent) production consumers redirected to native tools or dropped.

### 8.4 Fidelity / skip-set helpers

`cbm_informed_fidelity` / `build_cbm_skip_set`: no production consumer exists and 0.10.7 introduced no new native fidelity capability that would create one. **Keep as optional / do not wire to production** unless a real consumer is approved. They remain compatible (importance semantics unchanged).

---

## 9. Architectural principles preserved

This decision preserves all ten governing principles:

1. Clean-CTX remains independently functional without CBM (CBM absent → additive-only degradation, unchanged).
2. CBM remains optional intelligence, not a semantic dependency.
3. Clean-CTX semantic facts are authoritative within Clean-CTX.
4. CBM facts are advisory unless explicitly incorporated through a future approved contract (none approved here).
5. CBM FQNs do not redefine Clean-CTX semantic identity (`(domain, entity_type, name)` unchanged).
6. Clean-CTX does not reproduce CBM ranking/architecture intelligence (§8.3 retires the re-deriving Cypher helpers).
7. Clean-CTX remains responsible for final context compilation and fidelity (CBM still does no token-budget-aware compilation).
8. CBM output is treated as an external integration contract, not part of the Clean-CTX semantic substrate (§7 authority rule).
9. The bridge tolerates CBM wire-format evolution through an explicit adapter boundary (§10 wire-format migration).
10. The subprocess/session lifecycle reflects the daemon-backed CBM architecture (§6), not the obsolete 0.8.1 model.

---

## 10. Wire-format migration requirements

The bridge must adapt three typed response parsers and the format negotiation for tree output. This is the explicit adapter boundary (principle 9).

| Parser / path | 0.8.1 assumption | 0.10.8 reality | Migration |
|---|---|---|---|
| `map_search_result` | `results[]` of `{name, qualified_name, label, file_path, in_degree, out_degree}` objects | JSON is now `{total, count, cols, groups[{qn_prefix, file, rows[]}], has_more}`; default output is text tree | Read `format="json"`; reconstruct FQN as `qn_prefix + "." + name`, label/lines/in/out from row columns by `cols` index; `file` from group |
| `extract_trace_edges` | `callers[]`/`callees[]` of `{name, qualified_name, hop}` | JSON is `{function, direction, callees_total, callees:{cols,rows}, callers_total, callers:{cols,rows}}`; new `mode` | Read `format="json"`; parse `{cols,rows}` tables; honor `callers_total`/`callees_total` |
| `get_architecture` | flat `packages[]` / `boundaries[]` object arrays | JSON sections are `{cols, rows}` trees; default compact summary; `boundaries` only when requested via `aspects` | Parse `packages`/`boundaries` as `{cols,rows}`; request `aspects` explicitly when full data needed |
| `query_graph` | `{columns, rows, total}` | Unchanged | No migration; cache key (`cypher2:`) stable |
| Tree format negotiation | N/A (always JSON) | `format` enum `tree` (default) / `json` | Typed paths must pass `format:"json"`; raw proxy path passes text tree through unchanged (acceptable for LLM) |

To **tolerate future wire evolution**, parsers must be isolated behind a thin adapter layer keyed on tool + response shape, with shape-detection (presence of `cols`/`rows` vs `results[]`) rather than version sniffing, and a documented per-tool contract (an evolution of `CBM-WIRE-001`, `CBM-WIRE-002`, `CBM-QUERY-001`).

---

## 11. Capabilities to consume

Consume these CBM 0.10.8 capabilities through the integration surface (principle 6 — use native rather than reproduce):

| Capability | Consumed via | Value |
|---|---|---|
| Graph / BM25 / semantic retrieval | `search_graph` (3 modes, `fields`, degree filters) | Primary code-discovery path for agents |
| Cypher graph querying | `query_graph` (stable `{columns,rows,total}`) | Corroboration + custom traversal |
| Call-graph / data-flow / cross-service tracing | `trace_path` (`calls`/`data_flow`/`cross_service`, depth) | Impact + data-flow analysis (replaces blast-radius helper) |
| Architecture overview | `get_architecture` (aspects: hotspots, Leiden clusters, boundaries, layers, cycles, services) | Orientation intelligence for agents |
| Project + index lifecycle | `list_projects`, `index_repository`, `index_status`, `check_index_coverage` | Index health + coverage honesty |
| Cross-repository intelligence | `index_repository(mode=cross-repo-intelligence)` + `CROSS_*` edges | Optional future: cross-repo discovery |

Consumption is **advisory**: these inform context compilation and agent answers but do not create Clean-CTX semantic entities or relations.

---

## 12. Capabilities explicitly not consumed

The following CBM capabilities are **not** imported into the Clean-CTX semantic substrate, even though CBM exposes them (principles 4, 5, 8):

- Architecture clusters / hotspots / boundaries / layers / cycles / services (used only as advisory context).
- Importance / ranking / degree scores (advisory; no `get_symbol_importance` migration).
- Data-flow facts (`DATA_FLOWS`, `trace_path(mode=data_flow)` — available via raw proxy, not imported).
- Cross-repository relationships (`CROSS_*` edges — available, not imported).
- CBM-specific identity / FQNs / qualified names as Clean-CTX identity (SEM-001/SEM-019).
- Coverage / missed-graph info (available via `index_status`/`check_index_coverage`, advisory only).
- The new `check_index_coverage` tool (available but not whitelisted until a consumer is approved).

---

## 13. Failure / degradation behavior

Clean-CTX degrades exactly as it does today — CBM is strictly additive — but the *failure modes* expand with the daemon/session surface:

| Condition | Behavior | Contract |
|---|---|---|
| CBM binary absent / not on PATH | Clean-CTX runs without enrichment; no semantic impact | Unchanged (existing) |
| CBM unreachable at startup | Reported over JSON-RPC per CBM's contract; Clean-CTX treats as CBM-unavailable | Unchanged in effect |
| **Daemon admission conflict** (version/build/cache-root mismatch) | Distinct, **retryable** `CbmError` variant; NOT converted to empty success (preserves CBM-E-001) | **NEW** — must be a specific error class |
| **Startup-coordination timeout** | Distinct from query timeout; surfaced as startup-timeout, not a query failure | **NEW** |
| CBM tool failure mid-session (`result.isError`, transport fault) | Propagates as `Err(CbmError)`; `Ok(empty)` reserved for valid zero-result queries | Unchanged (CBM-E-001) |
| Query timeout | Existing query-timeout contract | Unchanged |
| CBM wire-shape regression | Adapter layer (§10) isolates shape detection; unknown shapes degrade to "no parsed data", not a crash | **NEW resilience requirement** |

**Failure rule:** every CBM failure is either `Err(CbmError)` or a clean absence; nothing may masquerade as empty success.

---

## 14. Compatibility requirements

1. **Pin `CBM_CACHE_DIR`** in the spawn environment to a Clean-CTX-managed canonical root so slugs/persistence are deterministic and isolated from other CBM consumers.
2. **One CBM version per cache root.** Clean-CTX must not launch a build that conflicts with an already-active daemon on the same root.
3. **Preserve `CBM-ID-001`** (canonical project identity): slug derivation, per-root registration, readiness isolation, proxy-gate scoping all remain authoritative.
4. **Preserve `CBM-E-001`** (explicit error propagation): `Ok(empty)` only for valid zero-result queries; every CBM failure → `Err`.
5. **Evolve `CBM-WIRE-001` / `CBM-WIRE-002` / `CBM-QUERY-001`** to the 0.10.8 tree/`{cols,rows}` shapes, with shape-detection rather than version sniffing.
6. **Stable contracts Clean-CTX may keep depending on:** FQN scheme, project slug derivation, `query_graph` `{columns,rows,total}`, top-level `callers`/`callees`/`function` keys, core `get_code_snippet` keys, `search_code` scoring formula, base edge labels (`CALLS/INHERITS/IMPORTS/USAGE/DEFINES*/CONTAINS_*`).
7. **Accidental 0.8.1 assumptions to drop:** per-row key envelopes; flat architecture arrays; JSON-by-default; `Function` label absent; `boundaries` absent; `%TEMP%` storage; uncoordinated-server model.

---

## 15. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Daemon admission conflicts surprise users (new failure mode) | Medium | Specific retryable `CbmError` class; clear messaging; pin `CBM_CACHE_DIR` |
| Tree-format parsers mis-handle edge shapes | Medium | Adapter layer with shape-detection; per-tool contract tests |
| Accidental import of CBM facts into substrate | Medium | Authority rule (§7); SEM-001/SEM-019; code-review gate |
| CBM version skew between Clean-CTX and an existing daemon | Low-Medium | One version per cache root; document the invariant |
| `CBM_CACHE_DIR` relocation breaks existing test fixtures / assumptions | Low | Pin explicitly; update fixtures to the new root |
| Retiring Cypher helpers removes capability with hidden consumers | Low | No production consumer exists (verified); grep before removal |
| 0.10.8 post-release patch changes a wire shape again | Low | Adapter layer + shape-detection absorbs minor evolution |

---

## 16. Non-goals (explicitly out of scope)

- Redesigning `EntityRef` or Clean-CTX semantic identity.
- Changing WorkspaceIndex architecture.
- Importing CBM graph facts into the semantic substrate.
- Broadening the Clean-CTX semantic model to match CBM.
- Implementing the migration (this is a decision document only).
- Adopting CBM tools beyond the six whitelisted (except `index_status`/`check_index_coverage` for lifecycle, advisory).
- Reproducing CBM ranking/architecture intelligence inside Clean-CTX.
- Supporting CBM < 0.10.8 after migration.

---

## 17. Migration sequencing

1. **Adapter boundary first** — isolate CBM response parsing behind a shape-detecting adapter keyed on tool + response shape (§10). This is the single most important structural change; it absorbs both the 0.10.8 shapes and future evolution.
2. **Wire-format migration** — update `map_search_result`, `extract_trace_edges`, `get_architecture` to the tree/`{cols,rows}` model; pass `format:"json"` on typed paths.
3. **Lifecycle adaptation** — pin `CBM_CACHE_DIR`; add daemon-conflict and startup-timeout `CbmError` classes; update process startup/shutdown for daemon coordination.
4. **Retire Cypher helpers** — remove `get_symbol_importance`, `get_dead_code`, `get_blast_radius`, `get_call_edges` (no production consumers); redirect any future need to native tools.
5. **Evolve wire invariants** — update `CBM-WIRE-001/002`, `CBM-QUERY-001` to 0.10.8 shapes in `ARCHITECTURAL_INVARIANTS.md`.
6. **Optional intelligence** — wire `index_status`/`check_index_coverage` for lifecycle; keep fidelity/skip-set unwired until a consumer is approved.

---

## 18. Consequences

**Positive:**
- Clean-CTX adopts the supported, current CBM architecture instead of an obsolete one.
- Broken/empty Cypher helpers are replaced by working native capabilities.
- Richer intelligence (architecture, data-flow, coverage, cross-repo) becomes available to agents.
- LLM-oriented tree output reduces token overhead vs 0.8.1 envelopes.
- Explicit boundary prevents scope creep of CBM facts into the substrate.

**Costs / constraints:**
- Three wire parsers must be rewritten; tree-format handling adds adapter complexity.
- Daemon coordination introduces new failure modes (conflicts, startup timeout) that must be handled and explained to users.
- `CBM_CACHE_DIR` must be pinned and documented.
- One CBM version per cache root is a new operational constraint.

**Neutral:**
- CBM remains optional; Clean-CTX is fully functional without it (unchanged).

---

## 19. Related architectural invariants / documents

- `docs/SEMANTIC_SUBSTRATE_ARCHITECTURE.md` — SEM-001…SEM-019 (authoritative; unchanged).
- `docs/ARCHITECTURAL_INVARIANTS.md` — CBM-ID-001, CBM-E-001, CBM-WIRE-001, CBM-WIRE-002, CBM-QUERY-001 (to be evolved to 0.10.8 shapes).
- `docs/plans/CBM_INTEGRATION_PLAN.md` — prior integration plan (superseded in part by this decision).
- `extradocs/CBM_Clean-CTX_0.8.1_Findings.md` — 0.8.1 baseline investigation.
- `docs/plans/SEMANTIC-RELATIONSHIP-IMPLEMENTATION-PLAN.md` — semantic substrate implementation (unaffected by CBM migration).

---

## 20. Decision Summary

**Clean-CTX is moving to the newer CBM architecture.**

- **Target version:** **CBM 0.10.8** (latest released; the v0.10.7 → v0.10.8 delta is a 2-file bugfix with zero architectural change, so the 0.10.7 investigation fully represents it).
- **Subprocess:** Clean-CTX **continues to launch CBM as a subprocess** — but that subprocess is now understood as an **MCP session/frontend**, not an independently owned runtime.
- **Daemon relationship:** Clean-CTX owns its session/process; CBM coordinates that session through a **shared local daemon** (auto-reused across sessions, started by the first, stopped by the last). Clean-CTX must pin `CBM_CACHE_DIR` and honor one-CBM-version-per-cache-root.
- **CBM is responsible for:** graph intelligence (discovery, ranking, BM25/semantic retrieval, architecture analysis, data-flow/cross-service/cross-repo tracing, coverage), indexing/project lifecycle, and session/daemon coordination.
- **Clean-CTX is responsible for:** semantic identity, meaning, framework/language-aware extraction, authoritative relations, provenance, traversal, normalization, context compilation, and fidelity. CBM output is an external integration contract, never part of the semantic substrate.
- **Immediately in scope:** the six whitelisted tools (with wire-format migration for `search_graph`, `trace_path`, `get_architecture`); retirement of the four internal Cypher helpers in favor of native CBM capabilities; lifecycle adaptation (cache-root pinning, daemon-conflict + startup-timeout error classes).
- **The single most important architectural rule:** **CBM is an intelligence provider, not a semantic authority — CBM facts are advisory, Clean-CTX semantic facts are authoritative, and CBM graph facts must never be imported into the Clean-CTX semantic substrate.**

---

*This document records an architectural decision only. It does not authorize or describe implementation. All implementation is subject to the project's standard phase lifecycle, verification gate, and architectural checkpoint rules.*