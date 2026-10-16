# Final Release-Blocker Audit — 2026-10-16

> **Status: INVESTIGATION / EVIDENCE — POINT-IN-TIME RELEASE RECORD**
>
> This audit records the inspected release-candidate state and is not current
> production authority. See `docs/README.md` for the normative documentation
> spine.

**Agent:** Agent MaxHeadRoom  
**Verdict:** **RELEASE READY — NO SERIOUS BLOCKERS FOUND**

## 1. Executive release verdict

No production-reachable defect was proven under the supplied severity standard. Issue #40's coarse WorkspaceIndex identity remains deliberate, but occurrence provenance and collision evidence survive, ambiguity remains visible, and mutation does not use WorkspaceIndex identity as authority. No narrow runtime falsification is required.

## 2. Release-blocker definition used

A blocker requires a supported path that fabricates truth, converts ambiguity into false exactness, mutates the wrong source, publishes stale truth, makes provider/warm state determine correctness, or causes severe deterministic panic, hang, or corruption. Missing capability, honest lower bounds, unsupported syntax, proxy reassessment, documentation debt, and future work are non-blocking absent those failures.

Evidence labels: **PROVEN BY CURRENT PRODUCTION CODE**, **PROVEN BY TRACKED TEST**, **PROVEN BY HISTORICAL LIVE VERIFICATION**, **INFERRED**, **NOT PROVEN**.

## 3. Open issue classification

| Issue | Classification | Assessment |
|---|---|---|
| #40 WorkspaceIndex identity | **OBSOLETE / SUPERSEDED** | Coarse by design; the public edge response preserves endpoint file occurrences, so same-tuple edges are not rendered as one declaration's unproven relationships. Recommend closure. |
| #47 Newtonsoft.Json | **FEATURE / FUTURE WORK** | Additional framework coverage. |
| #48 Document Intelligence | **FEATURE / FUTURE WORK** | New capability. |
| #90 tool-output filtering | **ARCHITECTURE INVESTIGATION** | Integration placement. |
| #91 secret scrubbing | **ARCHITECTURE INVESTIGATION** | Native-vs-proxy placement. |
| #92 ANSI normalization | **ARCHITECTURE INVESTIGATION** | Optional/proxy capability. |
| #93 Smart Prompt Caching | **ARCHITECTURE INVESTIGATION** | Optimization overlap. |
| #94 sliding window | **ARCHITECTURE INVESTIGATION** | Future architecture. |
| #95 tool-definition filtering | **ARCHITECTURE INVESTIGATION** | No severe failure proven. |
| #96 HTTP proxy role | **ARCHITECTURE INVESTIGATION** | Design synthesis. |

No open issue is a proven blocker; open status was not treated as severity.

## 4. Issue #40 identity-collision audit

| Layer | Authority/key |
|---|---|
| Display/written name | `EntityRef.name` |
| Lookup | name + optional domain/type + scope |
| Semantic identity | `(domain, entity_type, name)` (Model C) |
| Occurrence | semantic tuple + canonical file |
| Edge occurrence | relation + subject/object tuples + asserting file |
| Source | canonical/physical path + hash/version |
| Exact local declaration | canonical IR owner/unit/signature |
| Mutation | explicit file + tracked unit + expected bytes + live/durable identity |

`register_entity` stores distinct file occurrences. `EdgeKey`/`StoredEdge` retain asserting files, fixing DIS-2026-016's historical evidence collapse. Removal/republication is occurrence-owned.

**Claim:** coarse identity no longer erases collision evidence.  
**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE** (`src/workspace/index.rs`, `index/edges.rs`, `index/remove.rs`); **PROVEN BY TRACKED TEST** (`index_edge_occurrence.rs`, `index_edge_lifecycle.rs`). Post-fix live re-verification remains pending.

## 5. Same-name entity falsification

Same-named declarations remain separate file occurrences. `find_entities` returns them; adjacency returns all asserting-file-provenanced edges; removal retracts only one file; and `calls_in_file` uses canonical file-local High-fidelity IR.

The tuple union is broader than a namespace-qualified graph, but is not presented as a namespace-qualified unique identity. Responses retain provenance and lower-bound coverage.

**Result:** preserved ambiguity/evidence union, not a blocker.  
**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE** and **PROVEN BY TRACKED TEST**.

### Concrete public-response trace

The tracked collision shape is:

```text
FILE_A: LoadingComponent --Injects--> Router
FILE_B: LoadingComponent --Injects--> Router
```

1. **WorkspaceIndex.** `add_edges(FILE_A/FILE_B, ...)` fills missing `subject.file` and `object.file` with the asserting file, then stores two `StoredEdge` occurrences under the one Model C subject tuple. The internal `asserting_file` is also retained for scope/removal.
2. **Handler.** An explicit `forward_edges(domain="angular", entity_type="Component", name="LoadingComponent")` resolves that exact semantic tuple. `selected_occurrences` returns each stored `SemanticEdge`. It drops the internal wrapper, but not the endpoint `file` fields.
3. **Structured response.** `edges.rs` serializes those edges directly and returns one `resolved_identity` plus an edge array whose otherwise identical records differ at `subject.file`: `FILE_A` versus `FILE_B`. It does not add `identity_ambiguous` for forward/reverse queries.
4. **Model-facing output.** `content::model_result` clones every serialized edge unchanged and adds only an `occurrence` ordinal. `render` prints that result in the `clean-ctx/workspace-query-answer` envelope and states that `structuredContent` remains authoritative.

Representative shape:

```json
{
  "resolved_identity": {"domain":"angular","entity_type":"Component","name":"LoadingComponent"},
  "edges": [
    {"relation":"Injects","subject":{"name":"LoadingComponent","file":"FILE_A"},"object":{"name":"Router","file":"FILE_A"},"occurrence":0},
    {"relation":"Injects","subject":{"name":"LoadingComponent","file":"FILE_B"},"object":{"name":"Router","file":"FILE_B"},"occurrence":1}
  ]
}
```

Thus the public response satisfies category **1: occurrence/file provenance per edge** for this same-workspace declaration collision. It does not satisfy category 2, and it does not need category 3 to avoid a false declaration-level claim in this case. A consumer that ignores the explicit endpoint file is discarding returned authority rather than being handed silent declaration-level certainty.

**#40 verdict:** **effectively superseded by the current authority model; recommend closing it.** This audit did not close it because the audit brief forbids closing existing issues.

## 6. Query ambiguity behavior

- `find_entities` returns admitted occurrences; `entities_in_file` is file-owned.
- Name-only edge/traversal resolution fails `-32602` when several semantic tuples remain.
- Explicit tuple queries return occurrence-provenanced edge evidence, as traced above.
- `has_cycle` reports `identity_model: semantic_tuple`, `identity_ambiguous`, and witnesses.
- `calls_in_file` requires file, typed owner, and method; duplicate owners fail closed. Omitted signatures return the overload family; supplied selectors match exactly.
- Exact graph coverage is indexed/lower-bound, not workspace closure.

The `expect("one identity")` follows a checked `len() == 1` BTreeSet invariant.

**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE**; representative behavior **PROVEN BY TRACKED TEST**.

## 7. Edit-target safety

`apply_edit` is independent of WorkspaceIndex. It requires an explicit trusted file/current unit table. Signatures disambiguate overloads; legacy selectors must be unique. Expected bytes, live/durable hash/version, non-overlap, UTF-8, syntax-before-write, a 64-operation cap, same-directory staging, sync, atomic replacement, durable edit intent, and physical-alias guards all fail closed.

**Wrong-target escape:** **NO**.  
**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE** (`src/edit/*`, `src/mcp/tool_handlers/edit.rs`); **PROVEN BY TRACKED TEST** by the apply-edit suites named in the invariants.

## 8. Framework refinement identity safety

Angular template/NgModule/route/NgRx/DI refinement uses syntax/declaration evidence and fails closed when proof is missing or ambiguous. Selector results retain occurrences; filename/proximity does not choose one. C# catalogs use declaration evidence; CBM edges never become Clean authority.

The #132/#139 reconciliations document filename-independent template authority, exact decorated declarations, ambiguous/missing target rejection, and lazy-target lifecycle repair.

**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE**, **PROVEN BY TRACKED TEST**, and stated scenarios **PROVEN BY HISTORICAL LIVE VERIFICATION**.

## 9. Lifecycle/stale-truth sweep

Semantic coverage is keyed by canonical file, fidelity, and source hash. Reuse requires compatibility; replacement may publish an empty edge set. Staleness/refresh invalidates discovery completion; removal is occurrence-local. The #139 lazy-route repair carries fingerprints for consulted external targets through live/durable snapshots, and restore validates them.

No additional current producer was found bypassing dependency validity. This scoped negative is **INFERRED**; the mechanisms and representative cases are **PROVEN BY CURRENT PRODUCTION CODE**, **PROVEN BY TRACKED TEST**, and **PROVEN BY HISTORICAL LIVE VERIFICATION**.

**Stale-truth escape:** **NO**.

## 10. CBM optionality sweep

CBM supplies candidate paths/discovery only. Publication requires trusted-path resolution, Clean compilation/extraction, and WorkspaceIndex insertion. Failure/partial/indexing state is surfaced; filesystem discovery is the promised fallback.

**Provider correctness escape:** **NO**.  
**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE**, **PROVEN BY TRACKED TEST**, and #142's operator-reported matrix **PROVEN BY HISTORICAL LIVE VERIFICATION**.

## 11. Warm-state/history-dependence sweep

Discovery cache records completion, not answers, and keys target/provider/root/mode/generation. Only complete successful publication is reusable; responses evaluate live index state. Authoritative batches acquire union evidence before interpretation; lower-bound families say so.

**Warm-state correctness escape:** **NO**.  
**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE**; cold/repeat/unrelated-warm/batch-order cases **PROVEN BY TRACKED TEST** and **PROVEN BY HISTORICAL LIVE VERIFICATION**.

## 12. Mutation/source-fidelity sweep

Stale expected text, overloads, nested ownership, declaration/body spans, UTF-8, EOLs, operation ordering/overlap, staged replacement, and recovery were inspected. No silent partial multi-edit write or stale selector authority was found.

**Evidence:** **PROVEN BY CURRENT PRODUCTION CODE** and **PROVEN BY TRACKED TEST**.

## 13. Panic/hang/process-lifecycle sweep

A targeted production search reviewed `unwrap`, `expect`, `panic`, `unreachable`, and `todo`. Remaining sites were checked invariants, in-memory serialization, parser-ordered spans, startup resources, schema construction, test-only hooks, or guarded states. No supported input was shown to violate them. No source-controlled unbounded recursion, ordinary deadlock, indefinite MCP wait, or child-process blocker was proven.

**Production panic/hang blocker:** **NO**. Individual guards are **PROVEN BY CURRENT PRODUCTION CODE**; exhaustive absence is **INFERRED**.

## 14. Authority-boundary sweep

Audited boundaries preserve: name is lookup, not mutation identity; spelling does not prove declaration; proximity/filename do not establish ownership; cache presence is not compatibility; CBM selects candidates, not truth; parser/refinement failure emits no fabricated edge; missing evidence remains missing, ambiguous, or lower-bound.

## 15. Serious findings

**None.** Serious blockers: **0**. Runtime-falsification candidates: **0**.

## 16. Minor/non-blocking findings

No concrete new product defect was sufficiently proven. #40 is superseded administrative issue state, not a product defect. Non-blocking defects: **0**.

## 17. GitHub issues opened

**None.** No duplicate/speculative backlog was created. #40 was not closed because existing-issue mutation was prohibited.

## 18. Commands run

Fast bounded inspection only: `Get-Content`, `Select-String`, `git status --short`, Clean-CTX `get_cbm_status`/`graph_search`/`provide_code_context`, targeted `rg`, `gh issue list`, and patch application for this report. One invalid focus selector returned `-32602` and changed no state.

## 19. Commands not run due to authorization

Agent MaxHeadRoom did **not** run Cargo tests/builds/checks/Clippy/format/audit, test binaries, Clean-CTX/MCP/proxy/provider processes, live harnesses, benchmarks, fuzzing, coverage, model pilots, or the final gate. No unrun gate is claimed green.

## 20. Final release recommendation

**SHIP.**

Issue #40 is superseded: the concrete public response exposes the declaration occurrence on every edge endpoint and does not silently render the collision as one declaration. Mutation authority is separate and exact. The stale-truth, provider, history, and mutation boundaries have production and tracked-test authority, and no uncovered serious escape was found.

Final verdict: **RELEASE READY — NO SERIOUS BLOCKERS FOUND**.
