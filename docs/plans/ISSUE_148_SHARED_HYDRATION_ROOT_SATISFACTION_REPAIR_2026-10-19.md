# Issue #148 Shared Hydration Root-Satisfaction Repair

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Authoritative branch:** `(fix)Performance-Regression-Issues`  
**Mode:** RED -> implementation -> operator-run focused GREEN  
**Runtime commands executed by agent:** none

## 1. Executive result

The shared hydration boundary now treats provider search success as advisory candidate discovery only. Ready provider results--empty, partial, or apparently complete--cannot remove a configured root from Clean-owned filesystem acquisition. Provider-search reuse and authoritative filesystem completion are represented by distinct provider-scoped discovery-cache entries.

The previous `target.is_some()` authority discriminator was replaced with an explicit `HydrationAuthority`: ordinary queries require `CleanAuthoritative` acquisition, while exact incoming `builtin/TypeRef` requests retain `ProofBacked(AcquisitionTarget)` and the existing #144 proof machinery.

Deterministic tests cover Angular inbound equivalence, declaration/occurrence equivalence, forward/reverse parity, completion authority, cold/warm reuse, and all four provider states. Runtime validation remains operator-owned and has not been claimed.

## 2. Pre-edit authority map

| Symbol | Location | Previous authority meaning | Repair |
|---|---|---|---|
| `successful_roots` | `hydration.rs::discover_candidate_paths` | ready provider success removed a root from filesystem fallback | removed; provider success never satisfies a root |
| `requires_provider_neutral_proof` | `hydration.rs`, supplied by `execution.rs` | boolean exception to legacy provider authority | replaced by explicit `HydrationAuthority` |
| `target.is_some()` | `execution.rs::hydrate_workspace_index_for_target` | selected authoritative versus legacy acquisition semantics | removed as authority decision |
| provider `completions` | provider-success branches in `discover_candidate_paths` | queued provider search as reusable discovery completion and indirectly root satisfaction | narrowed to advisory `provider_completions` |
| filesystem `completions` | post-scan branch | recorded completed Clean scan | separated as `root_completions` |
| `mark_complete_for` | `discovery_cache.rs`, called through `hydration/cache.rs` | cached exact provider scope but callers conflated CBM completion with root completion | retained; comments and call-site separation now make provider scope advisory only |
| `discovery_is_complete` | CBM loop and filesystem pending-root selection | CBM cache hit could skip provider work and suppress fallback through `successful_roots` | CBM hit skips only CBM search; filesystem scope is checked independently |
| `AcquisitionTarget` | exact incoming TypeRef preparation/execution | identifies the #144 selective proof obligation | preserved inside `ProofBacked` |
| `AcquisitionRequirement` | `mcp/acquisition` | enumerates exact TypeRef coverage units | unchanged |
| `AcquisitionCompletionCache` | `McpState` and exact acquisition execution | caches only compatible source-complete #144 satisfaction | unchanged |

The provider-success writes that formerly removed roots and queued authoritative-looking completion occurred in both the test-injected and live CBM `Ok(...)` branches. The CBM discovery-cache read then allowed later requests to skip provider discovery and, through `successful_roots`, Clean fallback. All three transitions were changed at their shared boundary.

## 3. RED A design and expected pre-fix failure

`issue_148_angular_provider_states_converge_and_completion_requires_clean_acquisition` creates `ServiceX`, `ConsumerA`, and `ConsumerB`, then runs identical reverse `angular/Service/ServiceX` queries with provider unavailable, ready-empty, ready-subset, and ready-useful hints.

Before the repair, ready-empty or ready-subset could publish zero or one injector owner because provider success suppressed the root scan. The regression requires the same two attributed `Injects` edges in every state, cold/warm equality, one Clean scan, provider-search reuse, and truthful lower-bound coverage for this unsupported proof family.

## 4. RED B design and expected pre-fix failure

`issue_148_declaration_provider_states_cannot_shrink_occurrence_universe` creates `ContextA` and two independent constructor TypeRef owners. It runs unfiltered `find_entities ContextA` under the same four provider states.

Before the repair, a provider subset containing only `ContextA.cs` could suppress acquisition of both occurrence owners. The regression compares the complete domain/type/file occurrence set across provider states and cold/warm requests. It also proves provider-search reuse and separate Clean completion.

## 5. RED C separator

The Angular regression reads the existing `WorkspaceIndex` after preparation. It compares both consumer-forward `Injects` edges with the reverse `ServiceX` edges as identical subject/object pairs. This separates acquisition/publication failure from an adjacency defect without adding a public API.

## 6. RED D completion-authority test

The four-state matrices cover ready-empty, ready-subset, and ready-useful provider results. Each ready case must still perform exactly one filesystem scan and ends with two distinct cache entries: one advisory CBM scope and one authoritative filesystem scope. The warm repeat performs no semantic recompilation. Existing provider-readiness and discovery-cache regressions were updated to assert the same separation directly.

## 7. RED E #144 preservation

The exact incoming TypeRef entry point still constructs `AcquisitionTarget`, enters `HydrationAuthority::ProofBacked`, builds Clean inventory/provisional satisfaction, crosses the publication barrier, and commits only compatible `AcquisitionSatisfaction`. Existing `issue_144_*` regressions remain the focused preservation authority, including ready subset, empty, residual acquisition, exact coverage, and cold/warm equivalence.

## 8. Production changes

- Added explicit `HydrationAuthority::{CleanAuthoritative, ProofBacked}`.
- Removed provider-derived `successful_roots` and the boolean proof discriminator.
- Kept provider paths as additive candidates.
- Always sends configured roots through the independently cached filesystem acquisition scope.
- Split advisory provider completion from authoritative root completion.
- Commits either completion only after candidate processing and publication succeed.
- Included authority in request-local hydration cache identity.
- Clarified discovery-cache ownership comments.

## 9. Root-satisfaction model before/after

Before:

```text
ready provider Ok(paths) -> successful_roots -> filesystem root removed
```

After:

```text
provider Ok(paths) -> additive hints + provider-search reuse
configured root -> proof-backed obligation or Clean filesystem acquisition
Clean scan/publication succeeds -> filesystem completion may be recorded
```

## 10. Discovery completion/cache changes

The cache type remains provider-scoped. Its meanings are now explicit:

- CBM scope: that advisory provider search need not repeat for the current generation;
- filesystem scope: Clean-owned root acquisition completed for the current generation.

A CBM cache hit sets provider diagnostics and skips only provider work. It cannot satisfy the separately queried filesystem scope. Root invalidation continues to invalidate both scopes.

## 11. Provider hint semantics

Validated provider paths are still unioned into candidate selection. They may accelerate discovery and current-owner reuse, but their cardinality carries no negative or completeness authority. Empty, subset, and full-looking results have identical root-satisfaction semantics.

## 12. Clean fallback semantics

For every query family without a compatible selective proof contract, configured roots remain eligible for Clean filesystem discovery. The scan finds literal candidates; existing source hash, fidelity, dependency adequacy, and deterministic publication logic still selects only absent, stale, or inadequate owners for compilation.

## 13. Exact-TypeRef proof-route preservation

No #144 acquisition type, coverage unit, predicate, proof barrier, satisfaction, or completion-cache contract changed. `ProofBacked` makes the route explicit rather than deriving it from optional-target presence. Its residual Clean acquisition remains intact; the repair does not turn it into unconditional semantic recompilation.

## 14. Downstream components intentionally unchanged

Angular extraction, C# TypeRef extraction, EF classification/projection, ASP.NET, AutoMapper/SignalR design, `WorkspaceIndex` forward/reverse insertion, query filtering, and serialized response schemas were not changed. R0-R6 were not rewritten and R7 was not started.

## 15. Performance-preservation analysis

The regressions assert one Clean scan on the cold request, one provider search for each ready state, zero semantic recompilation on the warm repeat, and stable semantic truth. Compatible artifacts still flow through normal currentness checks. The repair adds required inventory/candidate discovery where provider success previously deleted required work; it does not force blanket recompilation.

## 16. Diagnostics changes

No public schema changed. Existing `provider`, `fallback_reason`, discovered/compiled counts, and coverage fields now expose combined provider-plus-filesystem work. Warm cache reuse reports provider participation consistently. Unsupported proof families remain truthful lower bounds even though avoidable provider-induced omissions are repaired.

## 17. Static post-repair bypass audit

Static search found no remaining `successful_roots` or `requires_provider_neutral_proof`. Both provider `Ok(...)` branches can only append candidates and queue advisory provider scopes. Every configured root is added to fallback before `pending_discovery_roots` independently checks its filesystem scope. Only completed filesystem scanning queues `root_completions`. Exact proof satisfaction is still committed exclusively through `AcquisitionCompletionCache` after the semantic publication barrier.

## 18. Files changed

- `src/mcp/tool_handlers/hydration.rs`
- `src/mcp/tool_handlers/hydration/execution.rs`
- `src/mcp/tool_handlers/hydration/cache.rs`
- `src/mcp/tool_handlers/query/prepare.rs`
- `src/mcp/discovery_cache.rs`
- `src/mcp/tool_handlers/query.rs`
- `src/tests/mcp/workspace_query_provider_equivalence.rs`
- `src/tests/mcp/workspace_query_provider_readiness.rs`
- `src/tests/mcp/workspace_query_6.rs`
- `src/tests/mcp/workspace_query_8.rs`
- this report

## 19. Focused commands awaiting authorization

No command below was executed by Agent MaxHeadRoom. Run them in order and return the first failure unchanged.

```powershell
cargo test --all-features issue_148_
cargo test --all-features issue_144_
cargo test --all-features ready_cbm_candidates_do_not_suppress_authoritative_filesystem_acquisition
cargo test --all-features red_f6_healthy_cbm_zero_retains_authoritative_filesystem_acquisition
cargo test --all-features red_f7_healthy_cbm_cannot_suppress_authoritative_filesystem_acquisition
cargo test --all-features red_h
cargo clippy --all-targets --all-features -- -D warnings
```

## 20. R6 reverification obligations

After the #148 focused gate is GREEN, rerun the R6 role, relationship, cross-file ancestry, lifecycle, cross-root, warm/cold, and work-count gates. That validation is separate because the shared acquisition repair feeds R6 owners even though R6 resolution architecture did not change.

## 21. R7 status

R7 remains blocked until the #148 focused gate and separate R6 reverification are GREEN.

## 22. Residual risks

- Runtime compilation, formatting, Clippy, and behavior remain unverified until the operator runs the focused commands.
- Unsupported semantic families still report lower-bound coverage; this repair prevents optional provider state from unnecessarily shrinking that lower bound but does not invent exact proof.
- Clean scanning is newly required on provider-ready legacy paths. Existing currentness checks bound recompilation, but operator work-count evidence must confirm the intended cold/warm behavior.

## 23. Final verdict

The bounded production repair, deterministic regression coverage, and static bypass audit are complete. Validation is intentionally pending the operator-run gate.

ISSUE-148-IMPLEMENTATION:  
COMPLETE

ROOT-SATISFACTION-AUTHORITY:  
PROVIDER SUCCESS IS ADVISORY ONLY

EMPTY-PROVIDER:  
CANNOT SUPPRESS REQUIRED CLEAN ACQUISITION

PARTIAL-PROVIDER:  
CANNOT SUPPRESS REQUIRED CLEAN ACQUISITION

UNPROVED-FULL-LOOKING-PROVIDER:  
CANNOT SUPPRESS REQUIRED CLEAN ACQUISITION

DISCOVERY-COMPLETION:  
CANNOT EARN AUTHORITATIVE COMPLETION FROM PROVIDER SUCCESS ALONE

#144-PROOF-ROUTE:  
PRESERVED

DOWNSTREAM-SEMANTICS:  
UNCHANGED

R0-R6:  
PRESERVED

R6-REVERIFICATION:  
REQUIRED AFTER #148 FOCUSED GREEN

R7:  
BLOCKED PENDING #148 FOCUSED GREEN AND R6 REVERIFICATION

COMMANDS-AWAITING-OPERATOR-AUTHORIZATION:

1. `cargo test --all-features issue_148_`
2. `cargo test --all-features issue_144_`
3. `cargo test --all-features ready_cbm_candidates_do_not_suppress_authoritative_filesystem_acquisition`
4. `cargo test --all-features red_f6_healthy_cbm_zero_retains_authoritative_filesystem_acquisition`
5. `cargo test --all-features red_f7_healthy_cbm_cannot_suppress_authoritative_filesystem_acquisition`
6. `cargo test --all-features red_h`
7. `cargo clippy --all-targets --all-features -- -D warnings`
