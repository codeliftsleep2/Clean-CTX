# Issue #148 Provider-State Reverse-Edges Divergence Investigation

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Authoritative branch:** `(fix)Performance-Regression-Issues`  
**Mode:** Static causal investigation plus field-observation reconstruction  
**Production implementation authorized:** NO  
**Runtime execution authorized:** NO

## 1. Executive verdict

Issue #148 is a shared acquisition/proof-lifecycle defect. It is not Angular-specific, C#-specific, a reverse-index defect, or a framework producer defect.

**FIELD-OBSERVED:** On fixed source, fresh servers with CBM unavailable returned 9 and 16 Angular `Injects` reverse edges for two services; fresh CBM-ready servers returned zero and cached that zero. The same CBM-state dependence affected unfiltered `find_entities ContextA`: 7 occurrences without CBM versus 5 with ready CBM. Exact `builtin/TypeRef` reverse lookup, EF `HasEntity`, ASP.NET controller facts, and `entities_in_file` remained stable.

**STATICALLY-PROVEN:** The current implementation activates provider-neutral acquisition proof only when `hydrate_workspace_index_for_target` receives an `AcquisitionTarget`. Query preparation supplies that target only for exact reverse `builtin/TypeRef`. Angular `Service` reverse lookup and ordinary `find_entities` use the legacy path with no target.

**STATICALLY-PROVEN:** For every no-target path, a ready CBM `Ok(...)` result—empty, strict subset, or broader candidates—adds the project root to `successful_roots` and queues its CBM discovery scope for completion. Roots in `successful_roots` do not enter filesystem fallback. After candidate publication, that provider-specific scope/mode/name is cached as complete. This is the first causal divergence.

**STATICALLY-PROVEN:** The #144 proof architecture itself is sound for the exact incoming-TypeRef capability it models. Its routing is incomplete: `CoverageUnit`, source inventory, residual-unit proof, and proof-backed completion are not constructed for the affected Angular relationship or generic declaration discovery. The legacy ready-provider completion contract remains alongside the new authority boundary.

**STATICALLY-PROVEN:** When equal owner files are compiled, publication and `WorkspaceIndex` insert the same semantic edge into both forward and reverse adjacency. Provider state is not part of artifact identity, semantic projection currentness, or adjacency retrieval. The field divergence therefore originates before publication; the reverse index faithfully exposes an under-acquired generation.

**Recommendation:** Block R7. Repair the shared hydration root-satisfaction boundary so advisory provider success never removes a root from required Clean acquisition unless a compatible proof covers the query's actual semantic requirement. Preserve R0–R6, then reverify R6 because its durable and request-local semantics depend on shared acquisition and publication even though its resolution architecture is not falsified.

## 2. Field-observation inventory

The Issue #148 report is authoritative field evidence. The proprietary workspace was not requested or reconstructed.

| Observation | Query | Provider state | Cold/warm | Scope | Expected | Observed | Coverage metadata | Producer/family | Evidence |
|---|---|---|---|---|---|---|---|---|---|
| O1 | reverse `angular/Service/ServiceX` | CBM unavailable; filesystem | first query | RootA request, four configured roots | 9 injectors | 9 edges; 20 files discovered/compiled | filesystem fallback | TypeScript Angular `Injects` | FIELD-OBSERVED |
| O2 | same as O1 | CBM ready | fresh, first query after 60 s | same | 9 | 0 | `identity_not_indexed`, lower bound; no discovered/compiled counts | same | FIELD-OBSERVED |
| O3 | same as O2 | CBM ready | repeated warm query | same | 9 | 0 again | `identity_not_indexed` | same; discovery cache involved | FIELD-OBSERVED |
| O4 | `find_entities ServiceX`, then O2 | CBM ready | fresh sequence | same | declaration warming should not remove injectors | 0 Angular edges | declaration discovery did not repair inbound discovery | different discovery modes | FIELD-OBSERVED |
| O5 | reverse `builtin/TypeRef/ServiceX`, then O2 | CBM ready | fresh sequence | same | 9 | TypeRef query compiled 20 files; Angular query then returned 9 | exact-TypeRef route performs Clean acquisition | neutral TypeRef then Angular `Injects` | FIELD-OBSERVED |
| O6 | reverse `angular/Service/ServiceY` | CBM unavailable vs ready | one fresh instance per state | same | 16 | 16 vs 0 | ready result `identity_not_indexed`; repeated zero cached | TypeScript Angular `Injects` | FIELD-OBSERVED |
| O7 | unfiltered `find_entities ContextA` | unavailable vs ready | fresh baseline | all configured roots | 7 occurrences | 7 vs 5; ready omitted two `builtin/TypeRef` occurrences | filesystem compiled 207/207; ready compiled 115/236 | cross-language name/occurrence publication | FIELD-OBSERVED |
| O8 | forward EF `HasEntity` | unavailable vs ready | baseline | same | stable | 19 vs 19 | not reported divergent | C#/.NET EF | FIELD-OBSERVED |
| O9 | reverse exact `builtin/TypeRef/ContextA` | unavailable vs ready | baseline | same | stable | 2 vs 2 | exact TypeRef proof route | neutral C# TypeRef | FIELD-OBSERVED |
| O10 | controller entity/route/action queries | unavailable vs ready | baseline | same | stable | identical | not reported divergent | C# ASP.NET | FIELD-OBSERVED |
| O11 | `entities_in_file` | unavailable vs ready | baseline | one file | stable exact | identical; authoritative complete | file-owned, no name hydration | generic file projection | FIELD-OBSERVED |
| O12 | CBM graph query for ServiceX | CBM ready | observed separately | RootC project | advisory candidates only | one `DEFINES` row for the service definition, no injection row | not Clean coverage | CBM graph | FIELD-OBSERVED |
| O13 | Issue runner/baselines | unavailable vs 60-second ready | fresh instances | 13 cases per state | state-independent truth | affected and stable groups above | binary hash prefix `4f03ef67` retained | mixed | FIELD-OBSERVED |

The updated evidence proves breadth across two languages (TypeScript and C#), two query operations (`reverse_edges`, `find_entities`), at least two semantic families (Angular `Injects` and neutral TypeRef occurrence publication), and a multi-root workspace. It does not prove every framework or relationship is affected. Stable controls demonstrate that the defect is conditional on acquisition routing, not universal semantic nondeterminism.

## 3. Updated blast radius

| Candidate radius | Verdict | Evidence |
|---|---|---|
| A producer-specific | rejected as primary | O2/O6 and O7 cross Angular and neutral C# publication. |
| B framework-specific | rejected | Angular and unfiltered builtin occurrences diverge. |
| C language-specific | rejected | TypeScript and C# are both observed. |
| D relationship-family-specific | rejected as primary | `Injects` reverse results and occurrence sets both diverge. |
| E reverse-edges preparation-specific | contributory, not complete | Angular uses the legacy reverse route, but `find_entities` reaches the same defect through declaration mode. |
| F shared acquisition/proof lifecycle | **primary** | one ready-provider root-satisfaction branch suppresses Clean fallback for both modes. |
| G evidence adequacy/reuse | amplifier only | discovery completion reuses the bad acquisition decision; semantic projection reuse is source/fidelity checked. |
| H publication/reconciliation | not the first defect | missing owners never reach publication; equal owners use one provider-neutral path. |
| I cache/generation compatibility | secondary amplifier | the cache key is internally consistent but records semantically inadequate provider completion. |
| J query projection/exposure | not causal | lower-bound metadata is truthful but cannot restore omitted facts. |
| K multiple independent defects | not required by current evidence | one root-satisfaction defect explains every affected observation. |

**Earliest shared boundary:** conversion of ready provider search success into root satisfaction and reusable discovery completion without proof of the query's required Clean-authoritative source universe.

## 4. Relevant #144 architecture

Issue #144 introduced an explicit acquisition model for exact incoming `builtin/TypeRef` relationships:

- `AcquisitionTarget::exact_type_ref` names the exact neutral target.
- `CoverageUnit` binds canonical root, source kind, producer generation, producer capability, and relationship families.
- `AcquisitionRequirement::incoming_type_ref` requires every unit.
- filesystem inventory creates Clean-owned `InventoryManifest` values.
- a certified source predicate can produce a claim; claim validation checks target, scope, inventory generation, admission policy, predicate version, contract version, exhaustiveness, truncation, and limitations.
- `AcquisitionSatisfaction` records a unit only after semantic publication completes.
- residual units prevent exact completeness.
- `AcquisitionCompletionCache` stores only source-complete satisfaction under roots, producer generations, policy, predicate/contract versions, and evidence requirement.

**STATICALLY-PROVEN:** This architecture exists in `src/mcp/acquisition/**` and `src/mcp/tool_handlers/hydration/acquisition.rs`.

**STATICALLY-PROVEN:** Its currently modeled producer capability is only `BuiltinTypeRefIncoming`, with `HasConstructorParameterType` and `HasBaseType`. It does not model Angular `Injects` or generic declaration occurrence acquisition.

## 5. Current #144 repair audit

| Required property | Current implementation | #148 traversal/bypass | Verdict |
|---|---|---|---|
| CBM candidates are hints only | provider paths pass through path validation, Clean compilation, extraction, and publication | both affected paths still compile CBM candidates rather than importing CBM edges | holds for fact creation |
| readiness cannot satisfy a `CoverageUnit` | proof satisfaction is Clean inventory/claim based | affected paths construct no units; readiness instead satisfies legacy root discovery | holds inside proof path; bypass outside it |
| search success cannot satisfy a `CoverageUnit` | claims require exhaustive compatible Clean proof | no requirement exists for O2/O7 | holds inside proof path; bypass outside it |
| empty provider has no negative authority | exact TypeRef forces fallback despite empty `Ok` | legacy paths mark ready empty `Ok` successful | **violated outside exact TypeRef** |
| subset cannot suppress Clean acquisition | exact TypeRef root is not successful without proof | declaration and legacy inbound paths mark it successful | **violated outside exact TypeRef** |
| residual units force Clean acquisition | exact path obtains inventories and residual units | affected paths never construct residual units | bypass |
| reuse requires compatible source/dependencies | semantic projection checks source hash, fidelity, and dependencies | under-acquired files have no artifact to check | downstream rule holds but cannot help |
| proof satisfaction is provider-neutral | `ValidatedAcquisitionProof` provenance cannot bypass validation; current claims are Clean | no proof attempted for affected paths | holds where invoked |
| completion cache cannot be earned from unproved provider subset | acquisition completion rejects non-complete satisfaction | separate discovery cache accepts ready provider completion | **dual-cache boundary incomplete** |

The exact-TypeRef repair is internally coherent. The defect is the retained parallel completion authority in legacy hydration.

## 6. Provider-unavailable call path

Representative Angular path:

```text
workspace_query(reverse_edges angular/Service)
  -> prepare_reverse_edges
  -> edge_hydration_requirement = Semantic(High)
  -> PreparationContext::hydrate (no AcquisitionTarget)
  -> hydrate_workspace_index_for_target(target=None)
  -> discover_candidate_paths(InboundReference)
  -> no usable bridge / unavailable project
  -> filesystem_only_discovery or fallback_roots
  -> literal source inventory/candidate scan in every pending configured root
  -> select absent/stale/inadequate owners
  -> semantic compilation at High
  -> publication replaces owner semantic projection
  -> WorkspaceIndex inserts forward and reverse edge occurrences
  -> reverse lookup, scope filtering, projection
  -> lower-bound coverage unless an exact proof contract applies
```

**STATICALLY-PROVEN:** Filesystem fallback sees every admitted, non-excluded supported source file containing the literal target; candidate semantics still come only from Clean compilation.

**FIELD-OBSERVED:** This path found 20 TypeScript files and published 9/16 injector edges.

## 7. Provider-ready call path

```text
workspace_query(name-bearing query)
  -> legacy hydrate(target=None)
  -> requires_provider_neutral_proof = false
  -> ready CBM search returns Ok(paths), possibly empty/subset
  -> root inserted into successful_roots
  -> CBM scope queued in completions
  -> filesystem fallback excludes successful root
  -> only provider-returned owners can compile
  -> completion recorded after those candidates publish (zero candidates also succeeds)
  -> repeated same scope/mode/name skips discovery
  -> WorkspaceIndex lacks semantics from omitted owners
  -> query returns the smaller lower-bound result
```

**STATICALLY-PROVEN:** Both production and test-injected provider branches have the same predicate: ready plus either non-inbound discovery or no provider-neutral-proof requirement.

**STATICALLY-PROVEN:** `find_entities`, `forward_edges`, and `transitive_dependencies` all use `Declaration`; `reverse_edges` uses `InboundReference`. Mode separation explains why `find_entities ServiceX` does not warm Angular inbound discovery.

## 8. First causal divergence

The first divergence is in `discover_candidate_paths`, not in framework analysis:

1. `execution.rs` derives `requires_provider_neutral_proof` solely from `target.is_some()`.
2. only exact reverse `builtin/TypeRef` query preparation supplies a target.
3. ready CBM success for every other path inserts the root into `successful_roots` and queues completion.
4. the filesystem loop adds only roots absent from `successful_roots`.

The provider result has therefore changed which source owners Clean observes before any semantic producer runs.

## 9. Acquisition-requirement analysis

| Query/family | Requested evidence today | Enough to produce all searched facts? | Provider can alter satisfaction? |
|---|---|---|---|
| reverse exact `builtin/TypeRef` | semantic requirement plus `AcquisitionTarget`; current high projection; TypeRef coverage units | yes for certified units; explicitly lower-bound for uncertified languages | no proof satisfaction; provider only supplies additive candidates |
| reverse Angular `Service` | `Semantic(High)` only | fidelity is sufficient per acquired owner, but owner-universe evidence is absent | yes; ready success suppresses fallback |
| unfiltered `find_entities` | `LegacyEdit` declaration discovery | sufficient only for discovered owners; no occurrence-universe proof | yes; ready subset becomes completed root |
| EF/ASP.NET query refinements | usually legacy discovery plus framework preparation | producer evidence may be sufficient after owners arrive; universe proof is not generic | potentially, even though reported controls were stable |
| `entities_in_file` | explicit file owner | yes for that file; no name-based discovery needed | no relevant provider branch |

The affected queries do not request enough evidence to prove acquisition of every owner whose facts they later search.

## 10. CoverageUnit/proof analysis

**STATICALLY-PROVEN:** Current `CoverageUnit` identity is provider-neutral and includes root, language, producer generation, producer capability, and relationship families.

**STATICALLY-PROVEN:** The only producer capability is incoming builtin TypeRef. Consequently:

- no Angular `Injects` unit exists;
- no generic declaration/occurrence unit exists;
- no residual unit can remain for those requirements;
- their coverage projection cannot explain source-universe proof state;
- the discovery completion cache remains their effective completion authority.

The repair should generalize acquisition obligations only as far as necessary: introduce requirement-specific units/predicates for migrated query families, or conservatively require Clean root acquisition when no sound proof contract exists. Merely setting the existing TypeRef target for Angular would misdescribe the producer and relationship family.

## 11. Provider-hint/fallback analysis

CBM correctly remains advisory at the fact boundary: only file paths cross from search, and `filter_inbound_reference_row` drops `DEFINES` and `DEFINES_METHOD`. The field's one `DEFINES` row therefore statically maps to an empty inbound candidate set.

The defect is fallback selection:

- `Ok(empty)` is treated as completed discovery for a ready root;
- `Ok(subset)` is treated identically;
- the code has no coverage proof associated with either result;
- filesystem fallback is selected by provider success rather than residual semantic obligation.

An empty-only patch is insufficient because O7 is a non-empty subset case. The repair must address all unproved provider results.

## 12. Evidence-adequacy/reuse analysis

| Evidence/cache | Key/dependencies | Provider state dependency | Finding |
|---|---|---|---|
| semantic projection currentness | canonical owner, source hash, semantic fidelity, dependencies | none | sound for owners already published |
| compilation artifact | owner, source hash, fidelity, producer generations, semantic dependencies | none | provider-neutral |
| request preparation cache | inbound flag, name, fidelity requirement, optional acquisition target | indirect through first hydration | request-local reuse of same result is expected |
| discovery completion cache | provider scope, root, mode, name, optional fidelity, scope generation | provider is explicit | mechanically valid but semantically inadequate completion may be stored |
| acquisition completion cache | exact target, roots, admission policy, producer generations, predicate/contract versions, evidence, root generation | none | commits only source-complete satisfaction |

There is no evidence that a provider path marks a stale artifact current. Instead, the provider path prevents absent owners from reaching adequacy evaluation at all.

## 13. Publication analysis

`compile_candidates_for` validates paths, reads current source, computes source hash, selects fidelity, compiles retained semantic artifacts, checks for source races, and commits deterministically. `commit_candidate` replaces the owner projection and dependencies under a publication ticket.

**STATICALLY-PROVEN:** Provider state is absent from publication identity and currentness.

**INFERRED FROM FIELD + STATIC PATH:** O2/O6 publish no injector owner files because none were acquired. O7 publishes only the CBM candidate subset. This is under-publication caused upstream, not provider-conditioned publication of equal artifacts.

## 14. Reconciliation/refinement analysis

C# reconciliation and request-local catalogs consume retained/current published evidence. Angular query refinements similarly cannot synthesize missing constructor owners. Publication invalidates request-local derived catalogs when candidates compile.

**STATICALLY-CONSISTENT:** With equal published owners, reconciliation is provider-neutral. No branch in the inspected reconciliation/refinement path selects behavior based on CBM readiness.

**UNPROVEN:** Every framework-specific reconciliation path in the repository was not exhaustively audited because one earlier shared cause already explains all reported observations. The final repair audit must nevertheless scan for provider-specific bypasses.

## 15. Cache/generation analysis

The discovery cache is not stale in the ordinary sense. Its provider-specific generation and key match what it recorded. The semantic defect is that `mark_complete_for` is called after an acquisition that was never adequate for Clean truth.

This creates history dependence:

- cold ready-provider query records inadequate completion;
- warm identical query legitimately sees the entry and skips discovery;
- `find_entities` cannot satisfy inbound mode because modes differ;
- exact TypeRef filesystem acquisition incidentally publishes owners, allowing the later Angular query to succeed even though its own discovery remains inadequate.

The acquisition completion cache does not share this defect; it refuses incomplete satisfaction. The two caches encode different meanings that are conflated by the legacy route.

## 16. Forward-edge vs reverse-index analysis

`WorkspaceIndex` inserts each `StoredEdge` into both subject-keyed forward and object-keyed reverse adjacency. Reverse lookup selects the stored incoming occurrences; it does not reacquire or reinterpret relationships.

Smallest separating assertion:

1. after preparation, assert the expected `Injects` edge exists in forward adjacency from each consumer;
2. assert the identical edge exists in reverse adjacency for the service;
3. if both are absent, acquisition/publication failed;
4. if forward exists and reverse does not, index/retrieval failed.

Current field behavior after a TypeRef warm-up returns the reverse edges, which is consistent with an upstream acquisition defect. No evidence indicates forward-only publication.

## 17. Coverage/completeness analysis

O2/O3 truthfully report `identity_not_indexed`, `source_complete: false`, `lower_bound`, and `omitted_possible: true`. That prevents a formal exactness claim, but it does not satisfy provider independence.

The sparse discovery projection hides the decisive event because ready searched projects are treated as expected and omitted; zero discovered/compiled counts are also omitted. A caller cannot tell “provider returned no usable inbound paths and no Clean scan ran” from an ordinary healthy no-work outcome.

Diagnostics should expose an unproved-provider/no-Clean-acquisition reason after correctness is repaired, but diagnostics alone are not a repair. Lower-bound honesty does not authorize avoidable provider-induced under-acquisition.

## 18. Cross-producer/framework/language shared-path analysis

```text
                     workspace_query
                           |
          +----------------+----------------+
          |                                 |
   Declaration mode                  InboundReference mode
 find/forward/transitive                 reverse_edges
          |                                 |
          +------ legacy hydrate -----------+
                         |
               ready provider Ok(paths)
                         |
        [DEFECT] root marked successful without
             requirement-compatible proof
                         |
              filesystem root suppressed
                         |
             provider subset compiled
                         |
       Clean semantic producers (TS/C#/frameworks)
                         |
        owner projection + forward/reverse index
                         |
       smaller provider-dependent lower bound
                         |
          inadequate discovery completion cached

 exact builtin/TypeRef reverse
          -> target + CoverageUnits + Clean inventory/proof
          -> filesystem acquisition retained
          -> provider-neutral result
```

Shared machinery explains TypeScript Angular and C# occurrence divergence. Stable EF/ASP.NET controls show only that their particular owners were already acquired or returned by the provider in the baseline; they do not prove immunity.

## 19. Why #144 regressions missed #148

Every regression in `workspace_query_reverse_edge_completeness.rs` dispatches an exact reverse `builtin/TypeRef` query. Therefore every one sets `AcquisitionTarget::exact_type_ref`, making `requires_provider_neutral_proof = true`.

| #144 regression | What it proves | What #148 adds |
|---|---|---|
| ready subset cannot omit consumer | exact TypeRef provider subset cannot suppress Clean scan | Angular reverse and declaration discovery never enter this route |
| ready empty has no negative authority | exact TypeRef `Ok(empty)` still scans | legacy `Ok(empty)` is marked complete |
| warm request cannot cache subset | exact acquisition/discovery remains semantically complete across reuse | legacy discovery cache records inadequate completion |
| proof-backed reverse earns exact coverage | certified C# TypeRef units can become exact after publication | no Angular/declaration capability unit exists |
| cold/warm exact detail | acquisition completion cache preserves exact proof | affected cache is the separate discovery completion cache |
| complete zero is exact | verified Clean inventory can prove absence | CBM-empty is not such an inventory |
| provider unavailable detail | Clean proof truth is provider-neutral on exact route | unavailable fallback and ready legacy route select different owner universes |
| singular/batch parity | identical exact preparation preserves result/detail | no test crosses legacy provider modes/families |
| unproved language remains lower bound | TypeScript unit cannot claim exact TypeRef proof | lower bound can still be unnecessarily smaller under ready CBM |
| publication/source-race failures | failed candidate publication cannot earn proof | #148 candidates are never selected, so failure logic is never reached |
| unscoped cannot claim workspace completeness | missing scope blocks exact certificate | #148 is scoped but lacks a requirement-compatible certificate |

The missing topology is not merely “Angular coverage.” It is the transition from ready provider success to `successful_roots`/discovery completion on a hydration request without an `AcquisitionTarget`, across both `InboundReference` and `Declaration` modes.

## 20. One-defect vs multi-defect determination

| Observation cluster | First divergence | Cause |
|---|---|---|
| O2/O3/O6 Angular zeros | ready inbound provider `Ok(empty)` marks root successful and complete | shared unproved provider completion |
| O4 declaration warm-up does not help | discovery mode is part of cache/acquisition identity | expected mode separation plus same underlying defect |
| O5 exact TypeRef repairs later Angular result | exact route retains Clean scan and publishes the omitted owners | demonstrates routing asymmetry |
| O7 `find_entities` 7 vs 5 | ready declaration provider subset marks roots successful | same defect in Declaration mode |
| O8–O11 stable controls | no observed semantic divergence | owners were adequately present or operation bypasses name hydration |
| O12 one `DEFINES` row | filter discards definition-only row | explains empty advisory set, not the authority defect itself |

One shared defect is sufficient. A separate diagnostics opacity exists, but it is an observability gap, not the cause of semantic divergence.

## 21. C# R0–R6 impact assessment

### Decision

`CSharpImpact::SharedLifecycleRepairRequired`

R0–R5 neutral syntax, identities, source symbols, imports, aliases, and ancestry are not falsified. R6's resolved EF role authority is not falsified. The affected assumption is that required source owners reach the shared compilation/publication lifecycle independently of optional provider state.

R6 must be reverified after repair because:

- R6 durable role publication begins only after owner acquisition;
- cross-file ancestry requires all chain owners;
- request-local EF projection consumes the published durable role and admitted sources;
- O7 proves C# source occurrences can currently differ by provider state;
- stable field `HasEntity` results are positive controls, not a proof that every R6 topology is immune.

No R0–R6 production change is authorized in this investigation.

## 22. R7 authorization recommendation

R7 remains blocked. Beginning AutoMapper migration while shared acquisition can omit source owners would make Profile correctness and lifecycle results provider-state dependent and confound migration evidence.

Resume only after:

1. the shared root-satisfaction repair is approved and implemented;
2. provider-equivalence REDs are GREEN;
3. R6 focused role, relationship, cross-file, lifecycle, warm/cold, cross-root, and work-count gates are rerun.

## 23. Smallest deterministic RED design

### RED A — legacy inbound Angular acquisition

One synthetic TypeScript workspace:

- `ServiceX` declaration;
- two constructor consumers publishing `Injects`;
- provider harness states: unavailable, ready-empty, ready-strict-subset, ready-all-useful;
- identical exact `reverse_edges angular/Service/ServiceX` query.

Assert for every state:

- the same two forward `Injects` occurrences after preparation;
- the same two reverse results;
- identical source attribution/cardinality;
- repeated warm result equals cold result;
- an unproved provider result does not earn root completion;
- coverage remains truthful and provider-neutral.

### RED B — declaration/occurrence acquisition

One small mixed or C# workspace with a declaration and two TypeRef occurrences sharing the queried name. Run unfiltered `find_entities` under the same four provider states. Provider subset returns the declaration but omits both consumers.

Assert identical occurrence sets, publication ownership, cold/warm behavior, and no discovery completion from the unproved subset.

### RED C — forward/reverse separator

After each RED A preparation, inspect the index: forward and reverse occurrence sets must be identical projections of the same stored edge generation. This localizes any later failure beyond acquisition.

Do not duplicate the entire #144 certificate suite. Reuse its provider harness and add the missing no-target mode/requirement transition.

## 24. Repair-boundary recommendation

Repair the shared decision that converts provider search into root satisfaction.

Recommended contract:

```text
provider Ok(paths)
    -> additive candidate hints
    -> never removes a required Clean root by itself

root satisfied
    iff requirement-compatible proof covers it
    or a Clean authoritative acquisition for that requirement completed
```

Implementation design must:

- replace the boolean `requires_provider_neutral_proof` with an explicit acquisition requirement/disposition;
- distinguish advisory provider completion from authoritative Clean root satisfaction;
- prevent `successful_roots` and reusable completion from being earned by unproved provider empty/subset results;
- conservatively scan when a query family lacks a certified selective proof;
- extend capability-specific proof only where a sound predicate and producer generation exist;
- retain provider paths as additive candidates and avoid recompiling current owners;
- add an explicit diagnostic for provider hints not establishing Clean coverage.

Do not implement an empty-result-only exception. Do not globally recompile everything: filesystem inventory/candidate selection may scan, while current source/fidelity/dependency checks still skip compatible owners.

## 25. Performance implications

The repair will make some ready-provider legacy queries pay Clean discovery when no compatible coverage proof exists. That is required work, not a regression.

Preserve earned performance by:

- one admitted-source inventory per relevant root/generation;
- capability-scoped predicates and reverse dependencies;
- provider hints unioned with Clean-selected candidates;
- compilation only for absent/stale/inadequate owners;
- deterministic bounded parallel compilation already present;
- completion cache only after semantic publication and proof;
- warm reuse keyed by source/admission/producer/capability generation;
- no provider-state key in semantic truth compatibility.

The current exact TypeRef architecture demonstrates this shape. It should be generalized deliberately, not bypassed with unconditional global recompilation.

## 26. Architectural invariant assessment

| Candidate invariant | Assessment |
|---|---|
| PROVIDER-148-001 | required; violated by current legacy root completion |
| PROVIDER-148-002 | required; exact proof path enforces it, legacy path does not |
| PROVIDER-148-003 | required; provider currently narrows without retaining a path to omitted owners |
| PROVIDER-148-004 | required; directly falsified by O1/O2/O6/O7 |
| PROVIDER-148-005 | required; semantic artifact caches comply, discovery completion does not |
| PROVIDER-148-006 | required; violated when ready `Ok` suppresses unresolved acquisition |
| PROVIDER-148-007 | required; current adjacency architecture satisfies it after publication |
| PROVIDER-148-008 | required; O2 is a truthful but avoidably under-acquired lower bound |

Add these to authoritative invariant documentation only when their enforcement lands. PROVIDER-148-007 may cite existing dual-adjacency storage; the others require the repair and regressions.

## 27. Runtime evidence still required

No runtime evidence is required for the causal verdict. The field matrix supplies observed behavior, and current source statically proves the branch that creates it.

Runtime evidence will be required after an approved repair:

- deterministic RED/GREEN provider-state matrix for both missing modes;
- existing #144 focused suite;
- R6 focused suite and performance work counts;
- final all-feature verification gate;
- one consolidated live Issue #148 provider-state acceptance only after local gates, using owned synthetic fixtures rather than proprietary source.

No Cargo, build, test, Clippy, binary, MCP, CBM, provider, or live command was run for this investigation.

## 28. Explicit non-goals

- No production or test implementation.
- No modification of R0–R6.
- No R7 work.
- No proprietary workspace request or reconstruction.
- No CBM semantic authority.
- No assumption that CBM can supply `AcquisitionCoverageProof`.
- No global recompilation default.
- No weakening of completeness or lower-bound semantics.
- No caps, timeouts, or truncation changes.
- No framework producer rewrite.
- No claim of compiler-grade source-universe binding.
- No GitHub mutation.

## 29. Final recommendation

Approve a bounded Issue #148 repair design at the shared hydration root-satisfaction boundary. Provider results remain additive hints. A root leaves residual Clean acquisition only through requirement-compatible proof or completed Clean acquisition. Discovery completion must inherit that authority decision rather than create a parallel one.

After the repair, run the two provider-equivalence regressions, the existing #144 suite, and R6 reverification. Only then authorize R7.

**ISSUE-148-CAUSE:**  
SHARED-ACQUISITION-PROOF-LIFECYCLE — PRIMARY  
CACHE-GENERATION-COMPATIBILITY — SECONDARY AMPLIFIER (INADEQUATE DISCOVERY COMPLETION IS REUSED)

**ISSUE-148-VS-144:**  
#144 REPAIR IS INCOMPLETE AT ITS INTENDED BOUNDARY — THE PROOF CONTRACT HOLDS FOR EXACT INCOMING `builtin/TypeRef`, BUT LEGACY DECLARATION AND NON-TypeRef INBOUND ROUTES BYPASS IT

**PROVIDER-INDEPENDENCE:**  
VIOLATED

**CSHARP-CAMPAIGN-IMPACT:**  
SHARED LIFECYCLE REPAIR REQUIRED — PRESERVE R0-R6, REPAIR SHARED BOUNDARY, REVERIFY R6

**R7:**  
BLOCKED — REPAIR #148 SHARED BOUNDARY AND REVERIFY R6 FIRST

**#144-REGRESSION-GAP:**  
NO TEST CROSSED READY EMPTY/SUBSET PROVIDER SUCCESS INTO `successful_roots` AND DISCOVERY COMPLETION FOR A NO-`AcquisitionTarget` HYDRATION REQUEST; EXACT-TypeRef TESTS ALWAYS ACTIVATED THE PROOF ROUTE, LEAVING BOTH LEGACY `InboundReference` AND `Declaration` MODES UNTESTED

**REPAIR-BOUNDARY:**  
THE SHARED ROOT-SATISFACTION DECISION IN HYDRATION: READY PROVIDER SUCCESS MAY ADD CANDIDATES BUT MAY NOT SUPPRESS CLEAN FALLBACK OR EARN REUSABLE COMPLETION WITHOUT REQUIREMENT-COMPATIBLE COVERAGE PROOF

1. **First causal divergence:** ready provider `Ok(paths)` becomes `successful_roots` for no-target hydration, before filesystem acquisition and semantic compilation.
2. **Observation mapping:** Angular empty/subset and C# occurrence subset map to the same branch; mode separation explains history effects; stable controls either use the proof route, bypass name hydration, or already have adequate owners.
3. **Why #144 missed it:** every #144 provider-neutral regression used exact reverse `builtin/TypeRef`, the only query route that supplies an acquisition target and prevents provider success from satisfying the root.
4. **Smallest REDs:** one Angular inbound provider matrix and one declaration/occurrence provider matrix, each asserting forward publication, reverse/query results, coverage, and cold/warm equivalence.
5. **Smallest repair:** replace provider-success root completion with requirement/proof-based root satisfaction; retain provider paths as hints.
6. **R0–R6 changes:** no architecture change required; preserve them.
7. **R6 reverification:** required because shared source acquisition/publication feeds its durable and request-local semantics.
8. **R7 status:** blocked until repair and R6 reverification are GREEN.
