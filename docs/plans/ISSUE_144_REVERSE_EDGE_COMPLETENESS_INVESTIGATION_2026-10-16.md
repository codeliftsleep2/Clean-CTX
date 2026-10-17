# Agent MaxHeadRoom — Issue #144 Reverse-Edge Completeness Investigation

**Date:** 2026-10-16

**Mode:** static causal audit only

**Branch:** `(fix)Performance-Regression-Issues` at `2a565ffb`

**Comparison base:** `main` at `b5dce136`

No production code was changed. No build, test, benchmark, MCP, CBM, or live
workspace query was run. Runtime observations quoted below are inputs from the
issue report, not results reproduced by this investigation.

## 1. Executive verdict

**Causal classification: PROVIDER COVERAGE AUTHORITY DEFECT.**

The reported `discovered: 23, compiled: 11` is not evidence that a current
five-file cap, compilation budget, or twelve omitted files caused the result.
The old global five-candidate hydration cap was removed by `2986ce76`; current
candidate selection has no numeric acquisition limit. `discovered` counts the
raw paths returned by discovery in that hydration cycle, while `compiled`
counts only successful new semantic publications. Current, adequate files,
deduplicated owners, unsupported/excluded/missing paths, and failed publications
can all contribute to a discovered/compiled difference. The response deliberately
names the subtraction `discovered_not_compiled_this_cycle`; it is not an
omitted-file count.

The causal defect is one boundary earlier. For a configured CBM project, a
`Ready` provider plus a successful inbound-reference search marks that root
covered and suppresses the Clean-CTX filesystem fallback. The provider query
returns only caller paths already represented by qualifying relationships in
the provider graph. Neither readiness nor successful execution proves that the
returned path set exhausts every current source file capable of producing an
incoming `TypeRef`. Consequently, an optional provider can shrink the acquisition
universe—23 reported paths versus the issue's approximately 213 textual
name-bearing files—rather than merely changing acquisition cost.

The public exact-edge contract truthfully remains an indexed lower bound:
`source_complete:false`, `result_semantics:"lower_bound"`, and
`omitted_possible:true`. That contract is explicit in code and in
`docs/CLAUDE_INTEGRATION_RULES.md`. It prevents the response from claiming a
complete negative, but it does not make provider-dependent semantic truth a
sound acquisition architecture. A provider may offer candidates; it may not
become exclusive root-coverage authority without an adequate completeness proof.

**Final recommendation: PROCEED TO COMPLETENESS DESIGN.**

Do not restore a numeric cap and do not treat `23 - 11` as the repair target.
Design an explicit reverse-edge acquisition requirement and a proof-bearing
provider/root coverage decision. Until such a proof exists, provider results
must be advisory and Clean-CTX must retain the authoritative fallback for the
required source universe.

## 2. Question-by-question findings

| Question | Finding | Evidence class |
| --- | --- | --- |
| What does reverse `TypeRef` need? | Every admitted, supported source owner capable of publishing the requested incoming relationship, at evidence fidelity adequate to produce that relationship. | Code + architectural inference |
| What does it currently request? | `HydrationRequirement::LegacyEdit`; only Angular `Service` receives the distinct semantic requirement. | Code |
| Why can discovery be 23 rather than ~213 textual matches? | A Ready, successful provider inbound search covers the root and prevents filesystem literal scanning; only provider graph callers enter that lane. Scope, exclusions, supported extensions, and deduplication can narrow further. | Code; exact observed composition not established |
| Why compile only 11? | The selection phase removes ineligible/duplicate owners and, for `LegacyEdit`, files with a current High projection, matching source hash, and current semantic dependencies. `compiled` then counts successful new publications only. | Code |
| Is there a current cap? | No acquisition cap was found. The historical five-candidate cap existed at `4647341b` and was removed by `2986ce76`. The remaining 16-entry truncation applies only to diagnostic project coverage. | Code + history |
| Does completed hydration prove source completeness? | No. It proves completion of the selected discovery/fallback operation. Generic exact-edge coverage remains a lower bound even when hydration reports completed. | Code + documented contract |
| Is the behavior intentional? | The lower-bound response contract is intentional. Provider exclusivity without a source-universe proof is not an adequate correctness boundary. | Code + architecture |
| Is this primarily a performance regression? | No. Recent performance work changes reuse and repeated work, but the first incorrect authority decision is provider coverage. Performance changes can make an incomplete acquired set stable on warm requests. | Code + history |

## 3. Exact production call path

```text
workspace_query dispatcher
  -> query::handle_workspace_query
  -> WorkspaceQueryOperation::ReverseEdges
  -> edges::prepare_reverse_edges
  -> edges::try_prepare_reverse_edges
       -> PreparationContext::scope
       -> initial scoped identity/index read (result deliberately discarded)
       -> edge_hydration_requirement
            builtin/TypeRef -> HydrationRequirement::LegacyEdit
       -> PreparationContext::hydrate
            request-local memo key includes direction/name/requirement
       -> hydrate_workspace_index_for
            discovery_kind("reverse_edges") -> InboundReference
            discover_candidate_paths
              -> configured CBM project/root evaluation
              -> CBM inbound_reference_paths_in_project, if available
              -> filesystem fallback for roots not declared successful
            select_candidates
              -> normalize semantic owner
              -> require admitted supported regular file
              -> deduplicate owner
              -> omit current adequate LegacyEdit projection
            compile_candidate_for each selected owner
              -> publish semantic evidence or report Current/Failed
            reconcile C# exact calls once, if applicable
            mark discovery complete only after successful publication/reconciliation
       -> resolve exact identity after hydration
       -> construct request-local refinement catalogs, where applicable
       -> exact_identity_coverage
       -> deferred evaluator reads WorkspaceIndex reverse adjacency in scope
```

The final edge answer comes from Clean-CTX's `WorkspaceIndex`, not directly from
provider relationship rows. The defect is nevertheless provider-mediated:
provider paths decide which files Clean-CTX is given an opportunity to compile.

## 4. Requirement adequacy

### 4.1 Declared requirement

`edge_hydration_requirement` maps the reported
`reverse_edges(builtin/TypeRef/ContextA)` request to `LegacyEdit`. In current
selection logic, that requirement means that a candidate is reusable only when
the indexed semantic projection is:

1. High fidelity;
2. for the current source hash; and
3. backed by current semantic dependencies.

That is a meaningful **evidence-adequacy** rule. It repairs the older behavior
where mere index presence could suppress recompilation. It does not state or
prove the **source-universe** requirement for an incoming-reference query.

### 4.2 Required source universe

For reverse `TypeRef`, the target declaration alone is insufficient. The query
asks which asserting subjects depend on the target. The required acquisition
universe is therefore source-owner oriented: every in-scope supported file that
could publish an incoming `TypeRef` for the exact written identity, subject to
the repository's declared exclusions and language support.

Literal occurrence of `ContextA` is a conservative filesystem candidate test,
not semantic proof. Approximately 213 textual matches are therefore an upper
bound on possible source owners, not an expected edge count. Conversely, 23
provider graph callers are an indexed-evidence set, not proof that the other
textual owners cannot publish the relationship.

### 4.3 Requirement/selection mismatch

The system has a strong per-file adequacy predicate but no reverse-edge-specific
source-completeness requirement. `LegacyEdit` answers “may this discovered file
be reused?” It does not answer “have all files capable of asserting this incoming
relationship been discovered?” Those questions must not share an implicit
completion signal.

## 5. Provider coverage decision tree

For each configured provider project/root, current discovery behaves as follows:

```text
cached complete for (scope, inbound, name, requirement)?
  yes -> skip search; treat that root as covered for this discovery
  no  -> provider available?
          no -> schedule filesystem fallback
          yes -> read readiness and attempt inbound search
                  search fails -> schedule filesystem fallback
                  search succeeds + Ready
                    -> append provider paths
                    -> mark root successful; suppress filesystem fallback
                  search succeeds + StillIndexing or readiness failure
                    -> append provider paths as advisory
                    -> retain filesystem fallback

filesystem scan every root not marked successful
  -> walk admitted tree
  -> supported source extensions only
  -> literal name-bearing files
  -> append candidates
```

The `StillIndexing` path is conservative after `9d4629d0`: successful provider
paths remain useful, but they do not suppress filesystem acquisition. The defect
survives in the `Ready` path because readiness is a provider lifecycle state,
not a per-query exhaustiveness certificate.

`inbound_reference_paths_in_project` runs a graph query equivalent to callers
with any non-definition relationship to a target whose name matches the request;
`DEFINES` and `DEFINES_METHOD` are filtered. It returns caller file paths. It does
not return or validate an inventory of all source owners, all supported relation
producers, or a current-source coverage certificate. Candidate observation checks
returned paths; it cannot prove that unreturned paths are irrelevant.

## 6. Counter semantics

| Field | Actual meaning | What it does not mean |
| --- | --- | --- |
| `discovered` | Raw candidate-path vector length produced in the cycle before semantic-owner selection. | Files proven capable, unique owners, or complete source universe. |
| `compiled` | Selected candidates that successfully produced a new publication (`Published` or reconciled C# publication). | Every discovered path needing evidence. |
| `discovered_not_compiled_this_cycle` | Saturating arithmetic difference between those two counters. | Missing files, omitted edges, failures, or incompleteness count. |
| discovery `completed` | No unresolved fallback/publication/reconciliation failure for the discovery operation used. | Semantic-domain source completeness. |
| `source_complete:false` | The exact graph response lacks an earned source-universe certificate. | That returned edges are false; they are an indexed lower bound. |

For the observed 23/11, static code can establish the counter definitions but
cannot assign the remaining 12 among adequate-current, duplicate-owner,
unsupported, excluded, missing, or failed categories. The response does not
retain a reason histogram. Inferring “12 consumers omitted” would be false.

## 7. Cold, warm, and failure behavior

### Cold, Ready provider

The provider inbound result becomes the candidate universe for that root. Clean-CTX
compiles only candidates lacking adequate current evidence, then can cache the
discovery as complete. Files absent from the provider result receive no filesystem
opportunity in that cycle.

### Warm after successful completion

The discovery-completion cache skips repeat discovery for the same scope, mode,
name, and requirement. The query reads retained index evidence. A warm request
therefore does not progressively discover provider-omitted files; it stabilizes
the first completed acquired set until invalidation/generation change.

### Still indexing or unavailable provider

`StillIndexing`, readiness failure, provider unavailability, and search failure
retain or select filesystem fallback. Provider candidates may supplement the
fallback in the non-ready success case. These lanes are more conservative than
the Ready-success lane.

### Publication failure

Publication/reconciliation failure prevents completion marking, so a later
request can retry. This is legitimate convergence and is distinct from a Ready
provider result that was successfully but incompletely treated as root coverage.

## 8. Main/current and performance-history audit

### 8.1 Lower-bound contract

`main` already emits `source_complete:false` and `result_semantics:"lower_bound"`
for generic exact-identity graph coverage. The current branch did not introduce
that public limitation. Documentation explicitly states that an exact reverse
edge count is a minimum and that the discovered/compiled gap can contain already
indexed or deduplicated candidates.

### 8.2 Candidate cap

Historical reverse-edge hydration (`4647341b`) selected candidates through a
global `HYDRATION_MAX_CANDIDATES = 5`. Commit `2986ce76`
(`fix-workspace-query-hydration-completeness`) removed that cap. Current
`select_candidates` sorts the complete eligible selection and does not call
`take`, truncate it, or apply a compile budget. The 16-entry project-coverage
limit affects diagnostics only.

### 8.3 Relevant later changes

- `9d59499e` accelerated filesystem fallback traversal and reads. This changes
  cost, not the intended candidate predicate.
- `f792a3c7` cached completed discovery per project/mode/generation. This removes
  repeat work but makes the quality of the first completion decision decisive.
- `9d4629d0` restored fallback when provider readiness is not `Ready`; this is a
  correctness improvement but leaves Ready-success exclusivity intact.
- `61126a16` strengthened LegacyEdit reuse to current High-fidelity evidence,
  current source hash, and current dependencies. This improves file adequacy;
  it does not expand provider discovery.
- Recent request-local catalog and C# reconciliation work removes redundant work
  or repairs post-publication interpretation. No inspected change establishes a
  provider source-universe proof for reverse TypeRef.

Accordingly, Issue #144 is not explained by a newly introduced selection cap.
Performance work can expose or preserve the defect by avoiding repeat discovery,
but the first failing correctness boundary is provider coverage authority.

## 9. Relationship to the other reported observations

### Issue 1 — `find_entities` versus declaration-only EF result

**Classification: INDEPENDENT FILTER/PROJECTION DEFECT.**

The reported contradiction—an occurrence visible under `find_entities`, but no
matching declaration under the requested declaration-only domain/type filter—is
not explained by reverse-edge candidate selection. Declaration-only queries read
source-proven declaration occurrences and apply exact domain/entity-type filters;
reverse-edge acquisition and adjacency evaluation are not that filtering boundary.
Acquisition may affect what evidence exists in either query, but the stated
same-name occurrence/filter mismatch requires its own declaration classification
and projection trace.

### Issue 2 — two `ContextA` declarations receive different domains

**Classification: POSSIBLY RELATED.**

If one declaration lacks the supporting ancestry/type evidence needed for EF
refinement because its owner or dependency was not acquired in the compatible
generation, the same acquisition boundary could contribute. However, aggregate
23/11 diagnostics cannot identify either declaration owner, its source hash,
dependency generation, or refinement input. Since both declarations reportedly
appear, the asymmetry may instead be an independent classification/refinement
defect. Static evidence is insufficient to collapse it into Issue #144.

## 10. Minimal deterministic reproduction design

No live reproduction is required to establish the code-level authority defect.
The smallest regression should nevertheless prove the observable consequence:

1. Create one owned workspace root with `ContextA.cs`, `KnownConsumer.cs`, and
   `ProviderOmittedConsumer.cs`.
2. Make both consumers publish the same exact incoming `builtin/TypeRef/ContextA`
   relationship when compiled.
3. Inject a provider project that reports `Ready` and whose inbound search returns
   only `KnownConsumer.cs`.
4. Run cold reverse-edge preparation and assert the omitted consumer is absent
   under the current behavior while the response remains lower-bound.
5. Run the same clean fixture with the provider unavailable and assert filesystem
   fallback acquires both consumers.
6. Repeat the Ready request warm and assert the completion cache does not add the
   omitted consumer.
7. After the approved repair, invert the first assertion: Ready provider hints
   must not suppress required authoritative acquisition unless the injected
   provider supplies the newly defined completeness certificate.

Use existing hydration test seams for readiness, search results, and searched
projects. If they cannot distinguish scan execution, add a **test-only** fallback
scan observation—not production telemetry. The regression belongs under
`src/tests/**`; a live harness cannot substitute for it.

Suggested eventual user-run focused gate after implementation (not run here):

```powershell
cargo test --all-features workspace_query_reverse_edge_provider_completeness -- --nocapture
```

The exact test filter may be adjusted to the final tracked test name. Full final
verification remains governed by `docs/agent/verification.md` and must be handed
to the user under repository policy.

## 11. Completeness design recommendation

The repair requires an architectural decision because it changes shared provider,
discovery, caching, and externally observable query behavior.

### Recommended option

Introduce an explicit acquisition requirement for incoming semantic relationships,
separate from per-file fidelity. A provider may suppress authoritative fallback
only when it returns a certificate adequate for all of:

- the exact workspace/root scope and exclusions;
- current provider/source generation;
- every supported language/producer capable of publishing the requested relation;
- the inbound relationship family and exact identity semantics; and
- a complete source-owner inventory, including a defensible empty result.

Without that certificate, treat provider results as candidate accelerators and
run the Clean-CTX fallback required by the acquisition requirement. Cache only
the completed proof-bearing acquisition, not merely successful provider search.
Keep per-file `LegacyEdit` adequacy so already current files do not recompile.

### Alternatives

1. **Always filesystem-fallback reverse edges.** Simplest and safest immediate
   semantics, but can discard legitimate provider acceleration and increase cold
   cost substantially.
2. **Provider inventory reconciliation.** Compare provider coverage with a cheap
   authoritative source inventory, then search/compile only the uncovered owners.
   Potentially efficient, but it still needs explicit generation and producer
   capability semantics.
3. **Keep current lower-bound behavior only.** No implementation change, but it
   preserves provider-dependent truth and offers no complete consumer query. This
   is not recommended even though the response disclosure is honest.

Do not redefine `discovered`, inflate `compiled`, or remove the lower-bound flags
to make diagnostics look complete. Those would hide rather than repair the
authority defect. Whether the public reverse-edge API should eventually earn
`source_complete:true` is a separate approved contract decision; the first repair
is to stop granting exclusivity without proof.

## 12. Evidence map

Primary current authorities inspected:

- `src/mcp/tool_handlers/query/edges.rs` — reverse preparation, requirement,
  identity resolution, coverage construction, and reverse evaluation.
- `src/mcp/tool_handlers/query/prepare.rs` — request-local hydration memoization.
- `src/mcp/tool_handlers/hydration.rs` and `hydration/cache.rs` — orchestration
  and completion caching.
- `src/mcp/tool_handlers/hydration/discovery.rs` — provider readiness, successful
  roots, and filesystem fallback selection.
- `src/mcp/tool_handlers/hydration/filesystem.rs` — admitted literal-name scan.
- `src/mcp/tool_handlers/hydration/publication.rs` — owner normalization,
  adequacy filtering, compilation, and publication outcomes.
- `src/mcp/tool_handlers/query/diagnostics.rs` — public discovered/compiled fields.
- `src/mcp/tool_handlers/query/coverage.rs` — lower-bound contract and cycle gap.
- `src/cbm/project_search.rs` — inbound provider graph query.
- `docs/CLAUDE_INTEGRATION_RULES.md` — public minimum-count/gap semantics.
- `docs/plans/ISSUE_142_FINAL_CROSS_CUTTING_ACCEPTANCE.md` — accepted distinction
  between hydration completion and source completeness.

### Files changed

- Added this investigation report only:
  `docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_INVESTIGATION_2026-10-16.md`.
- No production, test, configuration, or generated file was changed.

### Static checks performed

- Focused `rg` searches over query, hydration, provider-search, tests, and
  completeness documentation.
- Focused source reads of the call path and its branch-specific helpers.
- Focused `git log`, `git show`, and `main...HEAD` history comparison for the
  acquisition/completeness path.
- `git diff --check --
  docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_INVESTIGATION_2026-10-16.md`
  completed successfully.
- No Cargo command or runtime characterization was performed.

### Runtime authorization disposition

Static evidence is sufficient for the primary causal classification. No runtime
command is required to complete this investigation. The focused command shown in
section 10 is a proposed post-implementation regression gate and requires separate
authorization; it was not run.

## 13. Final disposition

- **Primary cause:** provider coverage authority defect.
- **Contributing design limitation:** intentional generic exact-edge lower-bound
  contract with no earned source-completeness certificate.
- **Not causal:** current five-file cap (removed), diagnostic project truncation,
  or the arithmetic 23/11 gap by itself.
- **Performance interaction:** cache/reuse makes a completed acquired set stable;
  it does not create the first incomplete provider universe.
- **Issue 1:** independent filter/projection defect.
- **Issue 2:** possibly related; requires declaration-owner/generation evidence.
- **Recommendation:** **PROCEED TO COMPLETENESS DESIGN** before production change.
