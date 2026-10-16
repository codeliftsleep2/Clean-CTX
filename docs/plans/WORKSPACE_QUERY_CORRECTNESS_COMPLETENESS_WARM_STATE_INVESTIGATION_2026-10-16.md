# Workspace Query Correctness, Completeness, and Warm-State Authority Investigation — 2026-10-16

**Investigator:** Agent MaxHeadRoom  
**Status:** RELEASE-BLOCKING — Clean-owned provider authority repair focused GREEN confirmed; broader release gates remain

## EXECUTIVE VERDICT

**DO NOT SHIP.**

Two production defects are proven:

1. `HydrationRequirement::LegacyEdit` treats any `WorkspaceIndex::file_map()` entry as sufficient. It does not prove current source hash, adequate fidelity, valid dependencies, or satisfaction of the interpreting domain's evidence requirement.
2. A successful non-ready CBM search was treated as root coverage, suppressing authoritative Clean-owned fallback. Provider success could therefore influence truth despite not being cached as complete.

Neither is yet proven to have caused the live `ProjectContext` asymmetry. Exact structured identities, paths, candidate outcomes, projection hashes/fidelities, and provider readiness were not supplied.

Three field interpretations also exceed current contracts:

- `declarations_only` queries source-proven declarations, not the same identity set as unfiltered entity lookup. A C# class is normally `builtin/Class`; query-time EF classification does not rewrite it into a `dotnet/DbContext` declaration.
- Generic edge queries are lower-bound. Zero plus `source_complete: false` is not authoritative absence.
- `discovered_not_compiled_this_cycle` is discovered minus newly published. It includes already-current candidates and is not an omitted-file count. Thus 115/236 does not prove 121 missing compilations.

The live causal root cause remains **INSUFFICIENT EVIDENCE — TARGETED RUNTIME FALSIFICATION REQUIRED**.

## LIVE EVIDENCE

| Observation | Classification | Proven mechanism |
| --- | --- | --- |
| Five unfiltered entities versus zero with declaration/domain/type filters | **PROVEN QUERY-CONTRACT DIFFERENCE**; producer failure still possible | `declarations_only` changes the authority set before exact filtering. |
| Duplicate `ProjectContext` differs across roots | **INSUFFICIENT EVIDENCE — TARGETED RUNTIME FALSIFICATION REQUIRED** | Occurrences are file-preserving, but uniqueness gates and weak legacy acquisition can suppress derived interpretation. |
| Reverse zero and incomplete | **PROVEN EXPECTED DECLARED DEGRADATION** | Coverage explicitly declares lower-bound semantics and possible omissions. |
| Faster 115/236 cycle | **PROVEN QUERY-CONTRACT DIFFERENCE**; authority failure possible | `Current` candidates are not counted as compiled; selected/current/failed counts are absent. |

## PROVEN CALL PATH

```text
workspace_query
 -> handle_workspace_query
 -> operation.prepare
 -> PreparationContext::scope
 -> initial WorkspaceIndex read
 -> PreparationContext::hydrate
 -> hydrate_workspace_index_for
    -> discover_candidate_paths
    -> select_candidates
    -> compile_candidate_for
    -> aggregate PublishedCSharp
    -> reconcile_csharp_exact_calls once
    -> mark discovery completions
 -> construct query-local catalogs
 -> PreparedQuery::Indexed
 -> batch only: prepare indexed items again
 -> one WorkspaceIndex read guard
 -> evaluate/filter/render coverage
```

## COMPLETENESS STATE MODEL

**STATE:** completed provider discovery  
**OWNER:** session `HydrationDiscoveryCache`  
**KEY:** provider + canonical root + mode + name + optional fidelity  
**SCOPE:** one CBM project or filesystem root  
**GENERATION:** monotonic per provider/root  
**MUTATION:** after candidate processing and reconciliation  
**VALIDITY:** recorded generation equals current generation  
**INVALIDATION:** root edit/refresh  
**PARTIAL/FAILURE:** publication failure prevents marking; non-ready scopes are not recorded  
**REUSE:** a hit suppresses discovery and assumes prior evidence is sufficient

This proves one discovery operation completed at one scope generation. It does not prove current bytes, adequate projection, valid dependencies, domain evidence, or catalog uniqueness.

`HydrationReport::is_complete()` is the aggregate status check. A successful
non-ready CBM search previously entered `successful_roots`, avoided fallback,
and could yield a completed aggregate despite not being cached. The repaired
model separates provider usability from provider coverage: non-ready hints are
retained, but only completed Clean-owned acquisition and downstream processing
may establish authority.

`HydrationRequirement::Semantic` validates hash and fidelity. `LegacyEdit` checks only absence from `file_map()`. Population is substituted for adequate current evidence.

`CompletenessCertificate` is file-local. Generic graph coverage remains lower-bound except for explicitly upgraded domain cases.

## HYDRATION KEY MODEL

```text
HydrationKey {
  inbound: query_type == "reverse_edges",
  name,
  requirement: LegacyEdit | Semantic(fidelity)
}
```

It omits domain, entity type, and relation. Root/scope are shared across a valid batch. Domain/type omission is safe only when acquisition is genuinely domain-blind and sufficient. `LegacyEdit` does not express that proof.

The session cache adds provider, canonical root, mode, optional fidelity, and generation. A legacy entry still carries no domain evidence dependency.

## ROOT/SCOPE MODEL

`WorkspaceScope` admits `workspaceRoot` plus additional roots, optionally narrowed by `withinPath`. Results use physical occurrence provenance.

Hydration enumerates filesystem roots and CBM projects independently. Unregistered additional roots use filesystem fallback. Completion keys use canonical root identity, so one root does not directly satisfy another.

No root-key collision is proven. Plausible asymmetry mechanisms are non-ready CBM success preventing fallback, legacy population suppressing validation, catalogs seeing only indexed snapshots, and deliberate same-name ambiguity rejection.

## PROJECTCONTEXT IDENTITY TRACE

| Stage | Main root | Additional root |
| --- | --- | --- |
| Canonical path/root/hash | Runtime capture required | Runtime capture required |
| Declaration identity | Expected `builtin/Class/ProjectContext` | Expected `builtin/Class/ProjectContext` |
| Declaration occurrence | Preserved independently by file | Preserved independently by file |
| Base evidence | Per-file `HasBaseType -> TypeRef` | Per-file `HasBaseType -> TypeRef` |
| DbContext evidence | Must be independently earned | Must be independently earned |
| Discovered/selected/current | Runtime capture required | Runtime capture required |
| Compiled/published | Runtime capture required | Runtime capture required |
| Reconciled | Shared post-loop reconciliation | Shared post-loop reconciliation |
| Completeness | File presence is insufficient | File presence is insufficient |
| Unfiltered/filtered result | Exact identity/file required | Exact identity/file required |

Declaration storage preserves same-identity occurrences by distinct `EntityRef.file`. No name-based conflation is proven. Catalog helpers deliberately decline to choose ambiguous duplicate names, which preserves uncertainty but may suppress derived classification.

## DECLARATION FILTER TRACE

`prepare_find_entities` uses the same name-based legacy hydration regardless of `declarations_only`, domain, or entity type. The filters do not narrow acquisition.

```text
declarations_only = false
  -> find_entities_by_name[_in_scope]
  -> declarations plus indexed relationship endpoints

declarations_only = true
  -> declarations_by_name[_in_workspace]
  -> source-proven declaration occurrences
  -> exact domain and entity_type filters
```

Five unfiltered items and zero `dotnet/DbContext` declarations are compatible when those five are `builtin/Class`, `builtin/TypeRef`, or relationship endpoints. The live case needs their structured identities and files.

## REVERSE LOOKUP TRACE

Reverse lookup performs inbound-reference discovery, compiles selected candidates, then evaluates stored and refined edges. Generic exact-identity coverage declares `source_complete: false`, `result_semantics: lower_bound`, and `omitted_possible: true`.

The alternative `builtin/TypeRef + HasBaseType` query asks for a different source-true capability. It does not make the original identity complete. The reported zero means zero observed, not proven none.

## 115/236 COVERAGE TRACE

1. `candidates_discovered` is the discovered path-vector length.
2. Selection canonicalizes, validates, and deduplicates.
3. Legacy selection excludes files already in `file_map()`.
4. Semantic selection checks current hash/fidelity.
5. `candidates_compiled` increments only for `Published` and `PublishedCSharp`.
6. `Current` increments neither compiled nor failure.
7. Coverage computes discovered minus compiled.

Thus 115/236 proves 236 discovered paths and 115 new publications, not 121 omitted files. Selected/current/failed outcomes are required.

## QUERY-HISTORY ANALYSIS

Warm completion is safe only when prior candidates remain current and adequate. Legacy population-only selection does not prove that precondition.

Different names/directions have different cache keys, but unrelated publication expands catalogs and can change ambiguity. That is valid only after the authoritative query satisfies its full evidence requirement.

Fidelity-qualified semantic completion is isolated from legacy completion. Most entity/edge paths still request `LegacyEdit`, so a weaker populated projection can suppress reacquisition.

Occurrence storage is file-preserving. Tests must distinguish correct ambiguity from history-dependent missing evidence.

## SECOND-PASS ANALYSIS

Batch first preparation performs acquisition. Indexed items are prepared again using the same request-local reports so catalogs see the shared post-acquisition index. Evaluation occurs under one read guard.

This prevents early catalog construction but cannot repair an under-specified requirement or falsely completed aggregate. The second pass reuses rather than re-proves acquisition.

## RECENT RECONCILIATION-REPAIR ANALYSIS

**EXONERATED for the proven acquisition/currentness defects; no evidence ties it to the live asymmetry.**

Current ordering is candidate publications, one aggregated C# reconciliation, then completion marking. Reconciliation failure returns before marking. Sibling publication failure makes hydration partial. Batch second preparation occurs after reconciliation. Reverting to per-publication reconciliation would restore CPU amplification without fixing acquisition.

## ROOT CAUSE

**Proven architectural cause:** shared acquisition lacks a requirement expressing the evidence most semantic query domains need. `LegacyEdit` equates an index entry with no further work required. Provider readiness is also not fully reflected in aggregate completion.

**Live incident cause:** not proven. Structured identities and candidate lifecycle evidence are missing, while the declaration and 115/236 observations have contract explanations.

## CORRECTNESS INVARIANT

> For the same compatible source generation, authorized scope, and declared evidence requirement, cold, warm, and query-history variants must yield identical truth and completeness. Reuse may suppress only work whose source currentness, projection adequacy, dependency validity, and provider/root completion are proven by the reuse key.

A populated file map proves prior publication, not satisfaction of the current query's requirement.

## NARROW REPAIR BOUNDARY

After a causal RED:

- replace population-only satisfaction for the affected authoritative domain with current projection adequacy;
- require authoritative Clean-owned evidence before aggregate completion;
- treat CBM readiness as neither necessary nor sufficient once Clean evidence is established;
- preserve provider/root/name/direction/generation isolation;
- preserve provenance and honest lower-bound metadata;
- preserve one reconciliation per hydration loop;
- do not classify globally by name, force full-workspace compilation, special-case roots/types, or upgrade generic graph contracts.

Choosing existing `HydrationRequirement::Semantic` versus a richer domain requirement must follow the RED. A changed public contract requires architectural approval.

## STRUCTURAL RED DESIGN

1. Weak prior projection followed by authoritative classification versus a fresh cold state.
2. Two roots each declaring `ProjectContext : DbContext`, across cold, each-root-warm, and reversed order.
3. Declaration control requiring both `builtin/Class` occurrences; separately verify EF-owned derived classification.
4. Successful CBM search with `still_indexing` must not authorize complete refinement and must retry.
5. Incomplete reverse zero remains distinct from certified exact zero.
6. Test-only discovered, selected, current, published, and failed sets.
7. Compare Q, `[Q,A]`, `[A,Q]`, and partial-A-then-Q.

Instrumentation must remain test-only and observational.

## RUNTIME EVIDENCE REQUIRED

Capture canonical paths/root keys, source hashes, projection fidelity/dependencies, provider readiness/mode/name/generation, candidate outcome sets, completion entries, all five unfiltered identities/files, the filtered request/response, per-file `HasBaseType` and EF evidence, reconciliation order, and coverage inputs.

Complete deterministic local work first. Use one consolidated rebuilt-binary Claude pilot only if field-only evidence remains.

## RELEASE DISPOSITION

**DO NOT SHIP.**

Close the blocker only after a causal RED or proof of contract-only behavior, an approved narrow repair if needed, unchanged RED to GREEN, the applicable zero-warning final gate, and one consolidated field pilot if external behavior remains implicated.

## STRUCTURAL FALSIFICATION PREPARATION

### Legacy adequacy / history dependence

**TEST NAME:** production-reachable legacy adequacy history RED  
**STATUS:** BLOCKED — NOT PREPARED, NOT RUN  
**PROVEN DEFECT TARGET:** `LegacyEdit` population-only candidate selection  
**FIXTURE:** intended single-file semantic producer with a production-reachable weaker prior projection  
**PRECONDITION:** a prior valid lifecycle must leave the file indexed while inadequate for a later LegacyEdit-backed interpretation  
**QUERY HISTORY:** weak/partial publication, then authoritative query, compared with cold authoritative query over identical final bytes  
**EXPECTED AUTHORITY:** identical truth and completeness  
**CURRENT EXPECTED FAILURE:** candidate is discovered but suppressed solely by `file_map()` membership  
**INSTRUMENTATION:** candidate outcome ledger remains required if a valid producer path is found  
**PRODUCTION PATH EXERCISED:** must use public production publication and workspace-query preparation  
**WHY THIS IS PRODUCTION-REACHABLE:** not yet established for a semantic divergence. Angular Service already uses `Semantic(High)`; C#/EF catalog interpretation parses cached source independently of stored projection fidelity. External edits without `refresh_workspace` would bypass the documented lifecycle and are not accepted as a causal fixture.  
**EXACT FQN:** none until a valid state transition is proven

### Provider readiness / aggregate completeness

**TEST NAME:** `red_successful_still_indexing_search_cannot_authorize_complete_hydration`  
**STATUS:** CORRECTED RED→GREEN CONFIRMED — user reported all focused provider-readiness, fallback, ready, and failure controls GREEN  
**PROVEN DEFECT TARGET:** successful provider request incorrectly authorizes aggregate completion while readiness is `still_indexing`  
**FIXTURE:** one configured CBM project with successful declaration discovery, injected `StillIndexing` readiness, and one forced Clean publication failure  
**PRECONDITION:** provider search succeeds but Clean-owned authoritative evidence is not established on the first cycle  
**QUERY HISTORY:** two identical `find_entities(ProjectContext)` hydration cycles  
**EXPECTED AUTHORITY:** the failed Clean cycle is incomplete; no CBM completion is recorded; the provider retries; successful Clean fallback on the second cycle may complete independently of CBM readiness  
**OBSERVED FAILURE:** retry and zero durable completion reached their preceding assertions, then line 75 failed because both successful-but-`StillIndexing` reports were classified complete  
**INSTRUMENTATION:** existing readiness injection, discovery-call ledger, and completion-entry count  
**PRODUCTION PATH EXERCISED:** `hydrate_workspace_index -> discover_candidate_paths -> provider readiness aggregation -> HydrationReport::is_complete`  
**WHY THIS IS PRODUCTION-REACHABLE:** it uses the production hydration coordinator and existing provider fake seam; only provider I/O/readiness is injected  
**EXACT FQN:** `mcp::tool_handlers::query::tests_provider_readiness::red_successful_still_indexing_search_cannot_authorize_complete_hydration`

**USER-RUN RED EVIDENCE:** The focused test was discovered and executed. It panicked at
`src/tests/mcp/workspace_query_provider_readiness.rs:75` with:
`successful search is not ready evidence and cannot authorize aggregate completion`.
This proves the intended distinction: provider request success and retryability do not
establish ready evidence or authority to claim aggregate hydration completeness.
The user reported focused GREEN for the earlier truthfulness repair. The stronger
approved architecture subsequently corrected the test boundary: non-ready CBM
cannot establish authority, but it also cannot remain a completion dependency
after successful Clean-owned fallback, publication, reconciliation, and required
semantic processing. The user then reported all five corrected focused commands
GREEN: the narrowed provider-readiness regression, StillIndexing fallback union/
deduplication, Ready fallback suppression, search-failure retry, and degraded-CBM
filesystem fallback control.

### Two-root ProjectContext identity

**TEST NAME:** two-root duplicate-name provenance/history matrix  
**STATUS:** GREEN — user reported the focused two-root matrix passed
**PROVEN DEFECT TARGET:** unresolved live root asymmetry  
**FIXTURE:** primary and additional roots each declaring `ProjectContext : DbContext`  
**PRECONDITION:** independent canonical files and neutral base-type facts  
**QUERY HISTORY:** cold and warm repeat in both primary/additional root orders
**EXPECTED AUTHORITY:** both declaration occurrences and both neutral `HasBaseType` facts survive with physical provenance; name-only derived classification may refuse ambiguity  
**CURRENT EXPECTED FAILURE:** none manufactured; the fixture falsifies occurrence loss, base-fact loss, reconciliation omission, history dependence, and provider-state dependence
**INSTRUMENTATION:** candidate publication counts, C# reconciliation counter, declaration occurrence files, neutral base-fact occurrence files, and query-time classification output
**PRODUCTION PATH EXERCISED:** configured-root discovery, publication, declaration storage, scope filtering, and C# catalog ambiguity  
**WHY THIS IS PRODUCTION-REACHABLE:** ordinary configured primary/additional roots and source files  
**EXACT FQN:** `mcp::tool_handlers::query::tests_warm_state_authority::two_root_projectcontext_preserves_independent_source_authority`

The verified matrix runs fresh states for CBM `Ready`, CBM `StillIndexing` with
Clean filesystem fallback, and CBM unavailable with Clean filesystem fallback,
then repeats each with primary/additional roles reversed. Every case asserts a
cold publication of both C# files, exactly one aggregated C# reconciliation,
two independent `builtin/Class/ProjectContext` declaration occurrences, two
file-provenanced `HasBaseType -> builtin/TypeRef/DbContext` facts, an equivalent
warm result without recompilation, and no fabricated local `Extends` target or
`dotnet/DbContext` source declaration. The user reported the focused matrix
GREEN. The original root asymmetry did not reproduce across these deterministic
provider, history, and root-order variants; no additional production repair is
authorized by this path.

### Declaration contract control

**TEST NAME:** exact declaration identity does not equal derived EF classification  
**STATUS:** CONTROL — DESIGNED, NOT CODED, NOT RUN  
**EXACT FQN:** reserved: `mcp::tool_handlers::query::tests_warm_state_authority::projectcontext_declaration_filters_preserve_exact_identity_contract`

### Batch second-pass history control

**TEST NAME:** authoritative query truth is invariant across standalone and batch histories  
**STATUS:** CONTROL / POSSIBLE RED — DESIGNED, NOT CODED, NOT RUN  
**EXACT FQN:** reserved: `mcp::tool_handlers::query::tests_warm_state_authority::authoritative_query_truth_is_batch_history_independent`

### Diagnostic accounting control

**TEST NAME:** current candidates are not omitted candidates  
**STATUS:** CONTROL — DESIGNED, NOT CODED, NOT RUN  
**EXACT FQN:** reserved: `mcp::tool_handlers::query::tests_warm_state_authority::discovered_minus_published_is_not_missing_coverage`

### Reverse lower-bound control

**TEST NAME:** incomplete reverse zero remains lower-bound  
**STATUS:** CONTROL — existing coverage tests already exercise this contract; no duplicate added before the first RED is run  
**EXACT FQN:** existing coverage module tests; exact selection deferred until after the first RED gate

### Source-hash, fidelity, and dependency variants

**STATUS:** NOT PREPARED  
Source-hash mutation without explicit refresh bypasses the documented external-edit lifecycle. The Angular high-fidelity path already validates and upgrades through `HydrationRequirement::Semantic(High)`, making it a control rather than a LegacyEdit RED. A dependency-staleness fixture needs a proven production transition before test code is justified.

## CBM READINESS PROTOCOL AND ACQUISITION-PROGRESS DECISION

### Executive answer

Non-ready CBM is an intermediate provider state, not a terminal acquisition
outcome. It became terminal for one `workspace_query` cycle because hydration
used successful provider transport/search as the membership test for
`successful_roots`. The later fallback pass asks only whether a root is absent
from that set. Consequently, a successful query against a still-building graph
suppressed filesystem fallback even though readiness explicitly said the
candidate set could be incomplete.

This is a historical boundary mismatch, not an intended authority rule.

### Intended CBM indexing protocol

CBM indexing begins asynchronously when `GraphBridge` is constructed.
`ensure_indexed_for` is deliberately report-only:

- `Complete -> Ready`;
- `InProgress -> StillIndexing`, with elapsed time;
- `Failed -> Err`;
- `NotStarted -> StillIndexing { 0 }`.

It does not wait, poll, or spawn another indexer. In-progress indexing becomes a
failure after 60 seconds when the readiness gate is observed again. CBM request
transport has its own 30-second default timeout, but that is not an acquisition
readiness wait budget.

Direct graph tools honor the non-blocking protocol by returning a temporary
indexing error and telling the caller to retry once, then use host-native or
Clean-owned fallback. They do not present an empty graph as authoritative.

### Why workspace hydration diverged

`workspace_query` calls the readiness gate but then still performs the project
search. If that search returns `Ok`, hydration:

1. accepts any returned paths as advisory candidates;
2. inserts the root into `successful_roots`;
3. records exceptional readiness metadata;
4. avoids durable completion unless readiness was `ready`.

Step 2 controls fallback. The later filesystem-root reconciliation adds fallback
only for roots absent from `successful_roots`. Thus search transport success was
accidentally made equivalent to root coverage even though completion caching
correctly retained the readiness distinction.

History shows the responsibilities arrived in layers: project search/readiness
handling first, fallback-root coverage next, and generation-scoped discovery
completion afterward. The resulting topology preserved retryability but left the
fallback gate keyed to search success rather than coverage readiness.

### Governing architectural contracts

The durable contracts resolve the authority question:

- CBM is optional advisory candidate discovery; Clean compilation owns truth.
- Optional providers may change cost, not core truth.
- When CBM cannot cover a configured root, deterministic filesystem discovery
  scans that trusted root.
- Still-indexing cycles are incomplete and retryable.
- The provider is part of completion identity so CBM state cannot suppress a
  required filesystem acquisition path.

A non-ready graph cannot prove it covered the root. Therefore it cannot be the
sole acquisition provider for an authoritative hydration cycle.

### Alternatives

#### A. Await CBM readiness inside workspace_query

**Rejected as the default recommendation.** The bridge is intentionally
non-blocking and report-only. Waiting would require a new polling interval,
request deadline, cancellation propagation, fairness policy, and behavior after
the existing 60-second indexing-state timeout. Those are global externally
observable policies and remain separately unapproved. Waiting would also hold the
graph-bridge coordination boundary across repeated readiness checks unless the
hydration lifecycle were redesigned carefully.

#### B. Accept non-ready CBM candidates without authoritative fallback

**Truthful but insufficient for authoritative acquisition.** Non-ready hints
alone cannot complete an authoritative query. Once Clean-owned fallback and all
required downstream processing succeed, however, CBM readiness becomes
irrelevant and completion may be claimed.

#### C. Immediately fall back to Clean-owned filesystem acquisition for that root

**Recommended.** Retain any admissible CBM candidate paths already returned, but
treat readiness other than `Ready` as lack of root coverage and include that
root in the existing filesystem fallback set. Clean-owned scanning then supplies
the complete literal candidate universe under the existing trusted-root,
extension, exclusion, and compilation gates.

This uses an existing fallback rather than inventing waiting, retry, timeout, or
semantic policy. CBM can continue indexing in the background, and a later request
may use it once ready. Performance may differ; truth and completeness do not.

#### D. Add a configurable bounded choice between waiting and fallback

**Not justified now.** It introduces global budgets/defaults and externally
observable latency behavior without evidence that immediate fallback is
insufficient. It can be reconsidered only after measurements show a real need
and cancellation/timeout architecture is approved.

### Recommended decision

For `workspace_query` acquisition:

```text
Ready + successful search
  -> CBM covers the root

StillIndexing + successful search
  -> keep returned paths as advisory hints
  -> CBM does not cover the root
  -> run existing filesystem fallback for that root
  -> do not cache CBM completion

readiness failure or search failure
  -> CBM does not cover the root
  -> run existing filesystem fallback for that root

filesystem fallback unavailable/incomplete
  -> return partial/unavailable honestly
```

The decision is per root and per acquisition cycle. No waiting occurs inside the
request. No new retry count, timeout, budget, cache key, public contract, or
semantic authority is introduced.

### Effects and preserved boundaries

Affected: workspace-query candidate acquisition and its provider diagnostics.

Preserved: direct CBM graph-tool retry behavior; CBM background indexing; trusted
root admission; discovery modes; completion generations; query requirements;
candidate compilation; publication; one C# reconciliation per hydration cycle;
declaration identity; generic lower-bound graph semantics.

Potential cost: a cold request may perform both a partial CBM search and a
filesystem scan for the same root. That is acceptable under the existing
optional-provider invariant and can later be optimized only with equivalent
coverage proof.

### Implementation status

**IMPLEMENTED — CORRECTED FOCUSED GREEN CONFIRMED.** Successful non-ready searches
retain their returned paths but no longer enter `successful_roots`; the existing
filesystem fallback therefore covers the root, unions its candidates with CBM
hints, and relies on normal candidate selection to deduplicate shared paths.
Ready successful searches retain their covered-root behavior. Aggregate
completion now follows terminal Clean-owned acquisition/publication status,
not optional CBM readiness.

The focused structural regression is
`mcp::tool_handlers::query::tests_provider_readiness::still_indexing_cbm_candidates_are_unioned_with_filesystem_fallback`.
It proves the fallback scan runs, the CBM-only hint remains usable, a path
returned by both providers is compiled and published once, and completed Clean
fallback may establish completion. The earlier provider-readiness regression is
narrowed to retain its architectural intent through a forced Clean publication
failure and retry. The user reported all five focused verification commands
GREEN; no Cargo command was run by the agent.
