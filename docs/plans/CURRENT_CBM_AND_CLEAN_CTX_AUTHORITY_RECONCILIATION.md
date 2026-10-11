# Current CBM and Clean-CTX authority reconciliation

Date: 2026-10-11. Investigator: Agent MaxHeadRoom.
Scope: read-only implementation/history investigation requested before continuing
Issue #142. This document is the sole deliverable change. No production code,
test, configuration, dependency, or historical branch was changed. No merge,
rebase, server, build, Cargo verification, or live diagnostic trial was run.

Pinned evidence:

- Current Clean-CTX: `4c1627fcefe698977447dcee4e88830c20c7f422`,
  `(feat)Semantic_Intelligence_Architecture_Boundaries`; Cargo package 0.9.3.
- Historical Clean-CTX: `c899507678f7188e5fd161625bf92acdc2697d4c`,
  `feature/update-CBM-0.10.8`, last commit 2026-09-07; package 0.6.1.
- Current CBM fork main:
  `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`, inspected 2026-10-11,
  https://github.com/codeliftsleep2/codebase-memory-mcp-clean-ctx.
- Historical CBM comparison anchor: `5821078e`, 2026-08-29, packaging version
  correction to 0.10.8. This is an inspected reference revision, **not an exact
  source pin recovered from the old Clean-CTX branch**.

Evidence vocabulary: **PROVEN BY CLEAN-CTX CODE**, **PROVEN BY CLEAN-CTX TEST**,
**PROVEN BY CBM CODE**, **PROVEN BY CBM TEST**, **DOCUMENTED BY CBM**,
**HISTORICAL BRANCH DECISION**, **INFERENCE**, **PROPOSAL**, **NOT ESTABLISHED**.
A TEST label means the inspected tracked test asserts that property; it does
not mean the test passed in this investigation. No new live evidence is claimed.
Paths under CBM refer to the pinned fork; historical Clean-CTX paths refer to
its pinned branch. Current paths otherwise refer to this checkout.

## 1. Executive verdict

| Decision gate | Single verdict | Reason |
| --- | --- | --- |
| Old branch | **MINE SELECTIVELY** | Optionality and request-local advisory separation remain useful; current Clean-CTX lifecycle, batch, framework and invocation machinery has diverged. Selected wire/health tests are useful, but the old adapter does not capture all current paging semantics. |
| CBM role | **COMBINATION OF THE ABOVE** | Discovery accelerator plus optional advisory intelligence; optional resolution enrichment is a promising next use, conditional on source/site identity and explicit coverage. Direct core semantic authority is not earned by this audit. |
| #142 | **MORE LIVE EVIDENCE REQUIRED** | Current CBM gives stronger lifecycle/paging signals, but static Clean-CTX completion, projection-adequacy and catalog-snapshot findings remain. Current adapter compatibility must be established before a current-CBM runtime comparison is meaningful. |

**Controlling answer:** without CBM installed, Clean-CTX must correctly acquire
current authorized source, produce its canonical IR and supported language/meta
semantics, compress it at requested fidelity, preserve source/owner identity,
answer core workspace queries at their declared completeness level, refresh
stale projections, report ambiguity and missing evidence honestly, and deliver
consistent current results through MCP. Optional architecture/data-flow/service
intelligence can be unavailable. Core absence, uniqueness, identity, classification
and source freshness cannot depend on CBM installation.

Three findings govern the recommendation:

1. Current CBM is more than name/path search: it implements typed cross-file
   resolution, explicit index jobs, coverage diagnostics and snapshot-bound
   query continuation. This warrants optional enrichment evaluation.
2. Those features establish properties of CBM's accepted/indexed graph, not
   Clean-CTX's complete authorized source universe or required semantic
   projection. No core responsibility can be retired on this evidence.
3. Current Clean-CTX's client expects older response shapes and omits explicit
   JSON format and continuation handling. Updating the provider executable alone
   is insufficient; this is a concrete compatibility investigation target.

## 2. Old branch architecture reconstruction

**HISTORICAL BRANCH DECISION**, supported by old
`docs/CBM_MIGRATION_DECISION.md`, `docs/CBM_INTEGRATION_BLUEPRINT.md`,
`docs/plans/CBM_INTEGRATION_PLAN.md`, and the following implementation sequence:

| Stage / commit | Actual mechanism | Authority and failure behavior |
| --- | --- | --- |
| Transport migration, `029ec9b` | Move from older 0.8.1 integration to 0.10.x daemon frontend/session model; private cache cohort and compatibility admission | Clean-CTX owns its session, not the shared daemon; launch/admission failure must leave native compilation usable |
| Blueprint, `a296a4d`; integration, `54d6b4b` | Bridge importance consultation in context strategy; derived fidelity and symbol skip-set | Explicit fidelity or intent blocks consultation; compiler receives derived selection parameters, not graph facts |
| Coverage lifecycle, `5ed9522` | Expose provider coverage/status information | Diagnostics about provider evidence, not replacement canonical IR |
| Transitional state removal, `3f09c79` | Remove CbmFilterState; use request-local CbmIntelligence and renamed importance API | No persistent CBM intelligence in WorkspaceIndex or CompiledIR |
| D0, `74f75d3` | Runtime health recovery | Provider failure/degradation is recoverable; core path survives |
| D1, `211246c` | Trace data-flow from deterministic importance-selected seeds; merge and bound related files/symbols | Advisory request-local context; failed seeds omitted; retain relevant in-file symbols through skip-set |
| D2-Wire, `b3c8363` | Explicit JSON trace/search compatibility and table normalization | Transport contract repair, not semantic authority transfer |
| D2, `9eab40d` | Cross-service traces with similar request-local bounded union | Reachability hints, not authoritative service identity |
| D4, `c899507` | Expose collected D1/D2 intelligence as `_meta.cbm` on seven response paths | Omit absent capabilities; no additional consultation; content/structuredContent and semantic ownership remain separate |

No D3 completion is established by the inspected sequence. Phase labels are not
proof that intervening phases shipped.

**PROVEN BY CLEAN-CTX CODE (historical):**
`src/mcp/heuristics.rs::compute_strategy` requires no explicit fidelity, no
explicit intent, an available bridge and enabled intelligence. Importance
failure falls back to the ordinary strategy. In
`src/intelligence/fidelity.rs`, CbmIntelligence contains importance, skip-set,
optional data-flow and optional cross-service contexts. The compiler consumes
only selection parameters; it never receives the advisory type. Fidelity
recommendations use importance thresholds above 0.8 / below 0.4, with native
heuristics where no definitive signal exists.

Historical consultation defaults in `src/config.rs`: data-flow 3 seeds, depth
2, at most 12 symbols/6 files; cross-service 3 seeds, depth 2, at most 20
symbols/8 files. Bounds apply after merging and deterministic selection.
These are historical settings, not a recommendation to impose new global limits.

**PROVEN BY CLEAN-CTX TEST (historical):**
`src/tests/mcp/d4_advisory_meta.rs` contains ten tests for data-flow/service
shape, combined capabilities, omission, explicit-policy exclusion, ordering,
bounds, unchanged authoritative content/envelope, response branches and
wire/internal distinction. `src/tests/mcp/heuristics.rs` checks explicit
fidelity/intent precedence; `src/tests/cbm/trace_wire.rs` checks normalization.
These tests support their asserted boundaries; they do not prove a live modern
CBM build interoperates or that model hosts consume the metadata.

## 3. Old CBM baseline and intended division

The old migration record targets 0.10.8, discusses inspection of 0.10.7 and a
small persisted-coverage correction into 0.10.8. It does not pin an exact CBM
source SHA. A matching 0.10.8 tag was not found in the inspected refs.
Packaging revision `5821078e` is therefore the reproducible comparison anchor.

| Old decision | Old capability | Clean-CTX responsibility | CBM responsibility | No-CBM fallback |
| --- | --- | --- | --- | --- |
| Graph retrieval is optional | Search, query, architecture and trace tools | Source acquisition, compilation and identity | Ranked candidates, topology and related symbols | Native filesystem discovery and compilation |
| Intelligence guides selection | Importance and graph traces | Explicit intent/fidelity, compression contract | Advisory ranking and expansion | Ordinary heuristics; no `_meta.cbm` |
| Daemon lifecycle is provider-owned | Shared persistent indexing daemon | Own session, cache cohort and launch diagnostics | Worker, watcher and graph persistence | No provider process required |
| Provider evidence is not compilation truth | Graph edges and positions, including Hybrid LSP | Canonical semantic evidence and framework interpretation | Optional navigation/resolution hints | Native producers and truthful unresolved states |

Hybrid LSP was **already present** at the comparison anchor: historical
`src/pipeline/pass_lsp_cross.h` includes TypeScript and C# cross-file processing.
Calling Hybrid LSP an entirely new 0.11 capability would be incorrect.

## 4. Current CBM capability audit

For each row, CODE means **PROVEN BY CBM CODE** and TEST means **PROVEN BY CBM
TEST**, with the no-run qualification above. DOCUMENTED means **DOCUMENTED BY
CBM**. Limits remain **NOT ESTABLISHED** unless narrower code/test evidence
explicitly establishes them.

| Capability | Current code / test evidence | Established behavior and limits |
| --- | --- | --- |
| Repository indexing | CODE: `src/mcp/mcp.c::handle_index_repository`, pipeline; TEST: `tests/test_index_resilience.c` | Full/moderate add semantics; fast omits them. Success concerns accepted input/mode, not all Clean-CTX source requirements |
| Incremental indexing | CODE: file-hash store, pipeline LSP surface and incremental passes; TEST: `tests/test_incremental.c` | Hash/interface-sensitive updates and dependent resolution; equivalence for all Clean-CTX framework projections NOT ESTABLISHED |
| Watching | CODE: `src/watcher/watcher.c`; TEST: `tests/test_watcher.c` | Daemon-owned Git/worktree polling, non-Git opt-in, retries and missing-project handling; eventual updates, not a request-time source snapshot |
| Freshness | CODE: `coverage_path_freshness`, `src/store/store.h`; TEST: MCP freshness tests #1714 and metadata-changed case | Stored hashes exist internally, but the inspected exposed path freshness uses mtime_ns + size. Metadata match is not exact current content-hash proof |
| Readiness/completion | CODE: index status provider/executor and index_repository status; TEST: `tests/test_index_supervisor.c` | Queued/running/succeeded/failed/cancelled jobs; serving graph availability and last job status are distinct; in-process server without daemon provider reports unavailable |
| Coverage/failure | CODE: check_index_coverage/index_status, coverage store; TEST: `tests/test_index_resilience.c`, `tests/test_mcp.c` | Parse partial/unusable, ignored/oversized/read/extract failures, recording truncation, hash-record completeness and generation mismatch; clean diagnostics do not prove complete language semantics |
| Resource failure/stale graph | CODE: index policy and staged publication; DOCUMENTED: `docs/INDEX_RESOURCE_LIMITS.md`; TEST: `tests/test_index_policy.c` | Optional limits default disabled; explicit overbudget failure preserves serving index. A served prior graph is not the failed job's fresh graph |
| Root/project identity | CODE: canonical repo path, root policy, named-project status resolution in MCP handler | Explicit root boundary and ownership/admission checks; project slug or qualified name alone is not Clean-CTX physical source identity |
| Multi-project indexing | CODE: project stores and cross-repo pass | Separate indexed projects and selected targets; no atomic whole-Clean-CTX-root snapshot established |
| File discovery | CODE: accepted-file pipeline, coverage path/scope lookup | Knows accepted/indexed files and exclusions; graph query cannot discover files its indexing policy excluded |
| Symbol discovery/exact lookup | CODE: search_graph filters/BM25 and query_graph; TEST: grouped QN, exact-name ranking and filter tests | Name/QN/file filters useful; exact match among stored nodes is not source-universe uniqueness; search ranking/window can omit candidates |
| Inbound/outbound references/calls | CODE: pass_calls, trace_path, CALLS/USAGE/CALL_REFERENCE graph relations | Structural and typed relationships with strategy/confidence evidence; occurrence inventory and compiler-exact binding are different claims |
| Inheritance/interfaces/dependencies | CODE: base/import extraction and cross-file resolution; TEST: C#/TS LSP suites | Typed/name relationships across indexed definitions; Angular template/DI semantics not thereby established |
| Cross-file resolution | CODE: `src/pipeline/pass_lsp_cross.c`, `internal/cbm/lsp/*`; TEST: C#/TS cross-file receiver and ownership regressions | Real production cross-file semantic pass; subset typed resolution and heuristics, not Roslyn/tsserver semantic equivalence |
| Cross-repository/service | CODE: `src/pipeline/pass_cross_repo.*` | HTTP, async/channel, gRPC, GraphQL and tRPC linkage; run/target failure and skipped-store disclosure; multi-store commits are not an atomic source snapshot |
| Ambiguity | CODE: `src/pipeline/lsp_resolve.h`, pass_calls candidate properties | Some ambiguous matches rejected or candidate state retained; registry/text fallback still possible. Global no-false-unique guarantee NOT ESTABLISHED |
| Ranking/determinism | CODE: BM25 and stable resolution tie-breaks; TEST: exact-name ranking / semantic pagination | Reproducible selection improves stability; deterministic heuristic choice does not prove correct or unique semantic target |
| Hybrid LSP | CODE: TS/C# in-process contexts/registries and pass_lsp_cross dispatch; TEST: `tests/test_cs_lsp.c`, `tests/test_ts_lsp.c` | Type-aware inference without requiring external language servers. Language grammar support is broader than typed resolver support |
| Positions/provenance | CODE: graph node file/line properties, relation strategy/confidence, LSP surface codec | Useful site/target coordinates and resolver provenance; source-id/SHA/site binding suitable for Clean-CTX private evidence NOT ESTABLISHED |
| Query completeness/paging | CODE: query_graph cursor context and totals; TEST: `tool_query_graph_cursor_is_lossless_and_snapshot_bound` | Snapshot-bound query/result/store-generation continuation, exact vs lower-bound totals and truncation; complete graph result is not complete source semantics |
| Partial-result reporting | CODE: search/query/trace returned totals, budgets and continuation; TEST: MCP budget, saturation and paging cases | Real explicit limits; adapter must preserve them. Some saturated searches cannot offer usable continuation |
| Persistence/update/delete | CODE: store APIs, project deletion, hash/coverage replacement and watcher | Persistent provider graph with mutation generation; deletion/update responsibilities remain provider-local, not Clean-CTX publication authority |
| Batch/query surfaces | CODE: single query_graph and paged tool handlers | Arbitrary supported graph projection and repeated calls; atomic multi-requirement Clean-CTX acquisition API NOT ESTABLISHED |
| Semantic/document links | CODE: semantic search and doc-link indexing; TEST: `tests/test_doclinks.c`, semantic MCP tests | Optional retrieval and mentions links; documentation mention is not executable call or Angular/C# semantic evidence |

The registry in `src/mcp/mcp.c` declares tools and textual MCP results; no
assumption is made that every tool supplies a typed structuredContent schema.
Tool declaration, production dispatch, internal tests and live host exposure are
separate evidence levels.

## 5. Material CBM evolution and measured delta

**PROVEN BY CBM CODE/history:** current `server.json` advertises 0.11.0 versus
0.10.8 at `5821078e`; language description increases 161 to 162. These are
packaging claims, not a deployed executable fingerprint or typed-language count.
No intervening release count is asserted without an exact old source pin and
release mapping.

Measured reachable commit delta: `git rev-list --count 5821078e..origin/main`
returns **1,077**. From the persisted-coverage reference `afebf009` it returns
**1,224**. Counts include merges, not unique architectural features. The scoped
diff over `src/mcp`, `src/pipeline`, `internal/cbm/lsp`, `src/daemon`,
`src/watcher`, `src/store` spans **154 files, 82,833 insertions and 11,697
deletions**. This is scoped implementation churn, not a benchmark.

Registry comparison: **15 → 17 tools**; additions `get_file_outline` and
`compare_graphs`; no removals in those two registry snapshots. Existing names
conceal substantial schema/default changes and must not imply compatibility.

| Material change | History evidence | Integration consequence |
| --- | --- | --- |
| Async jobs and polling | `28b41c35` (#2144), supervisor tests | Separate process availability, job terminal state and serving graph freshness |
| Watch diagnostics | `37940854` (#2167) | Better observability, not automatic semantic readiness |
| Daemon identity conflict | `19d14447` (#2277) | Version/build/cohort admission remains important |
| Resource failure / worker diagnostics | `699be6c2`, `7acb5eac` | Failed refresh may preserve older serving evidence; failure must survive adapter |
| Resolution/findability quality | `1a342553`, `51364158` | Improved deterministic resolution and C# tie handling; does not prove compiler authority |
| Cross-repo run states | `e4f9735d` (#2133) | Partial targets / skipped legacy stores must not become complete global coverage |
| Memory spill / determinism | `7b77fd48`, `9a2bdc8c` | Potential resource/performance improvement; same-graph intent requires runtime validation |
| Document mentions/retrieval | `ea9fe36b`, `842535e7` | Additional optional knowledge, not source-derived core semantics |
| Paging and coverage surfaces | Current MCP implementation and tests | Preserve generation, totals, truncation and continuation rather than flatten to rows |

Exact throughput, memory savings, Windows performance and improvement in #142
are **NOT ESTABLISHED** by source/history inspection.

## 6. Current Clean-CTX capability audit

**PROVEN BY CLEAN-CTX CODE:** current canonical compilation, source-hash checked
semantic publication, WorkspaceIndex ownership, C# query-time classification,
Angular meta producers and the private invocation boundary are independent of
CBM. Production hydration consumes provider file paths, not CBM semantic edges,
and then runs authoritative Clean-CTX compilation/publication.

Evidence paths:
`src/mcp/tool_handlers/hydration.rs`, its `publication.rs`, `cache.rs`,
`report.rs`, `filesystem.rs` modules; `src/mcp/tool_handlers/query/*`, including
`classification.rs`, batch and call preparation; `src/ir/invocation_evidence.rs`;
`src/cbm/client.rs`, `bridge.rs`, `project_search.rs`.

**PROVEN BY CLEAN-CTX TEST:** current
`src/tests/mcp/workspace_query_hydration_fidelity.rs`,
`workspace_query_batch_preparation.rs`, `hydration_filesystem.rs`, CBM
`project_search.rs`, `cache_ownership.rs`, regression freshness/identity tests,
and invocation evidence/lifecycle/generic-consumer tests assert narrow contracts.
Prior operator evidence: the invocation harness recorded 59/59 live MCP cases
before this investigation; see `verification/invocation-evidence/COVERAGE.md`.
That evidence does not establish modern CBM compatibility or #142 permutations.

Current heuristics do not consume the historical CbmIntelligence flow;
`cbm_informed` is false. A test path named `inference_layer_cbm` is not proof
that a corresponding production semantic inference layer exists.

**PROVEN BY CLEAN-CTX CODE: compatibility gap.**
`src/cbm/client.rs::build_search_graph_args` omits format; `search_graph`
expects `results` and treats a missing array as empty. `trace_path` expects an
older callers/callees shape without explicit format. `query_graph` omits
format and retains only columns/rows through QueryRows. Current CBM defaults
to tree and JSON search uses a grouped representation. A tree response can fail
inner JSON parsing; an unexpected JSON shape can flatten into an empty set.
Neither is legitimate proof of absent symbols. This is a static mismatch,
not a claim that a particular user's installed binary failed live.

Current `src/cbm/bridge/indexing.rs` also tracks its own per-project
NotStarted/InProgress/Complete/Failed state, rather than polling the new daemon
job surface. InProgress beyond its existing 60-second threshold fails locally;
untracked projects can pass through as Ready for provider-side validation.
Startup/lazy refresh requests use fast mode. These local signals are not proof
of current daemon job success, complete semantic coverage or native-source
adequacy; fast mode is specifically insufficient for promising full typed
resolution enrichment. Existing thresholds are observations, not new policy.

## 7. Independent Clean-CTX developments since divergence

**PROVEN BY CLEAN-CTX CODE/history:** current branch contains later project
cache ownership (`2865a91`), staged batch preparation (`f817322`, `08df92b`,
`8f67dd7`, `eb4089f`), Angular High semantic acquisition (`f651fc1`), exact
coverage (`715902c`), physical hard-link ownership (`0126d4f`), failed-hydration
retry/C# constructor coverage (`fe7a47b`), stale/full authority refresh
(`93058c5`, `d3b85b6`), neutral C# bases (`af22d16`), qualified generic
resolution (`d83570b`), Angular external metadata/template occurrence/linkage
(`babdd03`, `c08ab31`, `3e51a93`) and the frozen private invocation/Rx boundary.

These are independent product responsibilities, not missing pieces to overwrite
with old integration code. Current lifecycle tests and harnesses must be
preserved during any selected port. Commit ancestry alone does not prove why a
responsibility was originally added; causal explanations below are architectural
requirements/inferences, not invented historical motives.

## 8. C1–C6 ownership matrix

Each row has exactly one primary class. Old/current CBM columns describe the
comparison anchor/current source, not an installed binary. C3 is deliberately
unassigned: this audit does not earn direct authoritative provider consumption.

| Capability | Clean-CTX current | CBM old | CBM current | Core requirement? | Preferred authority / class |
| --- | --- | --- | --- | --- | --- |
| Canonical IR and wire | Native producer/serializer | Graph, different model | Graph, different model | Yes | Clean-CTX **C1** |
| Compression and requested fidelity | Native strategy/compiler | Importance hints | Ranking/snippets/outline | Yes | Clean-CTX **C1** |
| Model-facing core context contract | MCP content/envelope | Provider text | Provider text/limits | Yes | Clean-CTX **C1** |
| Current-source/hash publication | Native source identity and tickets | Provider hashes/coverage | Internal hashes + exposed metadata freshness | Yes | Clean-CTX **C1** |
| Authorized scope / physical owner identity | Native roots/path owner | Provider project/QN | Improved project/root controls | Yes | Clean-CTX **C1** |
| Candidate file/declaration discovery | CBM hints or filesystem | Search/query | Better filters/paging/coverage | Yes for complete core queries | Native verified acquisition, CBM hints **C2** |
| Exact symbol acquisition | Native parse/identity | Graph name/QN | Graph filters and typed lookup | Yes | Clean-CTX verifies provider hints **C2** |
| Inbound/outbound core relationship discovery | Native projections + acquisition | Edges / traces | Typed edges and trace evidence | Yes within declared semantics | Clean-CTX producers, accelerated paths **C2** |
| Candidate completeness / absence / uniqueness | Known static gaps | Indexed candidates | Graph totals/coverage limits | Yes where claimed | Clean-CTX authorized-universe proof **C1** |
| Source generation vs provider index generation | Native hashes, provider cache generations | Provider index state | Snapshot/mutation generation | Yes | Distinct identities; core source owner **C1** |
| Core query batching/snapshot | Prepare/evaluate with catalog gap | Individual calls | Individual/cursor calls | Yes | Clean-CTX acquisition/refinement **C1** |
| Indexed-only graph query contract | Native lower-bound APIs | Graph query | Exact/lower-bound indexed totals | Yes when offered | Clean-CTX explicit partial semantics **C1** |
| C# classification/generic base refinement | Native structural/domain resolver | LSP/name relations | Stronger C# typed inference | Yes | Different evidence contract **C6** |
| Angular DI/template/selector semantics | Native framework layers | Generic TS graph | Typed TS plus generic links | Yes | Clean-CTX framework specialization **C6** |
| Rx pipeline written invocation evidence | Native parser-bound evidence | Graph calls | More typed calls/references | Yes | Clean-CTX written structural semantics **C6** |
| Invocation occurrence/source-site identity | Private source-id/SHA view | CALLS | CALLS + strategy | Yes | Clean-CTX **C1** |
| Optional precise call-target enrichment | Structural unresolved/native states | Hybrid LSP | Stronger cross-file typed results | No new promise yet | Request-local optional enrichment **C4** |
| Related-file / importance advice | Core can operate without | Advisory intelligence | Ranked graph retrieval | No | Explicit optional advice **C4** |
| Data-flow and service intelligence | No equivalent core promise | Trace modes | Broader cross-service evidence | No | Optional provider output **C4** |
| Multi-repo architecture / graph comparison | Core multiroot source scope only | Architecture/traces | Cross-repo states/compare_graphs | No global graph promise | Optional output **C4** |
| Semantic/document retrieval | Source context independent | Optional retrieval | Broader semantic/doc links | No | Optional retrieval **C4** |
| Provider daemon/worker/watcher | Own session adapter | Provider daemon | Async supervisor/watcher | No | CBM lifecycle **C5** |
| Provider graph persistence/spill/delete | Core owns own projections | Provider store | Generation/store/spill | No | CBM internals **C5** |
| Provider internal index policy/language registry | Native producer policy separate | Accepted graph sources | More limits/typed passes | No | CBM **C5** |

C3 can be reconsidered only for a precisely bounded fact whose provider origin,
source generation, source/site/target identities, resolver semantics, completeness
and disagreement behavior are demonstrably compatible. A confidence value or
stable QN alone does not meet that bar.

## 9. Responsibilities retained and takeover reassessment

| Responsibility / why native evidence is necessary | Missing provider information | Current CBM closes gap? | Disposition |
| --- | --- | --- | --- |
| Canonical/fidelity/context ownership | Clean-CTX IR, requested output semantics | No; snippets/outline differ | **KEEP CLEAN-CTX** |
| Exact source refresh and publication | Current content identity + required producer/fidelity | Internal hash and coverage stronger; exposed check is metadata-based and not native projection adequacy | **KEEP BOTH**; use diagnostics only |
| Candidate discovery for core questions | Complete authorized roots, exclusions and relevant declarations | Paging improves indexed enumeration; source-universe completeness not proven | **USE CBM AS ACCELERATOR**, native fallback remains |
| C# classification and cross-file identity | Native declaration identity, generic/domain classification | Typed resolution improves candidates; different evidence model and coverage | **KEEP BOTH**, native authority |
| Angular framework/meta layers | DI, selector/template occurrence and linkage semantics | Generic typed TS does not establish these contracts | **KEEP CLEAN-CTX** |
| Private invocation occurrences | Written call span/owner/arguments bound to source SHA | CALLS target edges are a different projection | **KEEP CLEAN-CTX**; optional later target hints |
| Batch correctness and completion reporting | Union requirements and consistent final catalogs | Provider paging/job state cannot schedule native catalog construction | **KEEP CLEAN-CTX** |
| Optional architecture/service/document breadth | No native core promise | Real provider capabilities | **USE CBM AS OPTIONAL EVIDENCE** for explicitly optional output |

No core duplication is eligible for retirement. The evidence establishes why it
must remain; it does not establish that every historical addition was motivated
solely by an older CBM deficiency.

## 10. Discovery-oracle evaluation

**PROPOSAL:** CBM can tell Clean-CTX where to look without deciding what native
evidence means: admitted provider project → candidate paths → root/owner check →
current source read/hash → required native producer/fidelity → checked publication
→ domain refinement. This matches the existing path-hint boundary.

It is sufficient to accelerate positive acquisition. It is insufficient alone
for authoritative absence or uniqueness: search windows, excluded sources,
unsupported semantics, partial parse, stale serving graph and incomplete roots
can omit alternatives. A complete graph query is complete only for its graph
and selected projection. Clean-CTX needs native scope discovery or an independently
validated coverage certificate sufficient for the exact requirement. No such
universal certificate is established here.

## 11. #142 reevaluation

This revisits the concrete findings in
[the prior investigation](QUERY_HISTORY_INDEPENDENCE_AND_DISCOVERY_COMPLETENESS_INVESTIGATION.md).

| Prior finding | What current CBM changes | Required Clean-CTX action / remaining evidence |
| --- | --- | --- |
| Successful search with still_indexing/failed can make aggregate discovery completed | Terminal job states, watch diagnostics and coverage are stronger | Preserve distinct readiness/serving/query completion; fix native aggregate truth after targeted reproduction. Provider upgrade cannot change HydrationReport::is_complete |
| Already-indexed LegacyEdit files skipped without uniform fidelity/hash proof | Provider freshness/coverage can identify suspect paths | Native producer/hash/fidelity adequacy still required even if provider reports clean |
| C# forward candidate/base refinement | Typed imports/receivers/base relations improve hints | Existing staged native repair remains; complete matching declarations/generic identity still need native verification |
| Reverse discovery differs between inbound provider path search and literal filesystem search | Query projection/paging can enumerate more indexed callers/targets | Enumerate target declarations as required, preserve ambiguity and prove scope completeness; missing target acquisition is not solved by caller paths |
| Angular selector/component and service candidate resolution | Better TS symbol graph | Framework/template/DI semantics not established by generic graph; native High coverage and complete selector acquisition remain |
| Cross-file/cross-repo identity | Provider projects/QNs/typed registry improve lookup | Match admitted physical source identity; duplicates, aliases and same names across roots require native ownership |
| Batch catalogs built during sequential preparation | No atomic native union acquisition surface found | Build/refine against an adequate final native snapshot after acquisition; provider cannot fix earlier captured catalogs |
| Transitive query acquires initial name only | Graph traversal can suggest further frontier | Native complete transitive contract needs adequate frontier acquisition; indexed-only contract must remain explicit |
| has_cycle is intentionally indexed-only | Provider graph also has bounded/snapshot semantics | Preserve witness/lower-bound distinction; no absence-of-cycle claim over unacquired source |
| Discovery cache adequacy vs aggregate completion differ | Stronger job/generation signals available | Cache key and completion proof must match actual provider/root/name/mode/fidelity/source requirements, not merely successful RPC |
| Freshness and retries after edits/failure | Watcher and retained graph can help | Preserve native invalidation/retry; a failed refresh's old serving graph cannot masquerade as new completion |

**INFERENCE:** current CBM can improve optional acquisition observability and
perhaps reduce fallback scanning. It does not remove the native completion,
projection-adequacy or batch-snapshot issues. **NOT ESTABLISHED:** which failures
occur with the pinned current CBM deployed, their frequency, or the smallest
common implementation correction. Thus the selected #142 verdict remains
MORE LIVE EVIDENCE REQUIRED rather than claiming a runtime resolution.

## 12. Native no-CBM fallback matrix

| Operation | With CBM | Without CBM | Same semantic truth required? |
| --- | --- | --- | --- |
| Core context and requested fidelity | Native compile, optional selection hints | Native compile/ordinary strategy | Yes for explicit core contract; optional advice may differ |
| Current source/semantic refresh | Provider hints plus native source/hash validation | Native source/hash validation | Yes |
| Core entity and relationship acquisition | Paged hints verified by native compilation; fallback if inadequate | Authorized filesystem/native discovery | Yes for authoritative completed answers |
| C# base/interface classification | Optional typed candidates + native refinement | Native candidates/refinement | Yes |
| Angular DI/template/selector/Rx queries | Native domain evidence with optional file hints | Native domain evidence | Yes |
| Written calls_in_file | Native private occurrence compilation | Same native compilation | Yes |
| Batch core query | Native union adequacy/final snapshot | Same native adequacy/snapshot | Yes |
| Indexed-only graph APIs | Native indexed lower-bound answer | Native indexed lower-bound answer | Same truthful contract; published lower-bound contents can differ |
| Optional resolved-target enrichment | Explicit provider result/limits, never overwrite occurrence | Explicit unavailable/unenhanced state | No for optional added binding; core written evidence unchanged |
| Optional graph search/architecture/data-flow/services | Provider capability with disclosed coverage/failure | Explicit unavailable | No; not a core completeness claim |
| Document/semantic retrieval and graph comparison | Optional provider result | Explicit unavailable | No |

Absent, disabled, unhealthy, still-indexing and stale providers must all preserve
core correctness. The current filesystem literal discovery and LegacyEdit
adequacy paths have identified gaps; calling them a fallback does not prove
they already satisfy every stronger complete-query promise. Record and fix those
native gaps rather than declaring CBM mandatory.

## 13. Provider disagreement policy

**PROPOSAL; not an implemented global policy:** compare source generation,
identity and evidence sufficiency before comparing conclusions.

- Native evidence is authoritative only when its own current-source, scope,
  producer and fidelity requirements are satisfied. Native stale evidence must
  be reacquired rather than winning merely because it is native.
- CBM hints are advisory. Conflicting hints cannot mutate native identity,
  canonical IR or invocation records. Reacquire relevant current source.
- Stale/incompatible provider generation or unbound site identity makes direct
  evidence unusable; it can at most guide an authorized source read.
- Partial/ambiguous evidence cannot establish unique resolution or absence.
- For independently authoritative evidence, if eventually earned, matching
  provenance but conflicting conclusions requires explicit unresolved/conflicting
  output or omission of optional enrichment—not silent provider preference.

Existing request-local records, native publication checks and provider result
metadata are sufficient starting points. Do not invent a generalized evidence
framework. Preserve provider diagnostics at the adapter boundary and native
coverage at the query boundary. Any new public conflict state needs explicit
contract approval before implementation.

## 14. Hybrid LSP evaluation

**PROVEN BY CBM CODE:** this is an in-process tree-sitter/type-inference and
cross-file registry implementation, not proof of external Roslyn/tsserver
analysis. `internal/cbm/lsp/cs_lsp.h`, `ts_lsp.h`,
`src/pipeline/pass_lsp_cross.c`, `lsp_resolve.h` and `pass_calls.c` form a real
production route. Typed resolution matches call records with a confidence floor
of 0.6 and may use registry/textual fallback. Ambiguous caller tails can be
rejected; deterministic tie-breaking is not compiler-exact binding.

Current CBM internally does more than line/name matching:
`lsp_resolve.h::cbm_pipeline_invocation_site_rank` compares non-empty byte spans
and source_origin for occurrence-exact joins, rejects mismatches, and ranks
exact sites above legacy rows. Exact-caller competing targets can return NULL.
This strengthens potential optional composition. It does **not** establish that
the exposed MCP result carries those fields together with a matching source SHA,
or that all legacy/fallback resolution is occurrence-exact. Do not confuse
internal site-aware joining with an already compatible Clean-CTX evidence wire.

| Desired optional evidence | Code/test support | Remaining authority limit |
| --- | --- | --- |
| C# receiver/type | var/new, typed parameters, this/base, casts/patterns, await, generics and extension dispatch; `tests/test_cs_lsp.c` | No Roslyn analyzers/source generators/reflection/dynamic equivalence |
| TS receiver/type | Imports, typed scopes, generics, await/return chains and cross-file registries; `tests/test_ts_lsp.c` | Dialect/tsconfig and unknown JS evidence affect coverage |
| Exact target | Resolved target with strategy/confidence/candidates | Site/source generation binding and complete alternative set not established |
| Inheritance/interfaces/imports | Extracted definitions/base/import registries; C# interface and TS implements tests | Framework DI/template semantics remain native |
| References | USAGE/CALL_REFERENCE and graph traces | Reference graph is not complete written invocation inventory |
| Overloads | Typed/generic resolution handles some cases | Full language/compiler overload equivalence NOT ESTABLISHED |
| Cross-file identity | File-qualified registries and ownership regressions | Must map to native physical source/owner identity; QN alone insufficient |

Optional absence must be explicit. Native structural evidence and unresolved
states remain useful without LSP-backed enrichment. Before direct consumption,
prove same source generation, source span/caller/target identity, supported
resolver strategy and truthful ambiguity. Current graph mutation generation does
not itself provide the content-SHA binding needed for that proof.

## 15. Frozen invocation-boundary interaction

Parser-proven occurrence remains Clean-CTX structural authority. Its private
source-id/SHA-bound compilation-local lifecycle must not change. CBM CALLS
captures relationships/resolution, not an interchangeable occurrence record.
Do not persist private records or export them to align graph IDs.

**PROPOSAL:** after occurrence production, optionally request target candidates
for a precisely matched source site; validate current source/identity; compose
request-local optional resolution. If provenance or compatibility fails, preserve
written occurrence and report no enrichment. This is useful for typed receiver
navigation but is not yet production-proven or authorized for implementation.

## 16. Batch and discovery implications

Current CBM can support repeated/project-scoped queries and snapshot-bound pages.
It does not own native semantic acquisition requirements. A batch may deduplicate
the union of candidate requirements before native publication, then construct
all domain catalogs from the adequate final snapshot. That is a **PROPOSAL**
requiring approval if it changes shared completeness/public behavior.

Preserve preparation/evaluation separation already shipped. Paging one provider
query consistently does not make multiple independent queries a shared snapshot,
nor make sequentially captured C#/Angular catalogs fresh. Test item-order,
singular-vs-batch and preceding-warm-query equivalence separately.

## 17. Multi-repository implications

Clean-CTX authorized primary/additional roots and withinPath remain controlling.
Provider project identity cannot widen authorization. Canonical/physical paths,
hard links, same-named classes/interfaces, aliases and root collisions must be
verified natively. Cross-repo CBM links are useful optional breadth over indexed
projects, not proof of all authorized source repositories.

Current cross-repo execution discloses partial/skipped targets; committed target
stores are not globally rolled back on cancellation. Treat that as optional
partial evidence. A global coherent generation across all stores is
**NOT ESTABLISHED**. Do not conflate service reachability with service identity.

## 18. Performance implications

**INFERENCE:** provider filters, persistent indexes, typed registries, watcher
updates and spill improvements may reduce repeated scans and resource costs.
Native verified acquisition and request/session deduplication remain necessary.
A healthy provider can change execution cost, not authoritative completed truth.

No speed/memory/throughput measurement was run. Cold startup, MSVC/antivirus,
non-Git watching, large-root discovery, paging cost and failure recovery require
operator evidence. Do not introduce timeouts/budgets globally to make one pilot
pass; reuse existing approved limits and harness patterns.

## 19. Old branch disposition

| Historical change | Classification | Recommended treatment |
| --- | --- | --- |
| Optional/advisory authority split | STILL CORRECT | Retain principle, reconcile current native lifecycle |
| Transport/cohort/build conflict handling | CORRECT BUT NEEDS PORTING | Mine admission/health tests against current daemon |
| Explicit JSON search/trace decoding | CORRECT BUT NEEDS PORTING | Port concepts after schema capture; add current totals/cursors |
| Old flattened query rows | SHOULD BE REDESIGNED | Preserve full continuation/completeness metadata at adapter boundary |
| D0 health recovery | CORRECT BUT NEEDS PORTING | Preserve failure fallback; validate current lifecycle |
| D1/D2 optional context | CORRECT BUT NEEDS PORTING | Only as bounded optional advice; do not replace native semantics |
| D4 `_meta.cbm` | CORRECT BUT NEEDS PORTING | Mine omission/bounds/authority tests; reconcile current batch/envelope surfaces |
| Old transitional CbmFilterState | NO LONGER NEEDED | Do not restore removed state |
| Old publication/query integration assumptions | SUPERSEDED BY CURRENT CLEAN-CTX | Preserve current hash/physical owner/fidelity/batch machinery |
| Assuming synchronous successful search proves readiness | ARCHITECTURALLY OBSOLETE | Current job/serving/coverage distinctions must survive |
| Treating fixed old tool shape as version compatibility | SUPERSEDED BY CURRENT CBM | Inspect negotiated schema/capabilities and fail unsupported use honestly |

**MINE SELECTIVELY**, rather than a wholesale rebase. Valuable concepts/tests
can inform fresh narrowly scoped ports. No old branch was changed in this audit.

## 20. Recommended current architecture

**PROPOSAL:** strengthen existing CbmClient/GraphBridge and discovery boundary:

```text
optional provider adapter
  -> admitted project + explicit wire format
  -> job / serving generation / coverage / paging diagnostics
  -> candidate paths and optional bounded intelligence
  -> native authorized source acquisition and adequacy checks
  -> native publication and domain interpretation
  -> truthful core MCP result (+ explicitly optional enrichment)
```

No provider semantic facts should enter canonical IR or WorkspaceIndex merely
because they are available. Native core paths remain executable without the
provider. Optional resolution is request-local and later than written evidence.
No new provider registry, second graph or persisted target-resolution fabric is
justified by this audit.

## 21. Explicit non-goals

No production implementation, dependency bump, old-branch merge/rebase, provider
schema redesign, generic evidence framework, new public completeness policy,
private invocation persistence, replacement of native framework producers,
local full test suite, or host pilot occurs in this task. No current CBM README
claim is used as proof of Clean-CTX correctness.

## 22. Migration implications

**PROPOSAL:** first capture modern wire/lifecycle behavior, then make isolated
adapter compatibility corrections with tracked regressions. Preserve native
fallback and existing lifecycle/owner tests. Only then consider selected old
advisory features. Keep #142 semantic/completion changes in a separately approved
phase with RED/GREEN evidence and live MCP permutations.

Avoid copying historical config and selection defaults blindly. Automatic
selection can alter compressed output; explicit intent/fidelity precedence and
core correctness require preservation tests. Porting D4 must account for current
batch content modes and structured envelopes, not just seven historical paths.

## 23. Compatibility and version negotiation

Current packaging versions are not executable identity. Capture actual binary
path/version/build, daemon cohort/cache ownership and advertised tools/schemas.
Use existing initialization/tools-list paths; do not add a provider framework.
Check required capabilities, explicit format support, job-status availability,
paging fields and semantic indexing mode. A name appearing in tools/list does
not prove the expected response schema.

Explicit JSON parsing must reject incompatible shapes, not turn them into empty
candidates. Preserve totals/relations/truncation/cursors; stale continuation must
restart or disclose failure rather than silently claim completion. Fast indexing
cannot promise the typed semantic evidence of full/moderate mode. Unsupported
optional capability means unavailable; unsupported required acceleration means
native fallback, never degraded core truth.

## 24. Tests and falsification before implementation

Use existing tracked tests under `src/tests/**` and existing PowerShell MCP
verification patterns under `verification/**`; generated evidence belongs under
target. Internal regressions and operator MCP evidence are distinct. These are
required future checks, **not runs or a blanket authorization request**.

| Gate | Falsifier and evidence to collect |
| --- | --- |
| Wire compatibility | Current binary tools/list and raw search/query/trace/status responses; explicit JSON and malformed/missing shape; no false empty result |
| Paging/completeness | More candidates than default/window/budget; exact vs lower-bound totals; continuation across mutation; lost/truncated pages cannot establish complete absence |
| Readiness | Successful search while queued/running/failed; prior serving graph after failed refresh; unknown job provider; no premature aggregate completion/cache hit |
| Freshness | Same-size/same-mtime content mutation; stale CBM + fresh native and converse; edit/delete/recreate; current native SHA/producer/fidelity remain controlling |
| Native adequacy | LegacyEdit after Low publication then High-needed query; failed projection retry; hash unchanged but missing required producer; prior indexed file must not bypass adequacy |
| Query history | Fresh server Q, warm unrelated Q, warm same Q, Q after edit, and permutations for C#/Angular/relationships; compare semantic facts/status/ambiguity, not timing |
| Batch | Item-order permutations, singular vs batch, union acquisition, earlier captured catalogs; same authorized source generation and adequate final native evidence |
| No-CBM matrix | Healthy/absent/disabled/unhealthy/indexing/stale providers; completed core truth converges or honestly remains incomplete |
| Identity | Duplicate names across files/roots, interface vs class, generic bases, alias/hard link, cross-repo target; no slug/QN substitution for physical identity |
| Hybrid resolution | Same source-site receiver, overloads, generics, unsupported/dynamic cases, resolver fallback/ambiguity and framework boundaries; optional enrichment cannot alter occurrence |
| Lifecycle | Worker failure/cancel, daemon admission conflict, session reuse, refresh/delete; record actual provider version and generation |
| Model boundary | Live MCP core queries across C#/Angular/TS/meta layers; validate fields, freshness and limits using existing harnesses; CLI stdio evidence and actual host consumption remain separate |

Reuse CBM's existing supervisor, MCP pagination/coverage, resilience and C#/TS
LSP tests as provider-side evidence. They do not substitute for Clean-CTX
adapter regressions or native fallback tests. Full repository tests remain CI-only.

## 25. Exact recommended next phase

**PROPOSAL: Current-provider compatibility and #142 acquisition falsification.**
Keep production redesign paused. Prepare a bounded extension of existing CBM/MCP
verification assets that records the pinned deployed binary, admission/schema,
explicit JSON shapes, job/serving state, coverage and complete paged candidate
collection. Pair it with focused injected-provider regressions for the native
readiness aggregate and projection-adequacy defects. Include no-CBM and
cross-file/cross-meta-layer query-history/batch cases.

Completion criteria: current wire compatibility is evidenced; terminal job state
is distinguished from fresh sufficient source evidence; candidate limits survive
to the query boundary; native fallback gaps are reproduced; the smallest #142
correction is supported by RED evidence and a concrete scoped plan. Obtain
specific authorization before policy-controlled runs. No new global authority,
completeness or public-result decision should be implemented under this audit.

The old branch remains historical evidence. CBM remains optional. All
**authoritative completed core answers** must converge on native current-source
truth whether the provider is healthy, absent, disabled, unhealthy, indexing or
stale. This requirement controls every future port and enrichment decision.
