# Agent MaxHeadRoom — Issue #144 Completeness Design 1

**Date:** 2026-10-17

**Phase:** approved design only

**Production implementation:** not authorized

**Predecessor:**
`docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_INVESTIGATION_2026-10-16.md`

## 1. Decision and scope

Issue #144 established a provider coverage authority defect in incoming-edge
acquisition. A provider that is Ready and successfully returns paths can
currently suppress Clean-CTX filesystem acquisition for a root without proving
that its result covers every source owner capable of producing the requested
incoming relationship.

The approved architectural rule is:

> **Any optional provider may substitute for authoritative acquisition only to
> the extent that it supplies proof adequate for the specific acquisition
> requirement.**

This design defines:

- the incoming-relationship acquisition requirement;
- provider-neutral proof contents and authority;
- producer and capability coverage;
- scope and generation compatibility;
- provider-hint and Clean-owned fallback reconciliation;
- completion-cache identity and invalidation;
- the transition from acquired evidence to an earned complete result; and
- the smallest incremental implementation and regression sequence.

This phase does not authorize production code, tests, Cargo, CBM execution,
MCP execution, benchmarks, or live verification.

## 2. Executive design

Introduce a provider-neutral `AcquisitionRequirement` that describes the source
evidence a query must acquire. For reverse edges its first concrete form is an
`IncomingRelationships` requirement.

Evaluate that requirement over independent **coverage units**:

```text
canonical root × source producer capability
```

An optional provider can satisfy a coverage unit only by returning a validated
`AcquisitionCoverageProof` whose scope, generation, producer capability, query
semantics, and exhaustiveness match the requirement. Provider paths without
such proof remain useful candidate hints. Clean-CTX then runs an authoritative
acquisition strategy only for the residual unsatisfied coverage units.

Provider and Clean candidates are unioned before evidence-adequacy selection.
Clean-CTX continues to compile and publish canonical semantic evidence; a
provider coverage proof never makes provider edges authoritative semantic
facts. Once every coverage unit is proven, every selected source is represented
by compatible current evidence, publication/reconciliation succeeds, and
interpretation is frozen against that same evidence generation, the query may
receive an earned completeness certificate.

The current `Ready` state remains a lifecycle prerequisite for requesting a
proof. It is never itself a proof.

The architectural center is:

> **Semantic completeness is earned by complete proof coverage of the query's
> required coverage units; neither provider readiness nor successful discovery
> completion can substitute for that proof.**

## 3. Authority model

The design separates four concepts:

| Concept | Question answered | Authority |
| --- | --- | --- |
| Provider readiness | Can this provider answer now? | Provider lifecycle only |
| Candidate hint | Which owners may be relevant? | Advisory; no negative authority |
| Acquisition coverage proof | Has a defined source universe been exhaustively evaluated for this requirement? | Clean-CTX after validation of a provider or Clean-owned proof |
| Semantic completeness certificate | Does the final result cover the required compatible semantic evidence? | Clean-CTX only |

No provider directly grants `source_complete:true`. A provider can prove that
Clean-CTX need not repeat a particular acquisition operation for a compatible
coverage unit. Clean-CTX remains responsible for:

1. validating the proof;
2. acquiring uncovered units;
3. validating candidate ownership and scope;
4. compiling or reusing adequate canonical evidence;
5. publishing and reconciling that evidence;
6. resolving/refining against one compatible frozen generation; and
7. constructing the public completeness certificate.

This preserves the existing boundary: optional providers change acquisition
cost, never semantic authority.

## 4. Incoming-relationship acquisition requirement

### 4.1 Conceptual shape

The design calls for one typed requirement rather than adding query-specific
booleans to the current `HydrationRequirement`:

```rust
enum AcquisitionRequirement {
    Declaration(DeclarationRequirement),
    IncomingRelationships(IncomingRelationshipRequirement),
}

struct IncomingRelationshipRequirement {
    target: AcquisitionTarget,
    scope: RequiredWorkspaceScope,
    relationship_families: RelationshipFamilySet,
    producer_requirements: ProducerRequirementSet,
    evidence_fidelity: SemanticFidelity,
    source_generation_policy: SourceGenerationPolicy,
}
```

The names are design-level, not a mandate for exact Rust spelling.

`HydrationRequirement::LegacyEdit` currently mixes acquisition intent with
per-file reuse fidelity. Implementation should separate:

- **what source universe must be acquired** (`AcquisitionRequirement`); and
- **what evidence makes one acquired owner adequate** (`EvidenceRequirement`).

The existing High-fidelity/current-hash/current-dependency LegacyEdit test can
become or back the latter without behavioral weakening.

### 4.2 Target

The target contains the stable information available before final semantic
resolution:

- requested written name;
- optional requested domain and entity type;
- normalized matching rules/version; and
- identity-model version.

Acquisition must not depend on post-acquisition classification that itself
requires the acquired evidence. If an exact semantic identity cannot be safely
resolved before acquisition, the requirement uses a conservative written-name
target and final identity resolution filters the published facts afterward.

### 4.3 Relationship families

The requirement names the relation families that can satisfy the query rather
than using an undifferentiated “inbound” flag. For the reported TypeRef case,
the set includes every supported producer relationship that targets a written
type identity, including `HasConstructorParameterType` and any sibling TypeRef
relationships supported by the public query contract.

The authoritative set must come from the semantic producer registry or direct
composition of existing producers. It must not be duplicated as a handwritten
list in query code. Adding a new relationship producer must either:

- add its capability to this requirement; or
- explicitly declare that it cannot contribute to this query family.

### 4.4 Producer requirements

A producer requirement identifies:

- language/source kind;
- semantic producer and schema version;
- relationship families it can emit;
- minimum evidence fidelity;
- candidate-predicate capability; and
- supported/unsupported construct boundary.

This prevents a provider proof for C# constructor TypeRefs from silently
covering TypeScript, another C# relationship producer, or a newer producer
schema it did not index.

### 4.5 Source universe

For each configured root, the required universe is every admitted, supported
source owner for each required producer capability. Admission incorporates the
canonical workspace scope, additional roots, exclusion rules, supported source
kinds, semantic-owner normalization, and generated-source policy.

The universe is not “all files containing the target name” unless the relevant
producer declares and tests a sound name-bearing guarantee. Aliases, indirect
syntax, generated relationships, metadata, templates, or producer-specific
resolution can invalidate literal-name narrowing. For a capability without a
sound narrowing predicate, authoritative Clean acquisition must enumerate and
validate every admitted owner for that coverage unit.

This is essential: replacing unproved provider narrowing with unproved literal
filesystem narrowing would preserve the same authority defect under a different
implementation.

## 5. Coverage units

### 5.1 Definition

A coverage unit is:

```text
canonical root identity
× source kind/language
× semantic producer capability/version
× relationship-family subset
```

Target and scope fingerprints belong to the proof/cache key but do not require
duplicating the root inventory. Splitting by producer capability lets a provider
prove the units it actually covers while Clean-CTX acquires only the residual.

The unit contains no provider identity, readiness state, provider project slug,
search status, or acquisition strategy. It describes a semantic acquisition
obligation. CBM, another optional provider, or Clean's own filesystem/compiler
path may produce proof for the same unit through different authorized
acquisition mechanisms. The completeness layer evaluates proofs, not which
mechanism produced them.

For example:

```yaml
required:
  - Root A x CSharpTypeRef
  - Root A x AngularTemplateRef
  - Root B x CSharpTypeRef

provider_proves:
  - Root A x CSharpTypeRef
  - Root B x CSharpTypeRef

clean_residual_acquisition:
  - Root A x AngularTemplateRef
```

This earns partial substitution without a whole-root rescan, a provider-specific
semantic branch, or loss of acceleration.

### 5.2 Satisfaction states

Each unit has exactly one terminal or nonterminal state:

```text
ProvenByProvider(proof)
ProvenByClean(proof)
Unsupported(reason)
Pending(reason)
Failed(reason)
```

`Unsupported` is terminal only when the public query contract explicitly
excludes that construct/source kind. It cannot convert an expected supported
producer into complete absence. `Pending` and `Failed` prevent completeness.

### 5.3 Composition

Completeness is conjunctive over required units:

```text
requirement satisfied
  iff every required coverage unit is ProvenByProvider or ProvenByClean
```

A mixed result is normal. For example, a provider may prove current C# coverage
for root A, provide only hints for root B, and lack TypeScript capability. Clean
then acquires root B/C# and both roots/TypeScript if those units are required.

## 6. Provider-neutral acquisition coverage proof

### 6.1 Proof contents

`AcquisitionCoverageProof` contains or binds to:

| Field | Required meaning |
| --- | --- |
| `provider_identity` | Stable provider/adapter identity; `clean_ctx` for Clean-owned acquisition. |
| `proof_contract_version` | Version of the proof validation contract. |
| `requirement_fingerprint` | Exact acquisition target, relationship families, and matching semantics evaluated. |
| `scope_fingerprint` | Canonical root, additional-root mapping where relevant, admission/exclusion policy, generated-source policy, and path semantics. |
| `source_generation` | Provider/source snapshot generation that the proof covers. |
| `inventory_fingerprint` | Identity/digest of the complete admitted source-owner inventory evaluated. |
| `producer_coverage` | Language, producer/schema versions, relationship families, and supported construct set. |
| `query_semantics` | Exact candidate predicate/search semantics and its version. |
| `exhaustiveness` | Affirmation that every owner in the bound inventory was evaluated under those semantics. |
| `pagination` | Proof that all pages/partitions completed with no truncation, cap, timeout, or budget exhaustion. |
| `candidate_owners` | Complete returned owner set for the bound query and unit, including a valid empty set. |
| `limitations` | Explicit exclusions; any limitation intersecting the requirement prevents satisfaction. |

An implementation may store compact fingerprints rather than large manifests,
but the provider adapter must be able to validate that those fingerprints bind
to a complete inventory. A status string, count, successful response, or
provider index state is not a substitute.

### 6.2 Proof authority

The core accepts proofs only through a provider-neutral capability interface.
Raw provider payloads do not implement or construct trusted completeness state
inside query code. The adapter translates provider output into an untrusted
proof claim; a Clean-owned validator checks compatibility and returns a typed
validated proof.

Conceptually:

```rust
trait AcquisitionProvider {
    fn acquire(
        &mut self,
        requirement: &AcquisitionRequirement,
        unit: &CoverageUnit,
    ) -> ProviderAcquisition;
}

struct ProviderAcquisition {
    hints: Vec<SourceOwner>,
    proof_claim: Option<AcquisitionCoverageProofClaim>,
}

fn validate_claim(...) -> Result<ValidatedCoverageProof, ProofRejection>;
```

This is the approved provider abstraction, not a registry/framework mandate.
If only one provider exists during the first slice, direct composition behind a
small interface is preferable to a dynamic plugin registry.

Provider identity is attached to proof provenance and invalidation dependencies,
never to `CoverageUnit`. Clean-owned acquisition is not modeled as a special
provider; it is a separate authorized proof producer satisfying the same unit
contract.

### 6.3 Trust boundary

“Validated” means structurally and semantically compatible with the Clean-CTX
requirement; it does not imply cryptographic attestation. The first CBM adapter
is local and process-configured. If remote/untrusted providers are introduced,
authentication and tamper resistance require a separate security decision.

### 6.4 Empty results

A provider may prove zero candidates only if its proof binds the empty result
to a complete compatible inventory and exhaustive query. `Ready + Ok([])` has
no negative authority by itself.

## 7. Generation and scope compatibility

### 7.1 Compatibility rule

A proof is usable only when all of the following match the requirement:

```text
canonical root identity
scope/admission/exclusion fingerprint
source inventory generation
provider index generation derived from that inventory
producer/schema capability versions
relationship-family semantics version
target/matching fingerprint
proof contract version
```

Compatibility is equality or an explicitly defined safe subsumption relation.
Do not infer compatibility from timestamps, Ready state, path similarity, or a
provider project slug.

Accordingly, the current proof identity is at minimum a binding of:

```text
coverage unit
+ canonical root/scope
+ acquisition requirement and relationship-family fingerprint
+ relevant source/inventory generation
+ exclusions/admission/generated-source policy fingerprint
+ producer/schema and matching-semantics versions
+ proof-contract version
```

Omitting any of these dimensions would merely move the present overly broad
authority from “root covered” to “coverage unit covered.” Provider provenance
can vary without changing unit identity, but a cached proof remains dependent
on the provenance-specific generation and validation facts that established it.

### 7.2 Source generation

The proof must bind the provider generation to a Clean-observable source
generation. A provider-private “generation 7” is insufficient unless the
adapter proves what source snapshot it represents. Acceptable first designs are:

- an inventory manifest digest shared with Clean-CTX;
- a provider generation paired with a validated root inventory digest; or
- an atomic source snapshot identifier owned by the indexing protocol.

The exact mechanism may follow provider capability, but no mechanism may claim
current coverage when external source changes are outside its observation
boundary.

### 7.3 Scope

Proof scope is filesystem-root based and uses the repository's canonical path
identity. Provider project slugs remain a separate namespace. A proof for one
root cannot satisfy another root merely because both map to one provider
project, and a proof must reflect the same exclusions and generated-source
policy used by the query.

### 7.4 Changes and invalidation

Any change to source inventory, source content relevant to producer discovery,
scope configuration, exclusions, supported extensions, producer/schema version,
matching semantics, or proof contract invalidates affected proof-backed cache
entries. Path-local edits may invalidate a root coverage generation even when
only one indexed semantic owner must be republished, because acquisition
exhaustiveness is a root-universe claim.

## 8. Provider hints and Clean fallback reconciliation

### 8.1 Algorithm

```text
1. Build the complete AcquisitionRequirement.
2. Expand it into required coverage units.
3. Ask each configured optional provider for hints and an optional proof claim.
4. Validate each claim against its exact unit and current requirement.
5. Mark only successfully validated units ProvenByProvider.
6. Retain every valid in-scope provider path as an advisory candidate,
   regardless of proof acceptance.
7. Compute residual units not proven by a provider.
8. Run the appropriate Clean-owned authoritative acquisition strategy for each
   residual unit.
9. Union provider hints and Clean candidates by canonical semantic owner.
10. Apply source eligibility and per-file evidence adequacy.
11. Compile/publish every selected inadequate owner.
12. Reconcile language-specific exact relations where required.
13. Commit unit proofs/cache entries only after acquisition and publication
    succeed for the compatible generation.
14. Freeze interpretation over the final compatible evidence generation.
15. Evaluate reverse edges and construct coverage from the satisfaction record.
```

### 8.2 Clean-owned acquisition strategies

A Clean proof is capability-specific:

- a sound literal-name scan may prove a unit only when the producer guarantees
  every contributing source contains the searched literal under the defined
  normalization rules;
- structured pre-scan may prove a unit when it enumerates every admitted owner
  and soundly determines candidate capability;
- full admitted-owner semantic validation is the safe fallback where no sound
  narrower predicate exists.

The strategy records the inventory and predicate version in its proof. Failed
reads, unsupported parsing inside a promised supported capability, cancellation,
budget exhaustion, or publication failure leave the unit nonterminal and revoke
completeness.

### 8.3 No duplicate semantic authority

Provider candidate paths and Clean candidates are acquisition inputs only.
They are deduplicated by canonical semantic owner before compilation. Provider
relationship rows must not be unioned directly into the final answer merely
because the provider supplied a coverage proof. The `WorkspaceIndex` remains
the semantic result authority.

### 8.4 Partial provider coverage

A rejected or partial proof does not discard useful hints. It changes only
negative authority:

```text
accepted proof -> hints + satisfied coverage unit
no/rejected proof -> hints only + residual Clean acquisition
```

This retains CBM's acceleration value without making it optional correctness.

## 9. Completion-cache design

### 9.1 Cache responsibility

The cache may remember **proven acquisition satisfaction**, not merely that a
search returned successfully. It answers:

> Has this exact coverage unit already been authoritatively acquired for this
> requirement and compatible source generation?

It does not answer whether a provider is Ready or whether some candidates were
previously found.

### 9.2 Key

The logical cache key contains:

```text
canonical coverage unit
requirement fingerprint
target/matching fingerprint
scope/admission fingerprint
source/inventory generation
producer/schema version
acquisition predicate version
proof contract version
minimum evidence requirement
```

The provider identity is proof provenance, not semantic identity. Equivalent
validated proofs from different providers can satisfy the same unit, but their
invalidation dependencies remain attached to the cached record.

### 9.3 Value

The value records:

- validated proof identity and provenance;
- candidate-owner set or a stable reference to it;
- satisfied coverage units;
- compatible source/inventory generation;
- publication completion state; and
- invalidation dependencies.

The existing generation-bump approach can remain, but the current
`(scope, discovery mode, name, optional fidelity)` identity is insufficient.
It lacks producer coverage, matching semantics, inventory proof, and proof
contract version.

### 9.4 Commit rule

Cache satisfaction only after:

- acquisition for the unit is exhaustive;
- candidate publication/reuse is successful and current;
- required reconciliation succeeds; and
- no newer source generation superseded the proof during the operation.

This is a two-phase logical operation: acquire/validate provisionally, then
commit proof-backed completion after semantic publication. A failed publication
must not leave a completion entry.

### 9.5 Invalidations

Invalidate affected entries on:

- `apply_edit`, deletion, restoration, rename, or external refresh;
- detected external source change;
- configured-root/additional-root or exclusion change;
- provider reindex onto a different inventory generation;
- producer/schema or candidate-predicate version change;
- relationship/matching semantics change; and
- failure to revalidate provider proof provenance.

No TTL is needed for correctness if generation ownership is complete. A TTL
cannot repair missing invalidation authority.

### 9.6 Warm-state invariant

A cache hit may remove repeated acquisition cost only. It must reproduce the
same satisfied units, candidate authority, compatible evidence generation, and
completeness outcome as an equivalent cold acquisition. Cache provenance and
diagnostic counters may differ; semantic truth may not.

## 10. Requirement union and batch behavior

Issue #142's domain-blind union-before-freeze rule remains controlling. Batch
preparation must:

1. collect every item's acquisition requirements;
2. union compatible producer/relationship coverage without losing stronger
   requirements;
3. acquire and publish the union;
4. freeze one final compatible semantic snapshot; and
5. interpret each item from that snapshot.

Subsumption is explicit. A proof for a broader relationship-family set,
higher evidence fidelity, identical scope, and identical current generation may
satisfy a narrower requirement. A name-specific proof cannot satisfy another
name; a C# proof cannot satisfy TypeScript; a declaration proof cannot satisfy
incoming relationships; an older producer version cannot satisfy a newer one.

## 11. Earned semantic completeness

### 11.1 Required gates

An incoming reverse-edge result can set:

```json
{
  "status": "authoritative_complete",
  "source_complete": true,
  "result_semantics": "exact",
  "omitted_possible": false
}
```

only when all gates pass:

1. **Requirement gate:** every source/producer capable of the requested
   relationship is represented in the requirement.
2. **Coverage gate:** every required coverage unit has a validated provider or
   Clean proof for the current compatible generation.
3. **Evidence gate:** every acquired candidate has adequate current canonical
   evidence or was successfully published.
4. **Reconciliation gate:** required cross-file/language reconciliation
   completed successfully.
5. **Freeze gate:** resolution, refinement, and edge evaluation use the same
   final compatible evidence snapshot.
6. **Scope gate:** returned occurrences and the coverage proof use the same
   effective workspace scope.
7. **Capability gate:** the semantic producer supports every construct promised
   by the query contract.

Failure of any gate retains lower-bound semantics and reports structured reasons.

### 11.2 `capability_established`

`capability_established` remains distinct from completeness. It means the
index/result demonstrates that a semantic identity/relation capability exists.
It must not become a synonym for source-universe coverage. A complete zero result
can have no observed edge while still possessing a valid completeness certificate.

### 11.3 Public certificate

The public response should expose a compact Clean-owned certificate, not raw
provider internals. It should identify:

- effective scope;
- compatible source generation/fingerprint;
- evidence requirements;
- covered producer capabilities;
- result semantics; and
- structured partiality reasons when incomplete.

Provider names may remain diagnostic provenance. They must not be the stated
authority for semantic completeness.

### 11.4 Lower-bound compatibility

Existing clients already understand `lower_bound`. During migration, queries
that do not earn all gates retain current fields. The design adds an earned exact
state; it does not relabel partial evidence. Whether every reverse-edge producer
can earn completeness in the first implementation slice depends on declared
producer capability and must be explicit.

## 12. Failure and partiality semantics

Recommended structured reasons include:

```text
provider_proof_absent
provider_proof_incompatible
provider_generation_stale
provider_capability_incomplete
provider_result_truncated
clean_acquisition_incomplete
source_read_failed
producer_unsupported
semantic_publication_failed
reconciliation_failed
generation_changed_during_acquisition
```

These reasons explain which gate failed. They do not expose arbitrary provider
payloads or claim that retry will solve a structural capability gap.

Budget exhaustion, cancellation, timeout, provider failure, and partial
pagination all preserve obtained hints/evidence but leave affected units
unsatisfied. They must never create a completion-cache entry.

## 13. Provider state behavior

| Provider outcome | Candidate use | Coverage authority | Clean residual acquisition |
| --- | --- | --- | --- |
| Unavailable | None | None | All required units |
| Still indexing | Returned safe hints, if any | None | All required units |
| Ready, search succeeds, no proof | Hints | None | All required units |
| Ready, incompatible/partial proof | Hints | Compatible proven subset only, if separable | Every residual unit |
| Ready, valid exhaustive proof | Hints | Exact proven units | Only residual units |
| Ready, valid empty proof | Empty candidate set | Exact proven units | Only residual units |
| Search/proof validation fails | Any already validated hints only | None for failed unit | Failed unit |

This replaces the current rule “Ready + successful search covers root” with
“validated proof covers its exact compatible units.”

## 14. CBM adapter obligations

CBM is the first provider, not the core abstraction. To suppress Clean
acquisition for a unit, its adapter must establish:

- the complete root inventory indexed by the CBM project;
- identity between that inventory generation and the current Clean scope;
- language and relationship producer coverage for the requirement;
- exhaustive inbound-query semantics for every relevant relationship producer;
- absence of internal result caps/truncation or incomplete pagination; and
- a complete candidate owner set, including authoritative empty results.

If current CBM APIs cannot provide those facts, the first implementation must
use CBM results as hints and run residual Clean acquisition. It must not invent
a certificate from `IndexingStatus::Ready`.

This limitation can be efficient if Clean reconciles against a cheap
authoritative inventory and evaluates only owners not proven by CBM. The design
does not require unconditional full filesystem semantic compilation.

## 15. Lifecycle trace

```text
Query preparation
  -> build IncomingRelationships requirement
  -> union batch requirements
  -> expand coverage units

Provider acquisition
  -> readiness gate
  -> hints + optional proof claim
  -> Clean validation

Residual Clean acquisition
  -> authoritative inventory
  -> capability-specific candidate predicate
  -> Clean coverage proof

Evidence publication
  -> union/deduplicate owners
  -> adequate-current reuse or compile
  -> source-current publication commit
  -> reconciliation

Completion commit
  -> validate generation has not advanced
  -> cache proof-backed unit satisfaction

Interpretation
  -> freeze compatible evidence
  -> resolve/refine exact identity
  -> read reverse adjacency in scope

External exposure
  -> lower-bound unless every gate is proven
  -> otherwise Clean-owned authoritative completeness certificate
```

Deletion, reset, refresh, and edit invalidate acquisition proofs before a later
query may reuse them. Semantic evidence lifecycle and acquisition-proof lifecycle
advance together but remain different types and responsibilities.

## 16. Implementation boundary proposal

Implementation is not authorized in this phase. When authorized, use incremental
slices that preserve current lower-bound behavior until exact completeness is
earned end to end.

### Slice 1 — typed requirement and characterization

- Add the incoming-relationship requirement and coverage-unit types.
- Preserve current acquisition behavior behind the new requirement.
- Add RED regressions showing Ready provider success without proof cannot cover
  a root and cannot create source completeness.
- Keep public results lower-bound.

### Slice 2 — provider-neutral proof validation

- Introduce the minimal acquisition-provider boundary and typed validation.
- Adapt CBM to return hints plus no proof unless it can supply every required
  field.
- Remove direct Ready-success root authority.

### Slice 3 — Clean residual acquisition proofs

- Define producer-specific authoritative strategies.
- Reconcile proven provider units with residual Clean units.
- Preserve per-file adequate-evidence reuse.

### Slice 4 — proof-backed cache

- Replace successful-search completion identity with requirement/proof-backed
  unit satisfaction.
- Connect every existing edit/refresh/delete/reset invalidation path.
- Prove cold/warm semantic equivalence.

### Slice 5 — earned exact result

- Carry the satisfaction record through freeze and interpretation.
- Emit exact completeness only when every gate passes.
- Retain structured lower-bound reasons otherwise.

### Slice 6 — documentation and final integration

- Update public query semantics and provider contract documentation.
- Trace creation through external response.
- Run the complete repository verification gate and the smallest consolidated
  field pilot only after local deterministic acceptance is complete.

Each slice should remain behavior-preserving except for the explicitly approved
authority change. Do not introduce a provider registry, generic policy engine,
or repository-wide completeness framework beyond the types needed by this
acquisition path.

## 17. Required regression matrix

Tracked tests must cover at least:

1. Ready provider + candidates + no proof -> hints retained, fallback required.
2. Ready provider + authoritative empty proof -> no fallback for proven unit.
3. Ready provider + proof omitting a required producer -> residual acquisition.
4. Ready provider + stale generation -> proof rejected.
5. Ready provider + incompatible root/exclusions -> proof rejected.
6. Truncated/paginated-incomplete provider result -> proof rejected.
7. Mixed provider/Clean units -> exact union with no duplicate publication.
8. Existing adequate evidence -> no recompilation, still counts toward semantic
   satisfaction.
9. Publication or reconciliation failure -> no completion-cache commit.
10. Cold/warm equivalent queries -> same edges and completeness.
11. Edit/delete/restore/refresh -> affected proof/cache invalidated.
12. Zero-edge complete fixture -> exact zero, not capability inference from a
    nonempty result.
13. Unsupported promised producer -> lower-bound with structured reason.
14. Batch items -> union-before-freeze and order-independent results.
15. Provider unavailable/StillIndexing -> Clean path remains capable of earning
    the same result.

The original small fixture remains suitable: one target, one provider-returned
consumer, and one provider-omitted consumer. Expand it only enough to exercise
producer-unit partitioning and generation invalidation.

## 18. Performance constraints

Correctness does not require discarding provider acceleration. The design allows:

- provider proofs to eliminate redundant Clean acquisition by unit;
- provider hints to prioritize or narrow Clean validation where the producer
  supplies a sound predicate;
- authoritative inventory reconciliation to scan only uncovered owners;
- current adequate semantic evidence to suppress recompilation;
- proof-backed cache hits to eliminate repeated discovery; and
- batch requirement union to avoid duplicate work.

Performance optimizations may change ordering and cost. They may not:

- omit a required coverage unit;
- reinterpret a budget as a smaller source universe;
- turn partial pagination into proof;
- treat provider readiness as exhaustiveness;
- skip generation validation; or
- emit exact completeness before terminal satisfaction.

No new global timeout, cap, budget, or concurrency policy is approved by this
design.

## 19. Rejected alternatives

### Always run filesystem fallback

Rejected as the target architecture. It restores a conservative acquisition
path but prevents proof-bearing providers from substituting equivalent work and
can make CBM acceleration irrelevant. It also fails to answer whether the
filesystem candidate predicate itself is sound for every producer.

### Continue Ready-success root coverage

Rejected. Readiness and request success do not prove inventory, producer, query,
or pagination completeness.

### Trust provider edges directly

Rejected. It would move semantic authority out of the canonical Clean
publication/index path and create provider-dependent truth.

### Increase limits or redefine counters

Rejected. Current acquisition has no relevant numeric candidate cap, and cycle
counters are not completeness evidence.

### CBM-specific branch in reverse query code

Rejected. It would encode one provider's lifecycle as core semantic policy and
repeat the authority defect for future providers.

### Generic completeness framework

Rejected for this phase. The design requires small typed acquisition,
coverage-unit, proof-validation, and satisfaction boundaries. It does not
authorize a registry or policy framework spanning unrelated query families.

## 20. Decisions fixed by this design

1. Incoming reverse queries receive an explicit source acquisition requirement.
2. Acquisition-universe coverage and per-file evidence adequacy are distinct.
3. Coverage composes by canonical root and producer capability.
4. Coverage units contain no provider identity or lifecycle state.
5. Provider and Clean acquisition are authorized proof producers for the same
   provider-neutral unit contract.
6. Provider paths are always hints unless accompanied by compatible validated
   proof.
7. Provider readiness is never completeness authority.
8. Providers substitute only the exact coverage units they prove.
9. Clean-CTX acquires residual units using capability-sound strategies.
10. Provider facts never bypass canonical Clean semantic publication.
11. Completion cache entries represent proof-backed satisfaction, not successful
   searches.
12. Exact public completeness is emitted only after acquisition, publication,
    reconciliation, generation, scope, freeze, and capability gates pass.
13. The architecture is provider-neutral; CBM is the first adapter.
14. Existing lower-bound behavior remains the safe migration default.

## 21. Deferred decisions

The implementation plan must resolve these using focused source inspection or
an explicit follow-up decision where globally observable:

- the exact inventory fingerprint mechanism CBM can expose;
- the authoritative mapping from current semantic producers to relationship
  families;
- which current producer candidate predicates have a provable name-bearing
  guarantee;
- whether proof records store candidate sets or stable references/digests;
- the compact public representation of covered producer capabilities; and
- whether the first exact-completeness slice supports only C# TypeRef or a
  larger explicitly enumerated producer set.

These are not permission to weaken proof. A producer without a resolved sound
strategy remains lower-bound.

## 22. Acceptance criteria for the eventual implementation

The design is satisfied only when:

- Ready provider success without proof cannot suppress required acquisition;
- a valid provider proof suppresses only equivalent work for exact compatible
  units;
- provider-unavailable and provider-proven executions produce identical
  semantic truth for equivalent source state;
- mixed provider and Clean acquisition can earn one completeness result;
- adequate current evidence avoids recompilation without disappearing from
  semantic coverage;
- every incomplete acquisition state stays lower-bound and explains why;
- cache hits preserve cold truth and completeness;
- generation/scope changes invalidate stale satisfaction;
- exact zero results require positive coverage proof;
- no provider-specific branch determines core semantic authority; and
- the production lifecycle reaches the public response from the default query
  path, not only through custom tests.

## 23. Files inspected and changed

### Inspected

- `docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_INVESTIGATION_2026-10-16.md`
- `docs/plans/ISSUE_142_FINAL_CROSS_CUTTING_ACCEPTANCE.md`
- `docs/agent/architecture.md`
- `docs/ARCHITECTURAL_INVARIANTS.md`
- `src/mcp/discovery_cache.rs`
- `src/mcp/tool_handlers/hydration.rs`
- `src/mcp/tool_handlers/hydration/cache.rs`
- `src/mcp/tool_handlers/hydration/discovery.rs`
- `src/mcp/tool_handlers/hydration/filesystem.rs`
- `src/mcp/tool_handlers/hydration/publication.rs`
- `src/mcp/tool_handlers/hydration/report.rs`
- `src/mcp/tool_handlers/query/coverage.rs`
- `src/mcp/tool_handlers/query/edges.rs`
- `src/mcp/tool_handlers/query/prepare.rs`
- `src/cbm/project_search.rs`
- `src/cbm/bridge/indexing.rs`

### Changed

- Added only
  `docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_DESIGN_1_2026-10-17.md`.
- No production, test, configuration, or generated file was changed.

## 24. Verification status

This is a design artifact. Focused static source inspection was performed.
No Cargo command, test, build, Clippy, MCP, CBM, benchmark, or live pilot was
run. The document must pass a whitespace check before handoff.

## 25. Final recommendation

**APPROVE THIS PROVIDER-NEUTRAL COMPLETENESS DESIGN FOR AN INCREMENTAL
IMPLEMENTATION PLAN.**

The next phase should convert these fixed decisions into a file- and test-level
implementation plan. Production implementation remains separately gated.
