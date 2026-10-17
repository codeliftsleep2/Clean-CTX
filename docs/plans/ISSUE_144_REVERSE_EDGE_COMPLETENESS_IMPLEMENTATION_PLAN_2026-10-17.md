# Agent MaxHeadRoom — Issue #144 Reverse-Edge Completeness Implementation Plan

**Date:** 2026-10-17

**Phase:** approved architecture translated to implementation plan

**Production implementation:** authorized and implemented through Phase 6;
Phase 7 final verification pending

**Implemented GREEN baselines:**

- `14eda02b` — provider-neutral acquisition authority (Phases 1–3)
- `00e6aa44` — Clean acquisition proof (Phase 4)
- `42f320a0` — proof-backed completion cache (Phase 5)
- `a23ca0a4` — earned exact reverse TypeRef coverage (Phase 6)

**Approved design:**
`docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_DESIGN_1_2026-10-17.md`

## 1. Executive plan

Implement the provider-neutral completeness design incrementally, preserving
lower-bound public behavior until the complete production lifecycle can earn an
exact result.

The current code audit resolves the most important deferred decision:

> **Current CBM cannot supply an authoritative acquisition coverage proof.**

Its Clean-CTX boundary exposes readiness, project/root mapping, search/query
rows, observed candidate paths, and graph results. It does not expose a complete
admitted-source inventory fingerprint, a source snapshot generation bound to
that inventory, producer/schema coverage, exhaustive relationship-family
coverage, or pagination/truncation attestation. `IndexingStatus::Ready`,
`QueryRows.total`, and successful `query_graph` execution cannot be composed
into those missing facts.

Therefore the first implementation must:

1. introduce provider-neutral requirements, coverage units, proof claims, and
   satisfaction types;
2. treat current CBM output as hints only;
3. build Clean-owned proof from an enumerated admitted-source manifest;
4. acquire only residual unproved units;
5. preserve adequate-current semantic evidence reuse;
6. replace provider-specific discovery completion with proof-backed unit
   satisfaction; and
7. keep `reverse_edges` lower-bound until every requirement, evidence,
   reconciliation, freeze, and exposure gate is integrated.

This initial correctness migration may run Clean acquisition for every unit
because CBM currently proves none. That is a temporary consequence of an honest
provider contract, not the target architecture. A later CBM protocol slice may
restore substitution unit by unit once CBM can provide verifiable proof.

## 2. Fixed architectural invariant

> **Semantic completeness is earned by complete proof coverage of the query's
> required coverage units; neither provider readiness nor successful discovery
> completion can substitute for that proof.**

A `CoverageUnit` contains only semantic acquisition dimensions:

```text
canonical root
× source kind/language
× semantic producer capability/version
× relationship-family subset
```

It contains no CBM identity, provider kind, readiness, search status, or
acquisition strategy. Providers and Clean-owned acquisition are independent
proof producers. The completeness layer evaluates validated proofs.

## 3. Deferred-decision resolution

### 3.1 CBM inventory fingerprint

**Decision:** current CBM supplies none; do not synthesize one.

Inspected boundaries:

- `src/cbm/project_search.rs::inbound_reference_paths_in_project` returns only
  caller paths after relationship filtering.
- `src/cbm/client.rs::QueryRows` carries query columns, rows, and a result total;
  the total is not a root inventory or source-generation identity.
- `src/cbm/bridge/indexing.rs::ensure_indexed_for` returns readiness only.
- `src/cbm/bridge.rs::configured_projects` / project-path maps establish routing,
  not content equivalence.
- candidate-path observation validates returned paths but cannot validate absent
  paths.

Initial adapter behavior is therefore:

```text
CBM search success -> candidate hints + no proof claim
CBM Ready          -> lifecycle state only
CBM empty result   -> empty hint set + no negative authority
```

A future CBM proof-capable protocol must expose, atomically or by a verifiable
binding:

- canonical root inventory digest;
- source snapshot/index generation tied to that digest;
- admission/exclusion policy identity;
- language/producer/schema coverage;
- relationship-family/query-semantics version;
- complete pagination/truncation state; and
- the complete candidate owner set for the requirement.

That provider protocol expansion is outside the initial implementation and
requires its own wire-contract tests before CBM may return a proof claim.

### 3.2 Producer-to-relationship mapping

**Decision:** define a narrow direct capability catalog for generic TypeRef
projection; do not add a generic registry.

Current authority already exists in:

- `src/ir/semantic_projection.rs::project_generic_facts`;
- `project_constructor_parameter_types`;
- `project_inheritance`;
- `GENERIC_SEMANTIC_PROJECTION_GENERATION`;
- language canonical/semantic-input generation constants; and
- `src/mcp/compatibility/producer.rs::GenerationCatalog`.

The first capability is conceptually:

```text
BuiltinTypeRefIncoming
  relationships:
    HasConstructorParameterType
    HasBaseType
  object identity:
    builtin / TypeRef / written target
  producer generations:
    SharedCanonicalPipeline
    GenericSemanticProjection
    active language Canonical + SemanticInput generations
    CSharpSemanticProjection when C# struct/record base facts are applicable
```

Implementation must derive generation values from `GenerationCatalog`, not
duplicate constants in hydration/query code. The capability mapping should be
a closed match/function near the new acquisition types. A plugin registry is
not authorized.

### 3.3 Sound candidate predicates

**Decision:** no existing broad predicate is automatically certified. The first
Clean proof enumerates the complete admitted supported-source inventory and
records a manifest. Candidate narrowing is then capability-specific.

For `builtin/TypeRef/<written name>`, the current semantic projection preserves
the target as a written type identity. The first implementation may use literal
name containment only after tracked producer tests establish this implication:

```text
owner publishes HasConstructorParameterType or HasBaseType to TypeRef X
  => the owner's authoritative source snapshot contains the required written
     target token under the same normalization/matching contract
```

The proof must cover qualified types, generic types, nullable/array forms,
aliases, nested types, Unicode identifiers, and every enabled language producer.
If any producer cannot satisfy that implication, its unit must use full
admitted-owner compilation/validation or remain lower-bound. A substring search
alone never proves a unit.

The existing filesystem traversal is useful but must be split conceptually:

- inventory enumeration proves which admitted supported owners exist;
- source reads/hash capture bind the inventory generation;
- a validated capability predicate selects candidates;
- publication proves candidate semantic representation.

### 3.4 Proof storage

**Decision:** store compact immutable proof records plus candidate owner keys in
the session cache. Do not persist proofs in SQLite in the first slice.

The session already owns source cache, workspace index, discovery cache, and
edit/refresh invalidation. Session-local storage avoids designing durable proof
migration before the lifecycle is proven. A proof record contains:

- `CoverageUnit`;
- requirement/target fingerprint;
- scope/admission fingerprint;
- inventory manifest digest;
- producer-generation fingerprint;
- acquisition predicate/proof-contract version;
- canonical candidate-owner set;
- provenance and invalidation dependencies; and
- publication/reconciliation completion state.

The candidate set is retained because a warm cache hit must not depend on
reconstructing provider output. A later measured optimization may replace large
sets with a stable manifest reference; that is not needed for correctness.

### 3.5 Public completeness representation

**Decision:** retain the current coverage schema during structural slices. Add
exact reverse-edge completeness only in the final exposure slice.

When earned, reuse the established fields:

```json
{
  "status": "authoritative_complete",
  "source_complete": true,
  "result_semantics": "exact",
  "omitted_possible": false,
  "scope": { "kind": "workspace", "value": "<canonical scope>" },
  "source_generation": "<proof-set fingerprint>",
  "evidence_requirements": ["incoming_relationship_units_proven", "..."],
  "reasons": []
}
```

Add a compact `covered_capabilities` array only if schema review shows clients
need it to interpret exactness. Provider names remain diagnostics and do not
appear as completeness authority. Partial responses retain existing lower-bound
fields and add structured reasons only when the response schema slice is
explicitly approved.

### 3.6 First exact-completeness scope

**Decision:** target `reverse_edges` for exact `builtin/TypeRef` identities,
across every enabled source-language unit present in the effective roots.

Do not claim exactness for only C# while silently excluding other enabled
languages in the same scope. The requirement expands from actual admitted
inventory source kinds. A root with C# and TypeScript creates both required
units. If TypeScript has no certified strategy, the result remains lower-bound
even when C# is proven.

Angular template relationships, calls, framework-specific identities, forward
edges, and transitive traversal are out of the first exactness scope. Their
coverage units may reuse the architecture later but must not be inferred from
the TypeRef slice.

## 4. Proposed source layout

No file is created by this plan. When implementation is authorized, prefer this
bounded decomposition:

```text
src/mcp/acquisition.rs
src/mcp/acquisition/
  requirement.rs
  proof.rs
  satisfaction.rs
  cache.rs
  clean.rs
  provider.rs

src/tests/mcp/
  acquisition_requirement.rs
  acquisition_proof.rs
  acquisition_cache.rs
  workspace_query_reverse_edge_completeness.rs
```

Exact splitting should keep every active file below 615 lines and avoid empty
abstractions. If the complete implementation remains comfortably small,
`requirement.rs`, `proof.rs`, and `satisfaction.rs` may be combined. Do not
place this under `src/cbm`; core acquisition authority must not depend on CBM.

Existing orchestration remains in `src/mcp/tool_handlers/hydration.rs`, but
material growth should move logic into acquisition modules rather than reactivate
an oversized coordinator.

## 5. Core types and symbols

### 5.1 `requirement.rs`

Add:

```rust
pub(crate) enum AcquisitionRequirement
pub(crate) struct IncomingRelationshipRequirement
pub(crate) struct AcquisitionTarget
pub(crate) struct CoverageUnit
pub(crate) enum ProducerCapability
pub(crate) enum RelationshipFamily
pub(crate) struct EvidenceRequirement
```

Required functions:

```rust
incoming_type_ref_requirement(args, scope, inventory_source_kinds)
required_coverage_units(requirement, roots, source_kinds)
producer_generation_fingerprint(unit, GenerationCatalog)
requirement_fingerprint(requirement)
```

Rules:

- constructors validate nonempty target and canonical root identity;
- `CoverageUnit` has no provider field;
- equality/hash includes semantic producer version and relationship family;
- requirements use a deterministic ordered representation before hashing;
- unsupported source kinds produce explicit unsatisfied units, not omission.

### 5.2 `proof.rs`

Add:

```rust
pub(crate) struct AcquisitionProofClaim
pub(crate) struct ValidatedAcquisitionProof
pub(crate) struct InventoryManifest
pub(crate) struct InventoryEntry
pub(crate) struct ScopePolicyFingerprint
pub(crate) enum ProofProvenance
pub(crate) enum ProofRejection
```

Required functions:

```rust
validate_proof_claim(requirement, unit, current_context, claim)
inventory_manifest_digest(entries)
scope_policy_fingerprint(config, workspace_scope)
proof_set_generation(validated_proofs)
```

Validation must fail closed on every mismatch: root, source kind, producer
generation, relationship family, target semantics, inventory generation,
scope/admission policy, truncation, limitation, or proof version.

`ProofProvenance` identifies Clean or provider origin for diagnostics and
invalidation only. It is absent from `CoverageUnit` equality.

### 5.3 `satisfaction.rs`

Add:

```rust
pub(crate) enum UnitSatisfaction
pub(crate) struct AcquisitionSatisfaction
pub(crate) enum IncompletenessReason
```

Required functions:

```rust
record_validated_proof(unit, proof)
residual_units(requirement)
is_source_complete()
incompleteness_reasons()
```

`is_source_complete` is a total conjunctive check over the requirement's exact
unit set. It must not inspect provider readiness, search success, candidate
count, or nonempty edges.

### 5.4 `provider.rs`

Add the smallest provider-neutral boundary:

```rust
pub(crate) struct ProviderAcquisition {
    hints: Vec<String>,
    proof_claims: Vec<AcquisitionProofClaim>,
}
```

Prefer a direct function/interface used by hydration over a provider registry.
The CBM adapter initially returns `proof_claims: Vec::new()`.

Modify:

- `src/cbm/project_search.rs`: preserve current search behavior; expose results
  through the provider-acquisition adapter without constructing proof.
- `src/mcp/tool_handlers/hydration/test_support.rs`: inject hints and explicit
  synthetic proof claims independently. Tests must never infer proof from Ready.

### 5.5 `clean.rs`

Add Clean-owned acquisition:

```rust
enumerate_inventory(root, scope_policy) -> InventoryManifest
acquire_unit(requirement, unit, manifest) -> CleanAcquisition
validate_type_ref_candidate_predicate(...)
```

`InventoryEntry` should include canonical semantic owner, source kind, and
content hash from the exact source snapshot used for candidate evaluation.
Sort and deduplicate before hashing. Hard-link aliases collapse through
`semantic_owner_path` so one physical semantic owner cannot appear twice.

Traversal must reuse current built-in exclusions, gitignore/config exclusions,
supported-language dispatch, canonical root handling, and read-failure behavior.
Extract those policies from `hydration/filesystem.rs`; do not duplicate them.

The returned Clean proof remains provisional until candidate evidence
publication and reconciliation commit successfully.

### 5.6 `cache.rs`

Replace or migrate `HydrationDiscoveryCache` toward:

```rust
pub(crate) struct AcquisitionCompletionCache
pub(crate) struct AcquisitionCacheKey
pub(crate) struct AcquisitionCacheEntry
```

The key binds:

- provider-neutral coverage unit;
- target/requirement fingerprint;
- canonical scope/admission fingerprint;
- inventory generation/digest;
- producer-generation fingerprint;
- candidate-predicate version;
- proof-contract version; and
- evidence requirement.

The entry owns the validated proof, candidate owners, and publication-complete
state. Provider provenance is value metadata, not key identity, except where
needed as an invalidation dependency.

Do not delete the old cache in the first structural slice. Migrate callers and
tests, then remove `DiscoveryProvider`, provider-bearing `DiscoveryScope`, and
successful-search completion only after no production path uses them.

## 6. Existing symbols to change

### Query preparation

`src/mcp/tool_handlers/query/edges.rs`

- Replace `edge_hydration_requirement` for reverse TypeRef with construction of
  an `IncomingRelationshipRequirement` plus separate evidence adequacy.
- Carry `AcquisitionSatisfaction` into coverage construction.
- Do not set exactness in this file until Phase 6.

`src/mcp/tool_handlers/query/prepare.rs`

- Replace the request-local memo key's implicit direction/name/requirement
  identity with the canonical requirement fingerprint.
- Batch reuse is permitted only when requirement subsumption is explicit.

`src/mcp/tool_handlers/query/batch.rs`

- Union coverage units before acquisition.
- Reprepare indexed items only after union acquisition freezes the final
  compatible evidence generation.

### Hydration/acquisition orchestration

`src/mcp/tool_handlers/hydration.rs`

- Change `hydrate_workspace_index_for` to accept the typed acquisition and
  evidence requirements.
- Replace `successful_roots` authority with validated-unit satisfaction.
- Union provider hints and Clean residual candidates before `select_candidates`.
- Commit proofs only after publication and reconciliation succeed.
- Extend `HydrationReport` with an internal satisfaction record; preserve sparse
  public diagnostics during migration.

`src/mcp/tool_handlers/hydration/discovery.rs` or current discovery body

- Ready controls whether CBM is queried, not whether a root is covered.
- Provider success appends hints.
- Only validated claims satisfy units.
- Residual units dispatch to Clean acquisition.

`src/mcp/tool_handlers/hydration/publication.rs`

- Preserve current `has_current_legacy_edit_projection` semantics.
- Return per-owner represented/published/failed outcomes sufficient to prove
  every candidate has adequate current evidence.
- Do not count compilation as acquisition completeness.

`src/mcp/tool_handlers/hydration/filesystem.rs`

- Separate inventory enumeration from target candidate selection.
- Make exclusions/source-kind policy reusable by Clean proof generation.
- Retain fast parallel reads and deterministic sorting.
- Return per-file read failures so proof cannot collapse them into an aggregate
  boolean without reason.

### State and invalidation

`src/mcp/state.rs`

- Own the new acquisition completion cache.
- Expose narrow lock/check/commit/invalidate helpers; never hold the lock across
  provider calls, traversal, reads, or compilation.

`src/mcp/tool_handlers/hydration/invalidation.rs`

- Invalidate proof-backed units for affected canonical roots on edits,
  deletion/restoration, and refresh.
- Root generation invalidation is conservative and correct for inventory proof.

`src/mcp/state/source_cache.rs`

- Continue invalidating all acquisition proof state when an external source
  change cannot be attributed safely.

### Coverage and response

`src/mcp/tool_handlers/query/coverage.rs`

- Add a constructor for workspace incoming-relationship completeness derived
  only from `AcquisitionSatisfaction` plus evidence/freeze gates.
- Keep `capability_established` independent.
- Preserve `discovered_not_compiled_this_cycle` as execution accounting.

`src/mcp/tool_handlers/query/content.rs`, `src/mcp/tools.rs`, and
`src/mcp/tool_schemas.rs`

- Update only in the final exposure phase if new reason/capability fields are
  approved.
- Existing exact/lower-bound values remain wire-compatible.

## 7. Incremental phases

### Phase 1 — RED authority characterization

Add `src/tests/mcp/workspace_query_reverse_edge_completeness.rs` and register it
from `src/mcp/tool_handlers/query.rs`.

RED cases:

1. Ready CBM returns one of two C# TypeRef consumers; omitted consumer must not
   disappear behind root coverage.
2. Ready CBM returns empty; filesystem/Clean acquisition must still run.
3. Ready CBM warm repeat must not cache provider search as semantic completion.
4. Same fixture with CBM unavailable must produce the same edge set.

The unchanged current code should fail the semantic equality/fallback assertions.
Under repository policy, the user runs:

```powershell
cargo test --all-features workspace_query_reverse_edge_completeness -- --nocapture
```

Stash only the RED test and registration before production work, following
`docs/agent/architecture.md`.

### Phase 2 — provider-neutral types, no behavior change

Add requirement, unit, proof, and satisfaction types with focused unit tests:

- coverage-unit equality ignores provenance;
- different roots/producers/families/generations are distinct;
- proof mismatch rejects each required dimension;
- partial proof leaves exact residual units;
- no proof means no satisfaction even when hints exist.

Initially adapt the existing hydration request into the typed requirement while
retaining lower-bound response behavior.

Suggested user-run gate:

```powershell
cargo test --all-features acquisition_ -- --nocapture
```

### Phase 3 — honest provider adapter

Modify CBM integration so Ready/search success yields hints only. Delete the
Ready-success insertion into `successful_roots` for incoming requirements.
Synthetic test proof claims must use a separate injection API.

Update the old readiness regression whose name/expectation currently states
that Ready CBM suppresses filesystem fallback. Preserve its useful readiness
controls but change the invariant to “Ready without proof remains hints-only.”

At the end of this phase correctness is restored conservatively, but exact
completeness is still not emitted and the old cache must not record CBM-only
completion.

### Phase 4 — Clean inventory proof and residual acquisition

Implement deterministic admitted inventory manifests and capability-specific
candidate selection. Add tests for:

- root/additional-root separation;
- exclusions and supported extensions;
- hard-link semantic-owner deduplication;
- read failure prevents proof;
- deterministic digest independent of traversal order;
- content/source-kind change alters digest;
- exact written TypeRef predicate across enabled language fixtures;
- mixed synthetic provider proof leaves only the unproved unit for Clean.

The synthetic provider proof is test-only until a real provider supports the
contract. This phase demonstrates partial substitution without pretending CBM
can perform it yet.

### Phase 5 — evidence and proof-backed cache

Commit Clean/provided proofs only after every candidate is represented by
adequate current evidence and reconciliation succeeds.

Replace discovery cache tests with proof-backed cases:

- cold/warm edge and completeness equality;
- cache hit avoids repeat acquisition only for identical proof identity;
- target, scope, exclusion, producer generation, or inventory change misses;
- publication failure does not commit;
- edit/delete/restore/refresh invalidates;
- provider provenance change does not change unit identity but invalidates a
  proof whose source generation can no longer be revalidated.

Remove provider identity from completion scope. Delete old cache paths only
after all callers use the new cache.

### Phase 6 — earned exact coverage

Carry the final satisfaction record through batch freeze, identity resolution,
refinement, and reverse evaluation. Emit `authoritative_complete/exact` only for
exact builtin TypeRef when every active source-kind unit and all downstream
gates pass.

Acceptance cases:

- complete nonempty result;
- complete zero result;
- mixed provider/Clean proof set;
- unsupported/unproved language stays lower-bound;
- source read/publication/reconciliation failure stays lower-bound;
- batch order does not affect edges or completeness;
- withinPath and additional-root scope match proof scope exactly.

### Phase 7 — documentation, audit, and provider follow-up

Update:

- `docs/CLAUDE_INTEGRATION_RULES.md`;
- `docs/ARCHITECTURAL_INVARIANTS.md` with the approved invariant and test
  authority;
- relevant tool schema descriptions; and
- Issue #144 plan/acceptance records.

Create a separate CBM protocol plan if exact inventory/capability proof is to be
added. Do not block Clean correctness on that enhancement and do not relabel
current CBM output as proof.

Trace the production lifecycle from reverse query to public response before
claiming completion.

## 8. Test-file plan

| File | Responsibility |
| --- | --- |
| `src/tests/mcp/acquisition_requirement.rs` | Unit expansion, producer generations, deterministic fingerprints, subsumption. |
| `src/tests/mcp/acquisition_proof.rs` | Proof validation and rejection for every compatibility dimension. |
| `src/tests/mcp/acquisition_cache.rs` | Proof-backed key identity, commit rules, and invalidation. |
| `src/tests/mcp/workspace_query_reverse_edge_completeness.rs` | Production-path provider/Clean residual acquisition, exact/lower-bound behavior, cold/warm truth. |
| Existing `workspace_query_provider_readiness.rs` | Ready/StillIndexing/unavailable remain lifecycle states, never proof. |
| Existing `workspace_query_legacy_edit_adequacy.rs` | Current High/hash/dependency reuse remains intact. |
| Existing `workspace_query_batch_preparation.rs` | Requirement union and freeze/order invariance. |
| Existing `workspace_query_coverage.rs` and `workspace_query_exact_identity_coverage.rs` | Wire coverage semantics and counter separation. |
| Existing `hydration_filesystem.rs` | Inventory policy, exclusions, deterministic traversal, read failures. |

Every behavioral requirement must live under `src/tests/**`. Test-only provider
claims belong in hydration/acquisition test support and must be structurally
impossible to enable in production builds.

## 9. RED/GREEN sequence

For each phase that changes behavior:

1. add the narrow tracked regression and registration;
2. hand the exact focused command to the user;
3. record RED only when the intended assertion fails;
4. stash only that regression/registration;
5. implement the production slice;
6. restore the exact unchanged regression;
7. hand the same command to the user and record GREEN; and
8. run broader user-owned gates only after focused GREEN.

Do not combine all phases into one unreviewable RED suite. Structural type tests
that cannot compile against pre-change code are not behavioral RED evidence;
the Phase 1 production-path regression is the primary defect pin.

## 10. Performance preservation

The implementation has this mandatory performance acceptance criterion:

> **The common proof path must be O(N) or better over admitted source-owner
> metadata and must not require O(N) semantic compilation. Expensive semantic
> work remains proportional to owners whose required evidence is absent, stale,
> or inadequate.**

Here, `N` is the number of admitted source owners in the effective root set.
Inventory enumeration, canonical-owner deduplication, metadata/source-generation
validation, and manifest construction may each perform one bounded linear pass.
The implementation must not introduce nested owner scans, per-target full-tree
rescans inside one requirement union, or semantic recompilation merely to prove
that already adequate evidence is current.

The initial absence of CBM proof means Clean must acquire residual units. Avoid
turning that into unnecessary semantic recompilation:

- enumerate and hash supported owners once per root generation;
- union batch requirements before acquisition;
- share one inventory manifest across name-specific units in the same request;
- use provider hints as early candidates;
- use certified candidate predicates where proven;
- retain current adequate-evidence reuse;
- cache proof-backed satisfaction; and
- parallelize bounded source reads using the existing filesystem approach.

Add deterministic work-count regressions that distinguish inventory/proof work
from semantic compilation. For a fixture with `N` admitted owners and `K` owners
whose evidence is absent, stale, or inadequate, assert:

```text
inventory owners examined <= cN
semantic compilation attempts == K
warm unchanged semantic compilation attempts == 0
```

The constant `c` belongs to the concrete traversal design and should be pinned
without using wall-clock timing. Provider hints may reduce candidate-validation
work, but their absence must not cause more than linear metadata traversal or
force compilation of owners whose adequate current evidence is already proven.

Do not add a global timeout, file limit, candidate cap, or best-effort exactness.
If acquisition cannot finish, preserve lower-bound output with an explicit
reason.

## 11. Migration and removal map

| Current construct | Migration | Removal condition |
| --- | --- | --- |
| `HydrationRequirement::LegacyEdit` | Split acquisition requirement from evidence requirement. | Every hydration caller uses explicit types. |
| `DiscoveryMode::InboundReference` | Maps into incoming TypeRef requirement during transition. | No cache/acquisition authority relies on coarse mode. |
| `DiscoveryProvider` | Move to proof provenance diagnostics. | Completion identity is provider-neutral. |
| `DiscoveryScope { provider, root }` | Replace with provider-neutral coverage unit/root scope. | Old discovery cache has no callers. |
| `successful_roots` | Replace with validated unit satisfaction. | Provider success cannot suppress residual acquisition. |
| `HydrationDiscoveryCache` | Replace with proof-backed acquisition cache. | Invalidation and warm-state regressions pass. |
| aggregate `scan.completed` | Replace proof authority with manifest plus per-file outcomes. | Clean proof rejects every incomplete inventory/read. |
| unconditional generic exact-edge lower bound | Add earned TypeRef exact branch. | Full production lifecycle and schema tests pass. |

Do not retain both old and new authority paths after migration. Provider success
may remain diagnostic; it must not remain an alternate completion gate.

## 12. Architectural review gates

Stop for explicit approval if implementation discovers a need to:

- change exactness for query families beyond reverse builtin TypeRef;
- alter global timeouts, caps, budgets, concurrency, or cancellation;
- persist proof records durably;
- change external CBM wire/API behavior;
- introduce a provider registry/plugin framework;
- change workspace scope or exclusion policy;
- change semantic identity; or
- treat unsupported producer constructs as complete.

The current approval covers the provider-neutral design and this implementation
plan only after separate production authorization. It does not pre-approve
those expansions.

## 13. Verification handoff

Agents must not run Cargo under repository policy. During implementation, hand
off focused commands phase by phase. After all focused GREEN results, the user
runs the authoritative final gate from `docs/agent/verification.md`; do not
rederive or silently abbreviate it.

The eventual field gate should be one consolidated pilot after local completion:

- original `ContextA` reverse TypeRef query;
- provider enabled and unavailable controls;
- cold/warm equality;
- edge-set and completeness certificate capture; and
- performance/work-count observation.

Live Claude testing remains last resort and is not part of this plan-only phase.

## 14. Files inspected and changed

### Inspected

- approved Issue #144 investigation and design artifacts;
- `src/cbm/client.rs`;
- `src/cbm/bridge.rs` and `src/cbm/bridge/indexing.rs`;
- `src/cbm/project_search.rs`;
- `src/ir/semantic_projection.rs`;
- language producer modules and generation constants;
- `src/layers/meta/semantic.rs`;
- `src/mcp/compatibility/producer.rs`;
- `src/mcp/discovery_cache.rs`;
- hydration, filesystem, publication, invalidation, query preparation, batch,
  coverage, and response modules; and
- existing hydration/readiness/cache/adequacy/batch/coverage regressions.

### Changed

- Added only
  `docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_IMPLEMENTATION_PLAN_2026-10-17.md`.
- No production, test, configuration, generated, or provider file was changed.

## 15. Final recommendation

**APPROVE THIS FILE/SYMBOL/TEST-LEVEL PLAN FOR SEPARATE PRODUCTION
IMPLEMENTATION AUTHORIZATION.**

The first implementation must not claim that current CBM can prove a coverage
unit. It should restore correctness with CBM hints plus Clean-owned residual
proof, preserve lower-bound responses during migration, and earn exact reverse
TypeRef completeness only after the complete proof and semantic lifecycle is
integrated.
